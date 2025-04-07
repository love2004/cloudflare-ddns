use crate::domain::error::DomainError;
use async_trait::async_trait;

/// IP 服務接口
#[async_trait]
pub trait IpService: Send + Sync {
    /// 獲取當前的 IPv4 地址
    /// 
    /// # 返回
    /// 
    /// - `Result<String, DomainError>`: 成功時返回 IPv4 地址，失敗時返回錯誤
    async fn get_ipv4(&self) -> Result<String, DomainError>;
    
    /// 獲取當前的 IPv6 地址
    /// 
    /// # 返回
    /// 
    /// - `Result<String, DomainError>`: 成功時返回 IPv6 地址，失敗時返回錯誤
    async fn get_ipv6(&self) -> Result<String, DomainError>;
} 