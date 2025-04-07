use cloudflare_ddns::domain::config::{DdnsConfig, IpType};
use cloudflare_ddns::domain::dns::{DnsRecord, DnsService, DnsUpdateResult};
use cloudflare_ddns::domain::error::DomainError;
use async_trait::async_trait;

// 簡單測試結構
struct TestDnsService {
    update_result: Result<DnsUpdateResult, DomainError>,
    get_record_result: Result<DnsRecord, DomainError>,
    get_records_result: Result<Vec<DnsRecord>, DomainError>,
    create_record_result: Result<DnsRecord, DomainError>,
}

#[async_trait]
impl DnsService for TestDnsService {
    async fn update_record(&self, _record: DnsRecord) -> Result<DnsUpdateResult, DomainError> {
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

#[cfg(test)]
mod dns_service_tests {
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
    
    fn create_test_dns_record() -> DnsRecord {
        DnsRecord {
            id: Some("test_record".to_string()),
            name: "test.example.com".to_string(),
            record_type: "A".to_string(),
            content: "192.168.1.1".to_string(),
            ttl: 120,
            proxied: false,
        }
    }
    
    fn generate_update_result(record: DnsRecord, updated: bool) -> DnsUpdateResult {
        DnsUpdateResult {
            record,
            updated,
        }
    }
    
    #[tokio::test]
    async fn test_update_dns_record() {
        let record = create_test_dns_record();
        let update_result = generate_update_result(record.clone(), true);
        
        let dns_service = TestDnsService {
            update_result: Ok(update_result.clone()),
            get_record_result: Ok(record.clone()),
            get_records_result: Ok(vec![record.clone()]),
            create_record_result: Ok(record.clone()),
        };
        
        let result = dns_service.update_record(record.clone()).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.updated, true);
        assert_eq!(result.record.name, "test.example.com");
    }
    
    #[tokio::test]
    async fn test_get_dns_record() {
        let record = create_test_dns_record();
        
        let dns_service = TestDnsService {
            update_result: Ok(generate_update_result(record.clone(), false)),
            get_record_result: Ok(record.clone()),
            get_records_result: Ok(vec![record.clone()]),
            create_record_result: Ok(record.clone()),
        };
        
        let result = dns_service.get_record("test_zone", "test_record").await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.name, "test.example.com");
        assert_eq!(result.content, "192.168.1.1");
    }
    
    #[tokio::test]
    async fn test_get_dns_records() {
        let record1 = DnsRecord {
            id: Some("record1".to_string()),
            name: "test1.example.com".to_string(),
            record_type: "A".to_string(),
            content: "192.168.1.1".to_string(),
            ttl: 120,
            proxied: false,
        };
        
        let record2 = DnsRecord {
            id: Some("record2".to_string()),
            name: "test2.example.com".to_string(),
            record_type: "AAAA".to_string(),
            content: "2001:db8::1".to_string(),
            ttl: 120,
            proxied: false,
        };
        
        let dns_service = TestDnsService {
            update_result: Ok(generate_update_result(record1.clone(), false)),
            get_record_result: Ok(record1.clone()),
            get_records_result: Ok(vec![record1.clone(), record2.clone()]),
            create_record_result: Ok(record1.clone()),
        };
        
        let result = dns_service.get_records("test_zone").await;
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "test1.example.com");
        assert_eq!(records[1].name, "test2.example.com");
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        let record = create_test_dns_record();
        
        let dns_service = TestDnsService {
            update_result: Err(DomainError::dns_service("更新記錄錯誤")),
            get_record_result: Err(DomainError::dns_service("獲取記錄錯誤")),
            get_records_result: Err(DomainError::dns_service("獲取記錄列表錯誤")),
            create_record_result: Err(DomainError::dns_service("創建記錄錯誤")),
        };
        
        let result = dns_service.update_record(record.clone()).await;
        assert!(result.is_err());
        
        let result = dns_service.get_record("test_zone", "test_record").await;
        assert!(result.is_err());
        
        let result = dns_service.get_records("test_zone").await;
        assert!(result.is_err());
        
        let result = dns_service.create_record("test_zone", record.clone()).await;
        assert!(result.is_err());
    }
} 