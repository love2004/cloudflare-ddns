use cloudflare_ddns::domain::ip::IpService;
use cloudflare_ddns::domain::error::DomainError;
use async_trait::async_trait;

// 簡單的測試構造器
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

#[cfg(test)]
mod public_ip_service_tests {
    use super::*;
    
    // 測試獲取IPv4地址
    #[tokio::test]
    async fn test_ipv4_service() {
        let ip_service = TestIpService {
            ipv4_result: Ok("192.168.1.1".to_string()),
            ipv6_result: Ok("2001:db8::1".to_string()),
        };
        
        let result = ip_service.get_ipv4().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "192.168.1.1");
    }
    
    // 測試獲取IPv6地址
    #[tokio::test]
    async fn test_ipv6_service() {
        let ip_service = TestIpService {
            ipv4_result: Ok("192.168.1.1".to_string()),
            ipv6_result: Ok("2001:db8::1".to_string()),
        };
        
        let result = ip_service.get_ipv6().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2001:db8::1");
    }
    
    // 測試錯誤處理
    #[tokio::test]
    async fn test_error_handling() {
        let ip_service = TestIpService {
            ipv4_result: Err(DomainError::ip_service("測試錯誤")),
            ipv6_result: Err(DomainError::ip_service("測試錯誤")),
        };
        
        let result = ip_service.get_ipv4().await;
        assert!(result.is_err());
        
        let result = ip_service.get_ipv6().await;
        assert!(result.is_err());
    }
} 