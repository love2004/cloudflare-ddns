pub mod ddns;
pub mod settings;

pub use ddns::{DdnsConfig, IpType, DdnsConfigError};
pub use settings::{Settings, ServerSettings};

use std::sync::Arc;
use config::{Config, ConfigError, Environment};
use crate::domain::error::DomainError;

/// 配置加載器
#[derive(Debug, Clone)]
pub struct ConfigLoader {
    config: Arc<Config>,
}

impl ConfigLoader {
    /// 創建新的配置加載器
    pub fn new() -> Result<Self, ConfigError> {
        let mut config_builder = Config::builder();

        // 添加默認配置
        config_builder = config_builder.set_default("log.level", "info")?;
        
        // 從環境變量加載，前綴為 "DDNS_"
        config_builder = config_builder.add_source(
            Environment::with_prefix("DDNS")
                .separator("_")
                .try_parsing(true)
        );
        
        let config = config_builder.build()?;
        
        Ok(Self {
            config: Arc::new(config),
        })
    }
    
    /// 獲取 DDNS 配置
    pub fn get_ddns_config(&self) -> Result<DdnsConfig, DomainError> {
        let api_token = self.config.get_string("api_token").map_err(|e| {
            DomainError::config(format!("缺少 API 令牌: {}", e))
        })?;
        
        let zone_id = self.config.get_string("zone_id").map_err(|e| {
            DomainError::config(format!("缺少區域 ID: {}", e))
        })?;
        
        let record_id = self.config.get_string("record_id").map_err(|e| {
            DomainError::config(format!("缺少記錄 ID: {}", e))
        })?;
        
        let record_name = self.config.get_string("record_name").map_err(|e| {
            DomainError::config(format!("缺少記錄名稱: {}", e))
        })?;
        
        let update_interval = self.config.get_int("update_interval").unwrap_or(300) as u64;
        
        let ip_type_str = self.config.get_string("ip_type").unwrap_or_else(|_| "ipv4".to_string());
        let ip_type = IpType::try_from(ip_type_str.as_str())?;
        
        Ok(DdnsConfig {
            api_token,
            zone_id,
            record_id,
            record_name,
            update_interval,
            ip_type,
        })
    }
    
    /// 獲取日誌級別
    pub fn get_log_level(&self) -> String {
        self.config.get_string("log.level").unwrap_or_else(|_| "info".to_string())
    }
    
    /// 獲取原始配置
    pub fn get_raw_config(&self) -> Arc<Config> {
        self.config.clone()
    }
}

impl From<ConfigError> for DomainError {
    fn from(error: ConfigError) -> Self {
        DomainError::config(error.to_string())
    }
} 