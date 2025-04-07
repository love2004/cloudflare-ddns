use crate::domain::error::DomainError;
use crate::domain::http::{HttpClient, HttpClientExt};
use async_trait::async_trait;
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::any::Any;

/// Reqwest HTTP 客戶端實現
pub struct ReqwestHttpClient {
    client: reqwest::Client,
}

impl Default for ReqwestHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ReqwestHttpClient {
    /// 創建新的 Reqwest HTTP 客戶端
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl HttpClient for ReqwestHttpClient {
    async fn get(&self, url: &str, headers: Option<HeaderMap>) -> Result<String, DomainError> {
        let mut request = self.client.get(url);
        
        if let Some(headers) = headers {
            request = request.headers(headers);
        }
        
        let response = request.send().await.map_err(|e| {
            DomainError::LogicError(format!("HTTP request failed: {}", e))
        })?;
        
        if !response.status().is_success() {
            return Err(DomainError::LogicError(format!(
                "HTTP request failed with status: {}", response.status()
            )));
        }
        
        response.text().await.map_err(|e| {
            DomainError::LogicError(format!("Failed to read response body: {}", e))
        })
    }
    
    async fn post(&self, url: &str, body: Option<String>, headers: Option<HeaderMap>) -> Result<String, DomainError> {
        let mut request = self.client.post(url);
        
        if let Some(headers) = headers {
            request = request.headers(headers);
        }
        
        if let Some(body) = body {
            request = request.body(body);
        }
        
        let response = request.send().await.map_err(|e| {
            DomainError::LogicError(format!("HTTP request failed: {}", e))
        })?;
        
        if !response.status().is_success() {
            return Err(DomainError::LogicError(format!(
                "HTTP request failed with status: {}", response.status()
            )));
        }
        
        response.text().await.map_err(|e| {
            DomainError::LogicError(format!("Failed to read response body: {}", e))
        })
    }
    
    async fn put(&self, url: &str, body: Option<String>, headers: Option<HeaderMap>) -> Result<String, DomainError> {
        let mut request = self.client.put(url);
        
        if let Some(headers) = headers {
            request = request.headers(headers);
        }
        
        if let Some(body) = body {
            request = request.body(body);
        }
        
        let response = request.send().await.map_err(|e| {
            DomainError::LogicError(format!("HTTP request failed: {}", e))
        })?;
        
        if !response.status().is_success() {
            return Err(DomainError::LogicError(format!(
                "HTTP request failed with status: {}", response.status()
            )));
        }
        
        response.text().await.map_err(|e| {
            DomainError::LogicError(format!("Failed to read response body: {}", e))
        })
    }
    
    async fn delete(&self, url: &str, headers: Option<HeaderMap>) -> Result<String, DomainError> {
        let mut request = self.client.delete(url);
        
        if let Some(headers) = headers {
            request = request.headers(headers);
        }
        
        let response = request.send().await.map_err(|e| {
            DomainError::LogicError(format!("HTTP request failed: {}", e))
        })?;
        
        if !response.status().is_success() {
            return Err(DomainError::LogicError(format!(
                "HTTP request failed with status: {}", response.status()
            )));
        }
        
        response.text().await.map_err(|e| {
            DomainError::LogicError(format!("Failed to read response body: {}", e))
        })
    }

    /// 實現 as_any 方法
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl HttpClientExt for ReqwestHttpClient {
    async fn get_json<T: DeserializeOwned + Send + 'static>(&self, url: &str, headers: Option<HeaderMap>) -> Result<T, DomainError> {
        let mut request = self.client.get(url);
        
        if let Some(headers) = headers {
            request = request.headers(headers);
        }
        
        let response = request.send().await.map_err(|e| {
            DomainError::LogicError(format!("HTTP request failed: {}", e))
        })?;
        
        if !response.status().is_success() {
            return Err(DomainError::LogicError(format!(
                "HTTP request failed with status: {}", response.status()
            )));
        }
        
        response.json::<T>().await.map_err(|e| {
            DomainError::LogicError(format!("Failed to parse JSON response: {}", e))
        })
    }
    
    async fn post_json<T: DeserializeOwned + Send + 'static, U: Serialize + Send + Sync>(&self, url: &str, body: Option<&U>, headers: Option<HeaderMap>) -> Result<T, DomainError> {
        let mut request = self.client.post(url);
        
        if let Some(headers) = headers {
            request = request.headers(headers);
        }
        
        if let Some(body) = body {
            request = request.json(body);
        }
        
        let response = request.send().await.map_err(|e| {
            DomainError::LogicError(format!("HTTP request failed: {}", e))
        })?;
        
        if !response.status().is_success() {
            return Err(DomainError::LogicError(format!(
                "HTTP request failed with status: {}", response.status()
            )));
        }
        
        response.json::<T>().await.map_err(|e| {
            DomainError::LogicError(format!("Failed to parse JSON response: {}", e))
        })
    }
    
    async fn put_json<T: DeserializeOwned + Send + 'static, U: Serialize + Send + Sync>(&self, url: &str, body: Option<&U>, headers: Option<HeaderMap>) -> Result<T, DomainError> {
        let mut request = self.client.put(url);
        
        if let Some(headers) = headers {
            request = request.headers(headers);
        }
        
        if let Some(body) = body {
            request = request.json(body);
        }
        
        let response = request.send().await.map_err(|e| {
            DomainError::LogicError(format!("HTTP request failed: {}", e))
        })?;
        
        if !response.status().is_success() {
            return Err(DomainError::LogicError(format!(
                "HTTP request failed with status: {}", response.status()
            )));
        }
        
        response.json::<T>().await.map_err(|e| {
            DomainError::LogicError(format!("Failed to parse JSON response: {}", e))
        })
    }
    
    async fn delete_json<T: DeserializeOwned + Send + 'static>(&self, url: &str, headers: Option<HeaderMap>) -> Result<T, DomainError> {
        let mut request = self.client.delete(url);
        
        if let Some(headers) = headers {
            request = request.headers(headers);
        }
        
        let response = request.send().await.map_err(|e| {
            DomainError::LogicError(format!("HTTP request failed: {}", e))
        })?;
        
        if !response.status().is_success() {
            return Err(DomainError::LogicError(format!(
                "HTTP request failed with status: {}", response.status()
            )));
        }
        
        response.json::<T>().await.map_err(|e| {
            DomainError::LogicError(format!("Failed to parse JSON response: {}", e))
        })
    }
} 