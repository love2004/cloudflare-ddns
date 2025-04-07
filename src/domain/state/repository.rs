use crate::domain::error::DomainError;
use chrono::{DateTime, Utc};
use async_trait::async_trait;

/// 狀態條目
#[derive(Debug, Clone)]
pub struct StateEntry {
    pub last_ip: Option<String>,
    pub last_update_time: Option<DateTime<Utc>>,
}

/// 狀態存儲庫接口
#[async_trait]
pub trait StateRepository: Send + Sync {
    /// 獲取最後的 IP 地址
    /// 
    /// # 參數
    /// 
    /// - `config_id`: 配置 ID
    /// 
    /// # 返回
    /// 
    /// - `Result<Option<String>, DomainError>`: 成功時返回 IP 地址（如果有），失敗時返回錯誤
    async fn get_last_ip(&self, config_id: &str) -> Result<Option<String>, DomainError>;
    
    /// 設置最後的 IP 地址
    /// 
    /// # 參數
    /// 
    /// - `config_id`: 配置 ID
    /// - `ip`: IP 地址
    /// 
    /// # 返回
    /// 
    /// - `Result<(), DomainError>`: 成功時返回 ()，失敗時返回錯誤
    async fn set_last_ip(&self, config_id: &str, ip: &str) -> Result<(), DomainError>;
    
    /// 獲取最後的更新時間
    /// 
    /// # 參數
    /// 
    /// - `config_id`: 配置 ID
    /// 
    /// # 返回
    /// 
    /// - `Result<Option<DateTime<Utc>>, DomainError>`: 成功時返回更新時間（如果有），失敗時返回錯誤
    async fn get_last_update_time(&self, config_id: &str) -> Result<Option<DateTime<Utc>>, DomainError>;
    
    /// 設置最後的更新時間
    /// 
    /// # 參數
    /// 
    /// - `config_id`: 配置 ID
    /// - `time`: 更新時間
    /// 
    /// # 返回
    /// 
    /// - `Result<(), DomainError>`: 成功時返回 ()，失敗時返回錯誤
    async fn set_last_update_time(&self, config_id: &str, time: DateTime<Utc>) -> Result<(), DomainError>;
    
    /// 獲取完整的狀態條目
    /// 
    /// # 參數
    /// 
    /// - `config_id`: 配置 ID
    /// 
    /// # 返回
    /// 
    /// - `Result<Option<StateEntry>, DomainError>`: 成功時返回狀態條目（如果有），失敗時返回錯誤
    async fn get_state(&self, config_id: &str) -> Result<Option<StateEntry>, DomainError>;
    
    /// 設置完整的狀態條目
    /// 
    /// # 參數
    /// 
    /// - `config_id`: 配置 ID
    /// - `state`: 狀態條目
    /// 
    /// # 返回
    /// 
    /// - `Result<(), DomainError>`: 成功時返回 ()，失敗時返回錯誤
    async fn set_state(&self, config_id: &str, state: StateEntry) -> Result<(), DomainError>;
} 