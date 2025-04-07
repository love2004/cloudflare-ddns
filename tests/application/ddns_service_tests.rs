use cloudflare_ddns::application::ddns::DdnsApplicationService;
use cloudflare_ddns::domain::config::{DdnsConfig, IpType};
use cloudflare_ddns::domain::dns::{DnsRecord, DnsService, DnsUpdateResult};
use cloudflare_ddns::domain::ip::IpService;
use cloudflare_ddns::domain::state::{StateRepository, StateEntry};
use cloudflare_ddns::domain::error::DomainError;
use chrono::{DateTime, Utc};
use mockall::predicate::*;
use mockall::mock;
use std::sync::Arc;

// 創建Mock
mock! {
    pub DnsMock {}
    #[async_trait::async_trait]
    impl DnsService for DnsMock {
        async fn update_record(&self, record: DnsRecord) -> Result<DnsUpdateResult, DomainError>;
        async fn get_record(&self, zone_id: &str, record_id: &str) -> Result<DnsRecord, DomainError>;
        async fn get_records(&self, zone_id: &str) -> Result<Vec<DnsRecord>, DomainError>;
        async fn create_record(&self, zone_id: &str, record: DnsRecord) -> Result<DnsRecord, DomainError>;
    }
}

mock! {
    pub IpMock {}
    #[async_trait::async_trait]
    impl IpService for IpMock {
        async fn get_ipv4(&self) -> Result<String, DomainError>;
        async fn get_ipv6(&self) -> Result<String, DomainError>;
    }
}

mock! {
    pub StateMock {}
    #[async_trait::async_trait]
    impl StateRepository for StateMock {
        async fn get_last_ip(&self, config_id: &str) -> Result<Option<String>, DomainError>;
        async fn set_last_ip(&self, config_id: &str, ip: &str) -> Result<(), DomainError>;
        async fn get_last_update_time(&self, config_id: &str) -> Result<Option<DateTime<Utc>>, DomainError>;
        async fn set_last_update_time(&self, config_id: &str, time: DateTime<Utc>) -> Result<(), DomainError>;
        async fn get_state(&self, config_id: &str) -> Result<Option<StateEntry>, DomainError>;
        async fn set_state(&self, config_id: &str, state: StateEntry) -> Result<(), DomainError>;
    }
}

#[cfg(test)]
mod ddns_application_service_tests {
    use super::*;
    

    fn create_test_config() -> DdnsConfig {
        DdnsConfig {
            api_token: "test_token".to_string(),
            zone_id: "test_zone".to_string(),
            record_id: "test_record".to_string(),
            record_name: "test.example.com".to_string(),
            update_interval: 300,
            ip_type: IpType::IPv4,
        }
    }

    fn create_dns_record(ip: &str) -> DnsRecord {
        DnsRecord {
            id: Some("test_record".to_string()),
            name: "test.example.com".to_string(),
            record_type: "A".to_string(),
            content: ip.to_string(),
            ttl: 120,
            proxied: false,
        }
    }

    #[tokio::test]
    async fn test_get_current_ip_ipv4() {
        let mut ip_mock = MockIpMock::new();
        ip_mock.expect_get_ipv4()
            .times(1)
            .returning(|| Ok("192.168.1.1".to_string()));
        
        let dns_mock = MockDnsMock::new();
        let state_mock = MockStateMock::new();
        
        let config = create_test_config();
        
        let service = DdnsApplicationService::new(
            Arc::new(dns_mock),
            Arc::new(ip_mock),
            Arc::new(state_mock),
            config,
        );
        
        let result = service.get_current_ip_for_api().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "192.168.1.1");
    }

    #[tokio::test]
    async fn test_get_current_ip_ipv6() {
        let mut ip_mock = MockIpMock::new();
        ip_mock.expect_get_ipv6()
            .times(1)
            .returning(|| Ok("2001:db8::1".to_string()));
        
        let dns_mock = MockDnsMock::new();
        let state_mock = MockStateMock::new();
        
        let mut config = create_test_config();
        config.ip_type = IpType::IPv6;
        
        let service = DdnsApplicationService::new(
            Arc::new(dns_mock),
            Arc::new(ip_mock),
            Arc::new(state_mock),
            config,
        );
        
        let result = service.get_current_ip_for_api().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2001:db8::1");
    }

    #[tokio::test]
    async fn test_update_dns_record_with_ip_change() {
        let mut ip_mock = MockIpMock::new();
        ip_mock.expect_get_ipv4()
            .times(1)
            .returning(|| Ok("192.168.1.2".to_string()));
        
        let mut dns_mock = MockDnsMock::new();
        dns_mock.expect_update_record()
            .times(1)
            .returning(|record| {
                Ok(DnsUpdateResult {
                    record,
                    updated: true,
                })
            });
        
        let mut state_mock = MockStateMock::new();
        let config_id = "test_zone-test_record".to_string();
        
        state_mock.expect_get_last_ip()
            .with(eq(config_id.clone()))
            .times(1)
            .returning(|_| Ok(Some("192.168.1.1".to_string())));
        
        state_mock.expect_set_last_ip()
            .with(eq(config_id.clone()), eq("192.168.1.2"))
            .times(1)
            .returning(|_, _| Ok(()));
        
        state_mock.expect_set_last_update_time()
            .times(1)
            .returning(|_, _| Ok(()));
        
        let config = create_test_config();
        
        let service = DdnsApplicationService::new(
            Arc::new(dns_mock),
            Arc::new(ip_mock),
            Arc::new(state_mock),
            config,
        );
        
        let result = service.update_dns_record().await;
        assert!(result.is_ok());
        let update_result = result.unwrap();
        assert!(update_result.updated);
        assert_eq!(update_result.record.content, "192.168.1.2");
    }

    #[tokio::test]
    async fn test_update_dns_record_without_ip_change() {
        let current_ip = "192.168.1.1";
        
        let mut ip_mock = MockIpMock::new();
        ip_mock.expect_get_ipv4()
            .times(1)
            .returning(move || Ok(current_ip.to_string()));
        
        let dns_mock = MockDnsMock::new();
        
        let mut state_mock = MockStateMock::new();
        let config_id = "test_zone-test_record".to_string();
        
        state_mock.expect_get_last_ip()
            .with(eq(config_id.clone()))
            .times(1)
            .returning(move |_| Ok(Some(current_ip.to_string())));
        
        let config = create_test_config();
        
        let service = DdnsApplicationService::new(
            Arc::new(dns_mock),
            Arc::new(ip_mock),
            Arc::new(state_mock),
            config,
        );
        
        let result = service.update_dns_record().await;
        assert!(result.is_ok());
        let update_result = result.unwrap();
        assert!(!update_result.updated);
        assert_eq!(update_result.record.content, current_ip);
    }

    #[tokio::test]
    async fn test_update_dns_record_first_run() {
        let current_ip = "192.168.1.1";
        
        let mut ip_mock = MockIpMock::new();
        ip_mock.expect_get_ipv4()
            .times(1)
            .returning(move || Ok(current_ip.to_string()));
        
        let mut dns_mock = MockDnsMock::new();
        dns_mock.expect_update_record()
            .times(1)
            .returning(|record| {
                Ok(DnsUpdateResult {
                    record,
                    updated: true,
                })
            });
        
        let mut state_mock = MockStateMock::new();
        let config_id = "test_zone-test_record".to_string();
        
        state_mock.expect_get_last_ip()
            .with(eq(config_id.clone()))
            .times(1)
            .returning(|_| Ok(None));
        
        state_mock.expect_set_last_ip()
            .with(eq(config_id.clone()), eq(current_ip))
            .times(1)
            .returning(|_, _| Ok(()));
        
        state_mock.expect_set_last_update_time()
            .times(1)
            .returning(|_, _| Ok(()));
        
        let config = create_test_config();
        
        let service = DdnsApplicationService::new(
            Arc::new(dns_mock),
            Arc::new(ip_mock),
            Arc::new(state_mock),
            config,
        );
        
        let result = service.update_dns_record().await;
        assert!(result.is_ok());
        let update_result = result.unwrap();
        assert!(update_result.updated);
        assert_eq!(update_result.record.content, current_ip);
    }

    #[tokio::test]
    async fn test_force_update() {
        let current_ip = "192.168.1.1";
        
        let mut ip_mock = MockIpMock::new();
        ip_mock.expect_get_ipv4()
            .times(1)
            .returning(move || Ok(current_ip.to_string()));
        
        let mut dns_mock = MockDnsMock::new();
        dns_mock.expect_update_record()
            .times(1)
            .returning(|record| {
                Ok(DnsUpdateResult {
                    record,
                    updated: true,
                })
            });
        
        let mut state_mock = MockStateMock::new();
        let config_id = "test_zone-test_record".to_string();
        
        state_mock.expect_get_last_ip()
            .with(eq(config_id.clone()))
            .times(1)
            .returning(|_| Ok(Some("192.168.1.0".to_string())));
        
        state_mock.expect_set_last_ip()
            .times(1)
            .returning(|_, _| Ok(()));
        
        state_mock.expect_set_last_update_time()
            .times(1)
            .returning(|_, _| Ok(()));
        
        let config = create_test_config();
        
        let service = DdnsApplicationService::new(
            Arc::new(dns_mock),
            Arc::new(ip_mock),
            Arc::new(state_mock),
            config,
        );
        
        let result = service.force_update().await;
        assert!(result.is_ok());
        let (domain, ip) = result.unwrap();
        assert_eq!(domain, "test.example.com");
        assert_eq!(ip, current_ip);
    }
} 