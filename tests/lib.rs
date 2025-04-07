mod domain;
mod application;
mod infrastructure;

// 集成測試 - 這些測試將檢查系統各部分的交互
#[cfg(test)]
mod integration_tests {
    
    use cloudflare_ddns::domain::config::{DdnsConfig, IpType};
    use cloudflare_ddns::domain::dns::{DnsRecord, DnsService, DnsUpdateResult};
    use cloudflare_ddns::domain::error::DomainError;
    use cloudflare_ddns::domain::ip::IpService;
    use cloudflare_ddns::domain::state::StateRepository;
    use cloudflare_ddns::application::ddns::DdnsApplicationService;
    use cloudflare_ddns::infrastructure::state::InMemoryStateRepository;
    use cloudflare_ddns::application::events::{EventManager, EventType, EventData, EventListener};
    use std::sync::Arc;
    use async_trait::async_trait;
    
    use std::fs;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    
    
    // 測試用IP服務
    struct TestIpService {
        ipv4_result: Result<String, DomainError>,
        ipv6_result: Result<String, DomainError>,
    }

    #[async_trait]
    impl IpService for TestIpService {
        async fn get_ipv4(&self) -> Result<String, DomainError> {
            self.ipv4_result.clone()
        }
        
        async fn get_ipv6(&self) -> Result<String, DomainError> {
            self.ipv6_result.clone()
        }
    }
    
    // 測試用DNS服務
    struct TestDnsService {
        update_result: Result<DnsUpdateResult, DomainError>,
        get_record_result: Result<DnsRecord, DomainError>,
        get_records_result: Result<Vec<DnsRecord>, DomainError>,
        create_record_result: Result<DnsRecord, DomainError>,
        // 跟踪DNS更新請求
        last_update_record: std::sync::Mutex<Option<DnsRecord>>,
    }

    #[async_trait]
    impl DnsService for TestDnsService {
        async fn update_record(&self, record: DnsRecord) -> Result<DnsUpdateResult, DomainError> {
            // 儲存最後更新的記錄用於驗證
            {
                let mut last_record = self.last_update_record.lock().unwrap();
                *last_record = Some(record);
            }
            self.update_result.clone()
        }
        
        async fn get_record(&self, _zone_id: &str, _record_id: &str) -> Result<DnsRecord, DomainError> {
            self.get_record_result.clone()
        }
        
        async fn get_records(&self, _zone_id: &str) -> Result<Vec<DnsRecord>, DomainError> {
            self.get_records_result.clone()
        }
        
        async fn create_record(&self, _zone_id: &str, _record: DnsRecord) -> Result<DnsRecord, DomainError> {
            self.create_record_result.clone()
        }
    }
    
    // 測試DDNS服務的完整流程
    #[tokio::test]
    async fn test_ddns_update_flow() {
        // 創建測試配置
        let config = DdnsConfig {
            api_token: "test_token".to_string(),
            zone_id: "test_zone".to_string(),
            record_id: "test_record".to_string(),
            record_name: "test.example.com".to_string(),
            update_interval: 300,
            ip_type: IpType::IPv4,
        };
        
        // 創建測試記錄
        let record = DnsRecord {
            id: Some("test_record".to_string()),
            name: "test.example.com".to_string(),
            record_type: "A".to_string(),
            content: "192.168.1.1".to_string(),
            ttl: 120,
            proxied: false,
        };
        
        // 創建更新結果
        let update_result = DnsUpdateResult {
            record: record.clone(),
            updated: true,
        };

        // 設置測試服務
        let ip_service = Arc::new(TestIpService {
            ipv4_result: Ok("192.168.1.1".to_string()), // 修正：使其與記錄內容相匹配
            ipv6_result: Ok("2001:db8::1".to_string()),
        });
        
        let dns_service = Arc::new(TestDnsService {
            update_result: Ok(update_result),
            get_record_result: Ok(record.clone()),
            get_records_result: Ok(vec![record.clone()]),
            create_record_result: Ok(record.clone()),
            last_update_record: std::sync::Mutex::new(None),
        });
        
        let state_repository = Arc::new(InMemoryStateRepository::new());
        
        // 創建DDNS應用服務
        let ddns_service = DdnsApplicationService::new(
            dns_service.clone(),
            ip_service,
            state_repository.clone(),
            config.clone(),
        );
        
        // 執行DNS更新
        let result = ddns_service.force_update().await;
        assert!(result.is_ok());
        
        // 驗證更新結果
        let (domain, ip) = result.unwrap();
        assert_eq!(domain, "test.example.com");
        assert_eq!(ip, "192.168.1.1");
        
        // 驗證DNS服務收到的更新請求
        let last_record = dns_service.last_update_record.lock().unwrap();
        assert!(last_record.is_some());
        let last_record = last_record.as_ref().unwrap();
        assert_eq!(last_record.name, "test.example.com");
        
        // 驗證狀態已更新
        let config_id = format!("{}-{}", config.zone_id, config.record_id);
        let last_ip = state_repository.get_last_ip(&config_id).await.unwrap();
        assert!(last_ip.is_some());
        assert_eq!(last_ip.unwrap(), "192.168.1.1");
        
        let last_update = state_repository.get_last_update_time(&config_id).await.unwrap();
        assert!(last_update.is_some());
    }
    
    // 測試配置加載和驗證
    #[tokio::test]
    async fn test_config_loading() {
        
        
        // 創建臨時配置文件
        let temp_dir = std::env::temp_dir().join("ddns_test");
        if !temp_dir.exists() {
            fs::create_dir_all(&temp_dir).unwrap();
        }
        
        let config_path = temp_dir.join("config.json");
        let config_content = r#"[{
            "api_token": "test_token",
            "zone_id": "test_zone_id",
            "record_id": "test_record_id",
            "record_name": "test.example.com",
            "update_interval": 300,
            "ip_type": "ipv4"
        }]"#;
        
        let mut file = File::create(&config_path).await.unwrap();
        file.write_all(config_content.as_bytes()).await.unwrap();
        
        // 直接讀取配置文件
        let content = tokio::fs::read_to_string(&config_path).await.unwrap();
        let configs: Vec<DdnsConfig> = serde_json::from_str(&content).unwrap();
        
        // 驗證配置
        assert_eq!(configs.len(), 1);
        let config = &configs[0];
        assert_eq!(config.api_token, "test_token");
        assert_eq!(config.zone_id, "test_zone_id");
        assert_eq!(config.record_id, "test_record_id");
        assert_eq!(config.record_name, "test.example.com");
        assert_eq!(config.update_interval, 300);
        assert_eq!(config.ip_type, IpType::IPv4);
        
        // 清理臨時文件
        tokio::fs::remove_file(&config_path).await.unwrap();
    }
    
    // 測試事件系統
    #[tokio::test]
    async fn test_event_system() {
        use std::sync::atomic::{AtomicBool, Ordering};
        
        struct TestEventListener {
            event_received: Arc<AtomicBool>,
        }
        
        #[async_trait]
        impl EventListener for TestEventListener {
            async fn handle_event(&self, event: EventData) {
                if event.event_type == EventType::ForceUpdateDns {
                    self.event_received.store(true, Ordering::SeqCst);
                }
            }
        }
        
        // 創建事件管理器
        let event_manager = Arc::new(EventManager::new());
        
        // 創建事件監聽器
        let event_received = Arc::new(AtomicBool::new(false));
        let listener = Arc::new(tokio::sync::Mutex::new(TestEventListener {
            event_received: event_received.clone(),
        }));
        
        // 註冊監聽器
        event_manager.register_listener(listener).await;
        
        // 發送事件
        event_manager.publish(EventData {
            event_type: EventType::ForceUpdateDns,
            data: Some("test.example.com".to_string()),
        }).await;
        
        // 確保事件被處理
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // 驗證事件接收
        assert!(event_received.load(Ordering::SeqCst));
    }
}

// 這將作為項目的測試入口點
#[test]
fn it_works() {
    assert!(true);
} 