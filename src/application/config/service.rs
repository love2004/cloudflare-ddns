use crate::domain::config::DdnsConfig;
use crate::domain::error::DomainError;
use crate::infrastructure::config::FileConfigRepository;
use crate::application::events::{EventManager, EventType, EventData};
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, error};
use tokio::select;
use tokio::time::{sleep, Duration};

/// 配置服務
pub struct ConfigService {
    /// 配置存儲庫
    config_repository: Arc<FileConfigRepository>,
    /// 配置緩存
    config_cache: Arc<RwLock<Vec<DdnsConfig>>>,
    /// 事件管理器
    event_manager: Arc<EventManager>,
    /// 是否正在監視配置變更
    is_watching: Arc<RwLock<bool>>,
}

impl ConfigService {
    /// 創建新的配置服務
    pub fn new(event_manager: Arc<EventManager>) -> Self {
        let config_repository = Arc::new(FileConfigRepository::new());
        
        Self {
            config_repository,
            config_cache: Arc::new(RwLock::new(Vec::new())),
            event_manager,
            is_watching: Arc::new(RwLock::new(false)),
        }
    }

    /// 初始化配置服務
    pub async fn initialize(&self) -> Result<(), DomainError> {
        // 從文件加載配置
        let configs = self.config_repository.load_configs().await?;
        
        // 如果沒有配置，嘗試創建默認配置
        if configs.is_empty() {
            info!("沒有找到配置，將創建示例配置檔案");
            self.create_example_config().await?;
        }
        
        // 從文件重新加載配置
        let configs = self.config_repository.load_configs().await?;
        
        // 更新配置緩存
        {
            let mut cache = self.config_cache.write().await;
            *cache = configs;
        }
        
        // 開始監視配置文件變更
        self.config_repository.start_watching().await?;
        
        // 開始配置變更監聽任務
        self.start_config_watcher().await?;
        
        Ok(())
    }

    /// 創建示例配置檔案
    async fn create_example_config(&self) -> Result<(), DomainError> {
        // 從環境變量獲取配置（如果存在）
        let configs = self.get_configs_from_env()?;
        
        if !configs.is_empty() {
            // 如果從環境變量獲取到了配置，則保存到檔案
            self.config_repository.save_configs(&configs).await?;
            info!("從環境變量創建了 {} 個配置", configs.len());
            return Ok(());
        }
        
        // 如果沒有環境變量配置，則創建示例配置
        let example_config = vec![
            DdnsConfig {
                api_token: "your_cloudflare_api_token".to_string(),
                zone_id: "your_cloudflare_zone_id".to_string(),
                record_id: "your_cloudflare_record_id".to_string(),
                record_name: "your.domain.com".to_string(),
                update_interval: 300,
                ip_type: crate::domain::config::IpType::IPv4,
            }
        ];
        
        // 保存示例配置
        self.config_repository.save_configs(&example_config).await?;
        info!("創建了示例配置檔案，請編輯後使用");
        
        Ok(())
    }
    
    /// 從環境變量獲取配置
    fn get_configs_from_env(&self) -> Result<Vec<DdnsConfig>, DomainError> {
        let mut configs = Vec::new();
        
        // 檢查IPv4配置
        if let (Ok(api_token), Ok(zone_id), Ok(record_id), Ok(record_name)) = (
            std::env::var("CLOUDFLARE_API_TOKEN"),
            std::env::var("CLOUDFLARE_ZONE_ID"),
            std::env::var("CLOUDFLARE_RECORD_ID"),
            std::env::var("CLOUDFLARE_RECORD_NAME")
        ) {
            let update_interval = std::env::var("DDNS_UPDATE_INTERVAL")
                .map(|s| s.parse::<u64>().unwrap_or(300))
                .unwrap_or(300);

            configs.push(DdnsConfig {
                api_token,
                zone_id,
                record_id,
                record_name,
                update_interval,
                ip_type: crate::domain::config::IpType::IPv4,
            });
        }
        
        // 檢查IPv6配置
        if let (Ok(api_token), Ok(zone_id), Ok(record_id), Ok(record_name)) = (
            std::env::var("CLOUDFLARE_API_TOKEN"),
            std::env::var("CLOUDFLARE_ZONE_ID"),
            std::env::var("CLOUDFLARE_RECORD_ID_V6"),
            std::env::var("CLOUDFLARE_RECORD_NAME_V6")
        ) {
            let update_interval = std::env::var("DDNS_UPDATE_INTERVAL")
                .map(|s| s.parse::<u64>().unwrap_or(300))
                .unwrap_or(300);
                
            configs.push(DdnsConfig {
                api_token,
                zone_id,
                record_id,
                record_name,
                update_interval,
                ip_type: crate::domain::config::IpType::IPv6,
            });
        }
        
        Ok(configs)
    }

    /// 獲取所有配置
    pub async fn get_configs(&self) -> Result<Vec<DdnsConfig>, DomainError> {
        let cache = self.config_cache.read().await;
        Ok(cache.clone())
    }

    /// 獲取指定 IP 類型的配置
    pub async fn get_config_by_ip_type(&self, ip_type: &str) -> Result<Option<DdnsConfig>, DomainError> {
        let ip_type = crate::domain::config::IpType::try_from(ip_type)?;
        
        let cache = self.config_cache.read().await;
        let config = cache.iter()
            .find(|c| c.ip_type == ip_type)
            .cloned();
            
        Ok(config)
    }

    /// 保存配置
    pub async fn save_configs(&self, configs: Vec<DdnsConfig>) -> Result<(), DomainError> {
        // 驗證配置
        for config in &configs {
            config.validate()?;
        }
        
        // 保存到文件
        self.config_repository.save_configs(&configs).await?;
        
        // 更新緩存
        {
            let mut cache = self.config_cache.write().await;
            *cache = configs;
        }
        
        // 發布配置變更事件
        self.event_manager.publish(EventData {
            event_type: EventType::ConfigChanged,
            data: None,
        }).await;
        
        Ok(())
    }
    
    /// 開始監視配置變更
    async fn start_config_watcher(&self) -> Result<(), DomainError> {
        let mut is_watching = self.is_watching.write().await;
        if *is_watching {
            return Ok(());
        }
        
        *is_watching = true;
        
        let config_repository = self.config_repository.clone();
        let config_cache = self.config_cache.clone();
        let event_manager = self.event_manager.clone();
        let is_watching_clone = self.is_watching.clone();
        
        // 獲取配置變更接收器
        let mut change_receiver = config_repository.get_change_receiver();
        
        // 啟動監視任務
        tokio::spawn(async move {
            info!("Starting config change watcher");
            
            // 監聽配置變更信號
            while {
                let is_watching = is_watching_clone.read().await;
                *is_watching
            } {
                select! {
                    // 等待配置變更信號
                    _ = change_receiver.changed() => {
                        info!("Received config change notification");
                        
                        // 重新加載配置
                        match config_repository.load_configs().await {
                            Ok(new_configs) => {
                                // 更新緩存
                                {
                                    let mut cache = config_cache.write().await;
                                    *cache = new_configs;
                                }
                                
                                // 發布事件
                                event_manager.publish(EventData {
                                    event_type: EventType::ConfigChanged,
                                    data: None,
                                }).await;
                                
                                info!("Config reloaded successfully");
                            },
                            Err(e) => {
                                error!("Failed to reload config: {}", e);
                            }
                        }
                    },
                    // 添加超時，避免無限等待
                    _ = sleep(Duration::from_secs(60)) => {
                        // 定期檢查是否應該繼續監聽
                    }
                }
            }
            
            info!("Config change watcher stopped");
        });
        
        Ok(())
    }
    
    /// 停止監視配置變更
    pub async fn stop_config_watcher(&self) {
        // 停止仓库监视
        self.config_repository.stop_watching().await;
        
        // 停止服务监视
        let mut is_watching = self.is_watching.write().await;
        *is_watching = false;
        
        info!("Config watchers will stop on next check");
    }
} 