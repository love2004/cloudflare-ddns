use crate::domain::error::DomainError;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

/// DNS 記錄結構
/// 
/// # 欄位
/// 
/// - `id`: 記錄 ID
/// - `name`: 記錄名稱
/// - `record_type`: 記錄類型
/// - `content`: 記錄內容（IP 地址）
/// - `ttl`: 記錄 TTL（秒）
/// - `proxied`: 是否啟用代理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub id: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub content: String,
    pub ttl: u32,
    pub proxied: bool,
}

/// DNS 更新結果結構
/// 
/// # 欄位
/// 
/// - `record`: 更新後的 DNS 記錄
/// - `updated`: 是否實際進行了更新
#[derive(Debug, Clone, Serialize)]
pub struct DnsUpdateResult {
    pub record: DnsRecord,
    pub updated: bool,
}

/// DNS 服務接口
#[async_trait]
pub trait DnsService: Send + Sync {
    /// 更新 DNS 記錄
    /// 
    /// # 參數
    /// 
    /// - `record`: 要更新的 DNS 記錄
    /// 
    /// # 返回
    /// 
    /// - `Result<DnsUpdateResult, DomainError>`: 成功時返回更新結果，失敗時返回錯誤
    async fn update_record(&self, record: DnsRecord) -> Result<DnsUpdateResult, DomainError>;
    
    /// 獲取 DNS 記錄
    /// 
    /// # 參數
    /// 
    /// - `zone_id`: 區域 ID
    /// - `record_id`: 記錄 ID
    /// 
    /// # 返回
    /// 
    /// - `Result<DnsRecord, DomainError>`: 成功時返回 DNS 記錄，失敗時返回錯誤
    async fn get_record(&self, zone_id: &str, record_id: &str) -> Result<DnsRecord, DomainError>;
    
    /// 獲取區域內所有 DNS 記錄
    /// 
    /// # 參數
    /// 
    /// - `zone_id`: 區域 ID
    /// 
    /// # 返回
    /// 
    /// - `Result<Vec<DnsRecord>, DomainError>`: 成功時返回 DNS 記錄列表，失敗時返回錯誤
    async fn get_records(&self, zone_id: &str) -> Result<Vec<DnsRecord>, DomainError>;
    
    /// 創建 DNS 記錄
    /// 
    /// # 參數
    /// 
    /// - `zone_id`: 區域 ID
    /// - `record`: 要創建的 DNS 記錄
    /// 
    /// # 返回
    /// 
    /// - `Result<DnsRecord, DomainError>`: 成功時返回創建的 DNS 記錄，失敗時返回錯誤
    async fn create_record(&self, zone_id: &str, record: DnsRecord) -> Result<DnsRecord, DomainError>;
} 