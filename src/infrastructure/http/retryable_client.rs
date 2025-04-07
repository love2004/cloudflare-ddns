use crate::domain::error::DomainError;
use crate::domain::http::{HttpClient, HttpClientExt, ArcHttpClientExt};
use crate::infrastructure::http::ReqwestHttpClient;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use reqwest::header::HeaderMap;
use log::{error, warn, debug};
use serde::{de::DeserializeOwned, Serialize};
use std::any::Any;

/// 帶有重試機制的 HTTP 客戶端
pub struct RetryableHttpClient {
    inner_client: Arc<dyn HttpClient>,
    // 用於 JSON 操作的內部客戶端
    inner_reqwest: Arc<ReqwestHttpClient>,
    max_retries: u32,
    retry_delay: Duration,
}

impl RetryableHttpClient {
    /// 創建新的帶有重試機制的 HTTP 客戶端
    ///
    /// # 參數
    ///
    /// - `inner_client`: 內部 HTTP 客戶端
    /// - `max_retries`: 最大重試次數
    /// - `retry_delay`: 重試間隔
    pub fn new(inner_client: Arc<dyn HttpClient>, max_retries: u32, retry_delay: Duration) -> Self {
        // 嘗試轉換內部客戶端
        let inner_reqwest = match inner_client.clone().downcast_arc::<ReqwestHttpClient>() {
            Ok(client) => client,
            Err(_) => {
                // 創建一個新的 ReqwestHttpClient 作為備選
                warn!("Unable to downcast HTTP client to ReqwestHttpClient, creating a new instance");
                Arc::new(ReqwestHttpClient::new())
            }
        };
        
        Self {
            inner_client,
            inner_reqwest,
            max_retries,
            retry_delay,
        }
    }
    
    /// 使用重試機制執行操作
    async fn with_retry<F, Fut, T>(&self, operation_name: &str, f: F) -> Result<T, DomainError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, DomainError>>,
    {
        let mut last_error = None;
        
        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                debug!("{} 嘗試 {}/{}", operation_name, attempt, self.max_retries);
                sleep(self.retry_delay).await;
            }
            
            match f().await {
                Ok(result) => {
                    if attempt > 0 {
                        debug!("{} 在第 {} 次嘗試後成功", operation_name, attempt + 1);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    if self.is_retryable(&e) {
                        warn!("{} 失敗 (嘗試 {}/{}): {}", operation_name, attempt + 1, self.max_retries + 1, e);
                        last_error = Some(e);
                    } else {
                        // 不可重試的錯誤立即返回
                        error!("{} 發生不可重試的錯誤: {}", operation_name, e);
                        return Err(e);
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| DomainError::LogicError("未知錯誤".to_string())))
    }
    
    /// 判斷錯誤是否可重試
    fn is_retryable(&self, error: &DomainError) -> bool {
        error.is_retryable()
    }
}

#[async_trait]
impl HttpClient for RetryableHttpClient {
    async fn get(&self, url: &str, headers: Option<HeaderMap>) -> Result<String, DomainError> {
        self.with_retry(&format!("GET {}", url), || {
            let inner_client = self.inner_client.clone();
            let url_owned = url.to_string();
            let headers_owned = headers.clone();
            
            async move {
                inner_client.get(&url_owned, headers_owned).await
            }
        }).await
    }
    
    async fn post(&self, url: &str, body: Option<String>, headers: Option<HeaderMap>) -> Result<String, DomainError> {
        self.with_retry(&format!("POST {}", url), || {
            let inner_client = self.inner_client.clone();
            let url_owned = url.to_string();
            let body_owned = body.clone();
            let headers_owned = headers.clone();
            
            async move {
                inner_client.post(&url_owned, body_owned, headers_owned).await
            }
        }).await
    }
    
    async fn put(&self, url: &str, body: Option<String>, headers: Option<HeaderMap>) -> Result<String, DomainError> {
        self.with_retry(&format!("PUT {}", url), || {
            let inner_client = self.inner_client.clone();
            let url_owned = url.to_string();
            let body_owned = body.clone();
            let headers_owned = headers.clone();
            
            async move {
                inner_client.put(&url_owned, body_owned, headers_owned).await
            }
        }).await
    }
    
    async fn delete(&self, url: &str, headers: Option<HeaderMap>) -> Result<String, DomainError> {
        self.with_retry(&format!("DELETE {}", url), || {
            let inner_client = self.inner_client.clone();
            let url_owned = url.to_string();
            let headers_owned = headers.clone();
            
            async move {
                inner_client.delete(&url_owned, headers_owned).await
            }
        }).await
    }

    /// 實現 as_any 方法
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl HttpClientExt for RetryableHttpClient {
    async fn get_json<T: DeserializeOwned + Send + 'static>(&self, url: &str, headers: Option<HeaderMap>) -> Result<T, DomainError> {
        self.with_retry(&format!("GET JSON {}", url), || {
            let inner_client = self.inner_reqwest.clone();
            let url_owned = url.to_string();
            let headers_owned = headers.clone();
            
            async move {
                inner_client.get_json::<T>(&url_owned, headers_owned).await
            }
        }).await
    }
    
    async fn post_json<T: DeserializeOwned + Send + 'static, U: Serialize + Send + Sync>(&self, url: &str, body: Option<&U>, headers: Option<HeaderMap>) -> Result<T, DomainError> {
        // 為 body 生成 JSON 值
        let body_json = match body {
            Some(b) => match serde_json::to_value(b) {
                Ok(json) => Some(json),
                Err(e) => return Err(DomainError::SerializationError(format!("序列化失敗: {}", e))),
            },
            None => None,
        };
        
        self.with_retry(&format!("POST JSON {}", url), || {
            let inner_client = self.inner_reqwest.clone();
            let url_owned = url.to_string();
            let headers_owned = headers.clone();
            let json_clone = body_json.clone();
            
            async move {
                match json_clone {
                    Some(json) => {
                        inner_client.post_json::<T, serde_json::Value>(&url_owned, Some(&json), headers_owned).await
                    },
                    None => inner_client.post_json::<T, U>(&url_owned, None, headers_owned).await,
                }
            }
        }).await
    }
    
    async fn put_json<T: DeserializeOwned + Send + 'static, U: Serialize + Send + Sync>(&self, url: &str, body: Option<&U>, headers: Option<HeaderMap>) -> Result<T, DomainError> {
        // 為 body 生成 JSON 值
        let body_json = match body {
            Some(b) => match serde_json::to_value(b) {
                Ok(json) => Some(json),
                Err(e) => return Err(DomainError::SerializationError(format!("序列化失敗: {}", e))),
            },
            None => None,
        };
        
        self.with_retry(&format!("PUT JSON {}", url), || {
            let inner_client = self.inner_reqwest.clone();
            let url_owned = url.to_string();
            let headers_owned = headers.clone();
            let json_clone = body_json.clone();
            
            async move {
                match json_clone {
                    Some(json) => {
                        inner_client.put_json::<T, serde_json::Value>(&url_owned, Some(&json), headers_owned).await
                    },
                    None => inner_client.put_json::<T, U>(&url_owned, None, headers_owned).await,
                }
            }
        }).await
    }
    
    async fn delete_json<T: DeserializeOwned + Send + 'static>(&self, url: &str, headers: Option<HeaderMap>) -> Result<T, DomainError> {
        self.with_retry(&format!("DELETE JSON {}", url), || {
            let inner_client = self.inner_reqwest.clone();
            let url_owned = url.to_string();
            let headers_owned = headers.clone();
            
            async move {
                inner_client.delete_json::<T>(&url_owned, headers_owned).await
            }
        }).await
    }
} 