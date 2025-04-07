use crate::domain::error::DomainError;
use crate::domain::dns::{DnsService, DnsRecord, DnsUpdateResult};
use crate::domain::http::{HttpClientExt, ArcHttpClientExt};
use crate::domain::config::DdnsConfig;
use crate::infrastructure::http::ReqwestHttpClient;
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Cloudflare API 響應結構
#[derive(Deserialize, Debug)]
struct CloudflareResponse<T> {
    success: bool,
    #[serde(default)]
    errors: Vec<serde_json::Value>,
    result: Option<T>,
}

/// DNS 記錄更新請求結構
#[derive(Serialize, Debug)]
struct UpdateRecordRequest {
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u32,
    proxied: bool,
}

/// Cloudflare DNS 服務實現
pub struct CloudflareDnsService {
    http_client: Arc<ReqwestHttpClient>,
    config: DdnsConfig,
}

impl CloudflareDnsService {
    /// 創建新的 Cloudflare DNS 服務
    ///
    /// # 參數
    ///
    /// - `http_client`: HTTP 客戶端
    /// - `config`: DDNS 配置
    pub fn new(http_client: Arc<dyn crate::domain::http::HttpClient>, config: DdnsConfig) -> Self {
        // 由於 HttpClientExt 不是對象安全的，我們需要通過類型轉換來實現
        // 在實際應用中，可能需要修改架構以避免這種類型轉換
        let reqwest_client = match http_client.downcast_arc::<ReqwestHttpClient>() {
            Ok(client) => client,
            Err(_) => {
                // 創建一個新的 ReqwestHttpClient 作為備選
                log::warn!("Unable to downcast HTTP client to ReqwestHttpClient, creating a new instance");
                Arc::new(ReqwestHttpClient::new())
            }
        };
        
        Self {
            http_client: reqwest_client,
            config,
        }
    }
    
    /// 創建 Cloudflare API 請求頭
    ///
    /// # 返回
    ///
    /// - `Result<HeaderMap, DomainError>`: 成功時返回請求頭，失敗時返回錯誤
    fn create_headers(&self) -> Result<HeaderMap, DomainError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.config.api_token)).map_err(|_| {
                DomainError::validation("Invalid API token".to_string())
            })?,
        );
        
        Ok(headers)
    }
}

#[async_trait]
impl DnsService for CloudflareDnsService {
    async fn update_record(&self, record: DnsRecord) -> Result<DnsUpdateResult, DomainError> {
        let headers = self.create_headers()?;
        
        let update_data = UpdateRecordRequest {
            record_type: record.record_type.clone(),
            name: record.name.clone(),
            content: record.content.clone(),
            ttl: record.ttl,
            proxied: record.proxied,
        };
        
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            self.config.zone_id, self.config.record_id
        );
        
        let response: CloudflareResponse<DnsRecord> = self.http_client
            .put_json(&url, Some(&update_data), Some(headers))
            .await?;
        
        if !response.success {
            let error_msg = if !response.errors.is_empty() {
                format!("Cloudflare API error: {:?}", response.errors)
            } else {
                "Unknown Cloudflare API error".to_string()
            };
            
            return Err(DomainError::LogicError(error_msg));
        }
        
        match response.result {
            Some(updated_record) => Ok(DnsUpdateResult {
                record: updated_record,
                updated: true,
            }),
            None => Err(DomainError::LogicError("No record in response".to_string())),
        }
    }
    
    async fn get_record(&self, zone_id: &str, record_id: &str) -> Result<DnsRecord, DomainError> {
        let headers = self.create_headers()?;
        
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            zone_id, record_id
        );
        
        let response: CloudflareResponse<DnsRecord> = self.http_client
            .get_json(&url, Some(headers))
            .await?;
        
        if !response.success {
            let error_msg = if !response.errors.is_empty() {
                format!("Cloudflare API error: {:?}", response.errors)
            } else {
                "Unknown Cloudflare API error".to_string()
            };
            
            return Err(DomainError::LogicError(error_msg));
        }
        
        match response.result {
            Some(record) => Ok(record),
            None => Err(DomainError::dns_service(format!("DNS record not found: {}", record_id))),
        }
    }
    
    async fn get_records(&self, zone_id: &str) -> Result<Vec<DnsRecord>, DomainError> {
        let headers = self.create_headers()?;
        
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        );
        
        let response: CloudflareResponse<Vec<DnsRecord>> = self.http_client
            .get_json(&url, Some(headers))
            .await?;
        
        if !response.success {
            let error_msg = if !response.errors.is_empty() {
                format!("Cloudflare API error: {:?}", response.errors)
            } else {
                "Unknown Cloudflare API error".to_string()
            };
            
            return Err(DomainError::LogicError(error_msg));
        }
        
        match response.result {
            Some(records) => Ok(records),
            None => Ok(Vec::new()),
        }
    }
    
    async fn create_record(&self, zone_id: &str, record: DnsRecord) -> Result<DnsRecord, DomainError> {
        let headers = self.create_headers()?;
        
        let create_data = UpdateRecordRequest {
            record_type: record.record_type.clone(),
            name: record.name.clone(),
            content: record.content.clone(),
            ttl: record.ttl,
            proxied: record.proxied,
        };
        
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        );
        
        let response: CloudflareResponse<DnsRecord> = self.http_client
            .post_json(&url, Some(&create_data), Some(headers))
            .await?;
        
        if !response.success {
            let error_msg = if !response.errors.is_empty() {
                format!("Cloudflare API error: {:?}", response.errors)
            } else {
                "Unknown Cloudflare API error".to_string()
            };
            
            return Err(DomainError::LogicError(error_msg));
        }
        
        match response.result {
            Some(record) => Ok(record),
            None => Err(DomainError::LogicError("Failed to create DNS record".to_string())),
        }
    }
} 