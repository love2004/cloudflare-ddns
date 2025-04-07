use crate::domain::config::DdnsConfig;
use crate::domain::dns::DnsService;
use crate::domain::ip::IpService;
use crate::domain::state::StateRepository;
use crate::infrastructure::http::{ReqwestHttpClient, RetryableHttpClient};
use crate::infrastructure::ip::PublicIpService;
use crate::infrastructure::dns::CloudflareDnsService;
use crate::infrastructure::state::InMemoryStateRepository;
use crate::application::ddns::DdnsApplicationService;
use crate::application::config::ConfigService;
use crate::application::events::{EventManager, EventType, EventData, EventListener};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::time::Duration;
use log::{info, error, warn};
use async_trait::async_trait;

/// DDNS 服務事件監聽器
struct DdnsServiceEventListener {
    service_factory: Arc<ServiceFactory>,
}

#[async_trait]
impl EventListener for DdnsServiceEventListener {
    async fn handle_event(&self, event: EventData) {
        match event.event_type {
            EventType::RestartDdnsService => {
                info!("處理重啟 DDNS 服務事件");
                // 重啟所有 DDNS 服務
                self.service_factory.restart_all_ddns_services().await;
            },
            EventType::ForceUpdateDns => {
                info!("處理強制更新 DNS 記錄事件");
                // 強制更新指定的 DNS 記錄或全部記錄
                if let Some(record_name) = event.data.clone() {
                    self.service_factory.update_specific_record(&record_name).await;
                } else {
                    // 更新所有記錄
                    self.service_factory.force_update_all_dns_records().await;
                }
            },
            EventType::ConfigChanged => {
                info!("處理配置變更事件");
                // 重新加載配置並更新服務
                self.service_factory.reload_configs_and_restart_services().await;
            },
            EventType::ForceUpdateAll => {
                info!("處理強制更新所有 DNS 記錄事件");
                // 更新所有記錄
                self.service_factory.force_update_all_dns_records().await;
            },
        }
    }
}

/// 服務工廠，用於創建和組裝服務
#[derive(Clone)]
pub struct ServiceFactory {
    http_client: Arc<dyn crate::domain::http::HttpClient>,
    ip_service: Arc<dyn IpService>,
    state_repository: Arc<dyn StateRepository>,
    ddns_services: Arc<RwLock<HashMap<u64, Arc<Mutex<DdnsApplicationService>>>>>,
    event_manager: Arc<EventManager>,
    config_service: Arc<ConfigService>,
}

impl Default for ServiceFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceFactory {
    /// 創建新的服務工廠
    pub fn new() -> Self {
        // 創建基礎的 HTTP 客戶端
        let base_http_client = Arc::new(ReqwestHttpClient::new());
        
        // 創建帶有重試機制的 HTTP 客戶端
        let http_client = Arc::new(RetryableHttpClient::new(
            base_http_client.clone(),
            3, // 最大重試次數
            Duration::from_millis(500), // 重試間隔
        ));
        
        let ip_service = Arc::new(PublicIpService::new(
            base_http_client.clone(), // IP 服務使用基礎 HTTP 客戶端
            None,
            None,
        ));
        let state_repository = Arc::new(InMemoryStateRepository::new());
        let event_manager = Arc::new(EventManager::new());
        
        // 創建配置服務
        let config_service = Arc::new(ConfigService::new(event_manager.clone()));
        
        let factory = Self {
            http_client,
            ip_service,
            state_repository,
            ddns_services: Arc::new(RwLock::new(HashMap::new())),
            event_manager,
            config_service,
        };
        
        factory
    }
    
    /// 初始化事件監聽系統
    pub async fn init_event_listeners(self: &Arc<Self>) {
        // 創建並註冊事件監聽器
        let listener = Arc::new(Mutex::new(DdnsServiceEventListener {
            service_factory: self.clone(),
        }));
        
        self.event_manager.register_listener(listener).await;
        
        // 訂閱事件
        let _ = self.event_manager.subscribe(EventType::RestartDdnsService).await;
        let _ = self.event_manager.subscribe(EventType::ForceUpdateDns).await;
        let _ = self.event_manager.subscribe(EventType::ConfigChanged).await;
        let _ = self.event_manager.subscribe(EventType::ForceUpdateAll).await;
        
        // 初始化配置服務
        match self.config_service.initialize().await {
            Ok(_) => info!("配置服務初始化成功"),
            Err(e) => error!("配置服務初始化失敗: {}", e),
        }
        
        // 加載配置並創建服務
        self.load_configs_and_create_services().await;
        
        info!("已初始化事件監聽系統");
    }
    
    /// 從配置加載並創建服務
    async fn load_configs_and_create_services(&self) {
        match self.config_service.get_configs().await {
            Ok(configs) => {
                if configs.is_empty() {
                    info!("沒有找到配置，不創建 DDNS 服務");
                    return;
                }
                
                info!("從文件加載了 {} 個配置", configs.len());
                
                // 清空現有服務
                {
                    let mut services = self.ddns_services.write().await;
                    services.clear();
                }
                
                // 創建服務
                for config in configs {
                    let dns_service = self.create_dns_service(&config);
                    let service = DdnsApplicationService::new(
                        dns_service,
                        self.ip_service.clone(),
                        self.state_repository.clone(),
                        config.clone(),
                    );
                    
                    // 儲存服務實例
                    let key = self.generate_config_key(&config);
                    let mut services = self.ddns_services.write().await;
                    services.insert(key, Arc::new(Mutex::new(service)));
                    
                    info!("創建了 DDNS 服務: {}", config.record_name);
                }
            },
            Err(e) => {
                error!("加載配置失敗: {}", e);
            }
        }
    }
    
    /// 重新加載配置並重啟服務
    async fn reload_configs_and_restart_services(&self) {
        info!("重新加載配置並重啟服務");
        self.load_configs_and_create_services().await;
    }
    
    /// 創建 DNS 服務
    ///
    /// # 參數
    ///
    /// - `config`: DDNS 配置
    ///
    /// # 返回
    ///
    /// - 實現 DnsService 接口的服務實例
    pub fn create_dns_service(&self, config: &DdnsConfig) -> Arc<dyn DnsService> {
        let http_client: Arc<dyn crate::domain::http::HttpClient> = self.http_client.clone();
        Arc::new(CloudflareDnsService::new(
            http_client,
            config.clone(),
        ))
    }
    
    /// 創建 DDNS 應用服務
    ///
    /// # 參數
    ///
    /// - `config`: DDNS 配置
    ///
    /// # 返回
    ///
    /// - DDNS 應用服務實例
    pub async fn create_ddns_service(&self, config: DdnsConfig) -> DdnsApplicationService {
        let dns_service = self.create_dns_service(&config);
        
        let service = DdnsApplicationService::new(
            dns_service,
            self.ip_service.clone(),
            self.state_repository.clone(),
            config.clone(),
        );
        
        // 儲存服務實例以供 API 使用
        let key = self.generate_config_key(&config);
        let mut services = self.ddns_services.write().await;
        services.insert(key, Arc::new(Mutex::new(service.clone())));
        
        service
    }
    
    /// 獲取 IP 服務
    pub fn get_ip_service(&self) -> Arc<dyn IpService> {
        self.ip_service.clone()
    }
    
    /// 獲取事件管理器
    pub fn get_event_manager(&self) -> Arc<EventManager> {
        self.event_manager.clone()
    }
    
    /// 獲取配置服務
    pub fn get_config_service(&self) -> Arc<ConfigService> {
        self.config_service.clone()
    }
    
    /// 根據配置查找 DDNS 服務
    ///
    /// # 參數
    ///
    /// - `record_name`: 要查找的 DNS 記錄名稱
    ///
    /// # 返回
    ///
    /// - Option<Arc<Mutex<DdnsApplicationService>>>: 找到的服務實例，如果不存在則返回 None
    pub async fn find_ddns_service(&self, record_name: &str) -> Option<Arc<Mutex<DdnsApplicationService>>> {
        let services = self.ddns_services.read().await;
        for (_, service) in services.iter() {
            let service_guard = service.lock().await;
            if &service_guard.config().record_name == record_name {
                return Some(service.clone());
            }
        }
        None
    }
    
    /// 獲取第一個 DDNS 服務
    ///
    /// # 返回
    ///
    /// - Option<Arc<Mutex<DdnsApplicationService>>>: 第一個可用的服務實例，如果不存在則返回 None
    pub async fn get_first_ddns_service(&self) -> Option<Arc<Mutex<DdnsApplicationService>>> {
        let services = self.ddns_services.read().await;
        if let Some((_, service)) = services.iter().next() {
            return Some(service.clone());
        }
        None
    }
    
    /// 重啟所有 DDNS 服務
    ///
    /// 重新創建所有 DDNS 服務實例，並返回新的實例列表
    pub async fn restart_all_ddns_services(&self) {
        info!("重啟所有 DDNS 服務");
        
        // 獲取現有服務的配置
        let configs = {
            let services = self.ddns_services.read().await;
            let mut configs = Vec::new();
            for (_, service) in services.iter() {
                let service_guard = service.lock().await;
                configs.push(service_guard.config().clone());
            }
            configs
        };
        
        if configs.is_empty() {
            warn!("沒有找到要重啟的 DDNS 服務");
            return;
        }
        
        // 清空現有服務
        {
            let mut services = self.ddns_services.write().await;
            services.clear();
        }
        
        // 重新創建服務
        for config in configs {
            let dns_service = self.create_dns_service(&config);
            let service = DdnsApplicationService::new(
                dns_service,
                self.ip_service.clone(),
                self.state_repository.clone(),
                config.clone(),
            );
            
            // 儲存服務實例
            let key = self.generate_config_key(&config);
            let mut services = self.ddns_services.write().await;
            services.insert(key, Arc::new(Mutex::new(service)));
            
            info!("重新創建了 DDNS 服務: {}", config.record_name);
        }
        
        info!("所有 DDNS 服務已重啟");
    }
    
    /// 強制更新所有 DNS 記錄
    pub async fn force_update_all_dns_records(&self) {
        info!("強制更新所有 DNS 記錄");
        
        // 獲取所有服務並執行更新
        let services = {
            let services_guard = self.ddns_services.read().await;
            services_guard.values().cloned().collect::<Vec<_>>()
        };
        
        for service in services {
            let service_guard = service.lock().await;
            match service_guard.force_update().await {
                Ok((domain, ip)) => {
                    info!("強制更新 DNS 記錄成功: {} -> {}", domain, ip);
                },
                Err(e) => {
                    error!("強制更新 DNS 記錄失敗: {}", e);
                }
            }
        }
    }
    
    /// 更新特定DNS記錄
    pub async fn update_specific_record(&self, record_name: &str) -> bool {
        // 找到對應的服務
        let service_opt = self.find_ddns_service(record_name).await;
        
        if let Some(service) = service_opt {
            let service_guard = service.lock().await;
            match service_guard.force_update().await {
                Ok((domain, ip)) => {
                    info!("強制更新 DNS 記錄成功: {} -> {}", domain, ip);
                    true
                },
                Err(e) => {
                    error!("強制更新 DNS 記錄失敗: {}", e);
                    false
                }
            }
        } else {
            warn!("找不到記錄名稱為 {} 的DDNS服務", record_name);
            false
        }
    }
    
    /// 保存配置並應用變更
    pub async fn save_configs_and_apply(&self, configs: Vec<DdnsConfig>) -> Result<(), crate::domain::error::DomainError> {
        // 保存配置
        self.config_service.save_configs(configs).await?;
        
        // 重新加載配置並重啟服務
        self.reload_configs_and_restart_services().await;
        
        Ok(())
    }
    
    // 為配置生成唯一的鍵值
    fn generate_config_key(&self, config: &DdnsConfig) -> u64 {
        let mut hasher = DefaultHasher::new();
        config.record_name.hash(&mut hasher);
        config.ip_type.hash(&mut hasher);
        hasher.finish()
    }
} 