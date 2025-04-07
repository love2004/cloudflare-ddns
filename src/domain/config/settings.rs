use config::ConfigError;
use serde::Deserialize;
use std::env;
use log::info;

/// 伺服器設置結構
/// 
/// # 欄位
/// 
/// - `host`: 伺服器監聽的主機地址
/// - `port`: 伺服器監聽的端口
#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

/// 應用程式設置結構
/// 
/// # 欄位
/// 
/// - `server`: 伺服器相關設置
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerSettings,
}

impl Settings {
    /// 創建新的設置實例
    /// 
    /// # 功能
    /// 
    /// 從環境變量加載設置
    /// 
    /// # 返回
    /// 
    /// - `Result<Self, ConfigError>`: 成功時返回設置實例，失敗時返回錯誤
    pub fn new() -> Result<Self, ConfigError> {
        Self::from_env()
    }
    
    /// 從環境變量中獲取設置
    pub fn from_env() -> Result<Self, ConfigError> {
        let host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|e| ConfigError::Message(format!("Invalid port number: {}", e)))?;
        
        info!("從環境變量載入伺服器設置 - 主機: {}, 端口: {}", host, port);
            
        Ok(Self {
            server: ServerSettings {
                host,
                port,
            }
        })
    }
} 