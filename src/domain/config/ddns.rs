use serde::{Deserialize, Serialize};
use crate::domain::error::DomainError;
use std::fmt;
use std::hash::Hash;

/// IP 類型枚舉
/// 
/// # 變體
/// 
/// - `IPv4`: IPv4 地址
/// - `IPv6`: IPv6 地址
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum IpType {
    #[serde(rename = "ipv4")]
    IPv4,
    #[serde(rename = "ipv6")]
    IPv6,
}

impl fmt::Display for IpType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IpType::IPv4 => write!(f, "ipv4"),
            IpType::IPv6 => write!(f, "ipv6"),
        }
    }
}

impl TryFrom<&str> for IpType {
    type Error = DomainError;
    
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "ipv4" => Ok(IpType::IPv4),
            "ipv6" => Ok(IpType::IPv6),
            _ => Err(DomainError::validation(format!("Invalid IP type: {}", value))),
        }
    }
}

/// DDNS 配置結構
/// 
/// # 欄位
/// 
/// - `api_token`: Cloudflare API 令牌
/// - `zone_id`: Cloudflare 區域 ID
/// - `record_id`: DNS 記錄 ID
/// - `record_name`: DNS 記錄名稱
/// - `update_interval`: 更新間隔（秒）
/// - `ip_type`: IP 類型（IPv4 或 IPv6）
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DdnsConfig {
    pub api_token: String,
    pub zone_id: String,
    pub record_id: String, 
    pub record_name: String,
    pub update_interval: u64,  // 更新間隔（秒）
    #[serde(rename = "ip_type")]
    pub ip_type: IpType,
}

impl DdnsConfig {
    /// 驗證 DDNS 配置
    /// 
    /// # 返回
    /// 
    /// - `Result<(), DomainError>`: 成功時返回 ()，失敗時返回錯誤
    pub fn validate(&self) -> Result<(), DomainError> {
        // 驗證 API 令牌
        if self.api_token.trim().is_empty() {
            return Err(DomainError::validation("API token cannot be empty".to_string()));
        }
        
        // 驗證區域 ID
        if self.zone_id.trim().is_empty() {
            return Err(DomainError::validation("Zone ID cannot be empty".to_string()));
        }
        
        // 驗證記錄 ID
        if self.record_id.trim().is_empty() {
            return Err(DomainError::validation("Record ID cannot be empty".to_string()));
        }
        
        // 驗證記錄名稱
        if self.record_name.trim().is_empty() {
            return Err(DomainError::validation("Record name cannot be empty".to_string()));
        }
        
        // 驗證更新間隔
        if self.update_interval < 5 {
            return Err(DomainError::validation("Update interval cannot be less than 5 seconds".to_string()));
        }
        
        Ok(())
    }
}

/// DDNS 配置驗證錯誤
#[derive(Debug, thiserror::Error)]
pub enum DdnsConfigError {
    #[error("Configuration validation error: {0}")]
    ValidationError(String),
    
    #[error("Configuration loading error: {0}")]
    LoadingError(String),
    
    #[error("Missing required configuration: {0}")]
    MissingConfig(String),
} 