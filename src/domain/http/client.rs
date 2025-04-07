use crate::domain::error::DomainError;
use async_trait::async_trait;
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::any::Any;

/// 基本 HTTP 客戶端接口（可以轉換為對象的方法）
#[async_trait]
pub trait HttpClient: Send + Sync + Any {
    /// 發送 GET 請求
    /// 
    /// # 參數
    /// 
    /// - `url`: 請求的 URL
    /// - `headers`: 請求頭（可選）
    /// 
    /// # 返回
    /// 
    /// - `Result<String, DomainError>`: 成功時返回響應內容，失敗時返回錯誤
    async fn get(&self, url: &str, headers: Option<HeaderMap>) -> Result<String, DomainError>;
    
    /// 發送 POST 請求
    /// 
    /// # 參數
    /// 
    /// - `url`: 請求的 URL
    /// - `body`: 請求體（可選）
    /// - `headers`: 請求頭（可選）
    /// 
    /// # 返回
    /// 
    /// - `Result<String, DomainError>`: 成功時返回響應內容，失敗時返回錯誤
    async fn post(&self, url: &str, body: Option<String>, headers: Option<HeaderMap>) -> Result<String, DomainError>;
    
    /// 發送 PUT 請求
    /// 
    /// # 參數
    /// 
    /// - `url`: 請求的 URL
    /// - `body`: 請求體（可選）
    /// - `headers`: 請求頭（可選）
    /// 
    /// # 返回
    /// 
    /// - `Result<String, DomainError>`: 成功時返回響應內容，失敗時返回錯誤
    async fn put(&self, url: &str, body: Option<String>, headers: Option<HeaderMap>) -> Result<String, DomainError>;
    
    /// 發送 DELETE 請求
    /// 
    /// # 參數
    /// 
    /// - `url`: 請求的 URL
    /// - `headers`: 請求頭（可選）
    /// 
    /// # 返回
    /// 
    /// - `Result<String, DomainError>`: 成功時返回響應內容，失敗時返回錯誤
    async fn delete(&self, url: &str, headers: Option<HeaderMap>) -> Result<String, DomainError>;

    /// 進行類型轉換
    fn as_any(&self) -> &dyn Any;
}

/// HTTP JSON 客戶端擴展特徵（包含泛型方法）
#[async_trait]
pub trait HttpClientExt: HttpClient {
    /// 發送 GET 請求並解析 JSON 響應
    /// 
    /// # 參數
    /// 
    /// - `url`: 請求的 URL
    /// - `headers`: 請求頭（可選）
    /// 
    /// # 返回
    /// 
    /// - `Result<T, DomainError>`: 成功時返回反序列化的對象，失敗時返回錯誤
    async fn get_json<T: DeserializeOwned + Send + 'static>(&self, url: &str, headers: Option<HeaderMap>) -> Result<T, DomainError>;
    
    /// 發送 POST 請求並解析 JSON 響應
    /// 
    /// # 參數
    /// 
    /// - `url`: 請求的 URL
    /// - `body`: 請求體（可選）
    /// - `headers`: 請求頭（可選）
    /// 
    /// # 返回
    /// 
    /// - `Result<T, DomainError>`: 成功時返回反序列化的對象，失敗時返回錯誤
    async fn post_json<T: DeserializeOwned + Send + 'static, U: Serialize + Send + Sync>(&self, url: &str, body: Option<&U>, headers: Option<HeaderMap>) -> Result<T, DomainError>;
    
    /// 發送 PUT 請求並解析 JSON 響應
    /// 
    /// # 參數
    /// 
    /// - `url`: 請求的 URL
    /// - `body`: 請求體（可選）
    /// - `headers`: 請求頭（可選）
    /// 
    /// # 返回
    /// 
    /// - `Result<T, DomainError>`: 成功時返回反序列化的對象，失敗時返回錯誤
    async fn put_json<T: DeserializeOwned + Send + 'static, U: Serialize + Send + Sync>(&self, url: &str, body: Option<&U>, headers: Option<HeaderMap>) -> Result<T, DomainError>;
    
    /// 發送 DELETE 請求並解析 JSON 響應
    /// 
    /// # 參數
    /// 
    /// - `url`: 請求的 URL
    /// - `headers`: 請求頭（可選）
    /// 
    /// # 返回
    /// 
    /// - `Result<T, DomainError>`: 成功時返回反序列化的對象，失敗時返回錯誤
    async fn delete_json<T: DeserializeOwned + Send + 'static>(&self, url: &str, headers: Option<HeaderMap>) -> Result<T, DomainError>;
}

/// 擴展特徵，為 Arc<dyn HttpClient> 添加 downcast_arc 方法
pub trait ArcHttpClientExt {
    /// 嘗試將 Arc<dyn HttpClient> 轉換為 Arc<T>
    fn downcast_arc<T: HttpClient + 'static>(self) -> Result<Arc<T>, Self> where Self: Sized;
}

impl ArcHttpClientExt for Arc<dyn HttpClient> {
    fn downcast_arc<T: HttpClient + 'static>(self) -> Result<Arc<T>, Self> where Self: Sized {
        if (*self).as_any().is::<T>() {
            // 安全：我們已經檢查了類型
            let ptr = Arc::into_raw(self);
            let ptr = ptr as *const T;
            // 安全：因爲 HttpClient 要求 Send + Sync + 'static
            Ok(unsafe { Arc::from_raw(ptr) })
        } else {
            Err(self)
        }
    }
} 