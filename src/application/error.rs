use crate::domain::error::DomainError;
use crate::domain::config::DdnsConfigError;
use thiserror::Error;

/// 應用層錯誤
#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Domain error: {0}")]
    DomainError(#[from] DomainError),
    
    #[error("Configuration error: {0}")]
    ConfigError(#[from] DdnsConfigError),
    
    #[error("Infrastructure error: {0}")]
    InfrastructureError(String),
    
    #[error("Application error: {0}")]
    ApplicationError(String),
} 