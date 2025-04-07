use crate::domain::error::DomainError;
use crate::domain::http::HttpClient;
use crate::domain::ip::IpService;
use crate::infrastructure::http::ReqwestHttpClient;
use async_trait::async_trait;
use std::sync::Arc;
use std::net::Ipv6Addr;
use log::{info, warn, debug};

/// 公共 IP 查詢服務實現
pub struct PublicIpService {
    http_client: Arc<ReqwestHttpClient>,
    ipv4_url: String,
    ipv6_urls: Vec<String>,
}

impl PublicIpService {
    /// 創建新的公共 IP 查詢服務
    ///
    /// # 參數
    ///
    /// - `http_client`: HTTP 客戶端
    /// - `ipv4_url`: IPv4 查詢服務的 URL（可選，默認為 https://api4.ipify.org）
    /// - `ipv6_url`: IPv6 查詢服務的 URL（可選，默認為 https://api6.ipify.org）
    pub fn new(http_client: Arc<ReqwestHttpClient>, ipv4_url: Option<String>, ipv6_url: Option<String>) -> Self {
        // 設置多個 IPv6 檢測服務
        let default_ipv6_urls = vec![
            "https://api6.ipify.org".to_string(),
            "https://v6.ident.me/".to_string(),
            "https://ifconfig.co/ip".to_string(),
        ];
        
        let ipv6_urls = match ipv6_url {
            Some(url) => vec![url],
            None => default_ipv6_urls,
        };
        
        Self {
            http_client,
            ipv4_url: ipv4_url.unwrap_or_else(|| "https://api4.ipify.org".to_string()),
            ipv6_urls,
        }
    }
    
    /// 驗證 IPv6 地址格式
    fn validate_ipv6(&self, ip: &str) -> Result<String, DomainError> {
        match ip.parse::<Ipv6Addr>() {
            Ok(_) => Ok(ip.to_string()),
            Err(_) => Err(DomainError::validation(format!("無效的 IPv6 地址格式: {}", ip))),
        }
    }
}

#[async_trait]
impl IpService for PublicIpService {
    async fn get_ipv4(&self) -> Result<String, DomainError> {
        debug!("正在獲取 IPv4 地址...");
        let ip = self.http_client.get(&self.ipv4_url, None).await?;
        debug!("獲取到 IPv4 地址: {}", ip);
        Ok(ip)
    }
    
    async fn get_ipv6(&self) -> Result<String, DomainError> {
        debug!("正在獲取 IPv6 地址...");
        
        // 遍歷所有 IPv6 檢測服務，直到成功獲取一個有效的 IPv6 地址
        for (index, url) in self.ipv6_urls.iter().enumerate() {
            debug!("嘗試從 {} 獲取 IPv6 地址", url);
            
            match self.http_client.get(url, None).await {
                Ok(ip) => {
                    match self.validate_ipv6(&ip) {
                        Ok(validated_ip) => {
                            info!("成功獲取 IPv6 地址: {}", validated_ip);
                            return Ok(validated_ip);
                        },
                        Err(e) => {
                            warn!("從 {} 獲取的 IPv6 地址無效: {}", url, e);
                            continue;
                        }
                    }
                },
                Err(e) => {
                    warn!("無法從 {} 獲取 IPv6 地址: {}", url, e);
                    if index == self.ipv6_urls.len() - 1 {
                        return Err(DomainError::network(format!("所有 IPv6 檢測服務均失敗: {}", e)));
                    }
                }
            }
        }
        
        Err(DomainError::network("無法獲取 IPv6 地址，所有服務均失敗".to_string()))
    }
} 