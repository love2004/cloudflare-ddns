use crate::domain::error::{DomainError, ApiErrorType};

/// 為 Result 類型添加上下文相關方法的特徵
pub trait ResultExt<T, E> {
    /// 為錯誤添加上下文信息
    fn with_context<C, F>(self, context: F) -> Result<T, DomainError>
    where
        F: FnOnce() -> C,
        C: std::fmt::Display;
        
    /// 為錯誤添加資源 ID 上下文
    fn with_resource_id<I: std::fmt::Display>(self, resource_type: &str, id: I) -> Result<T, DomainError>;
    
    /// 為錯誤添加操作名稱上下文
    fn with_operation(self, operation: &str) -> Result<T, DomainError>;
}

impl<T, E: std::error::Error> ResultExt<T, E> for Result<T, E> {
    fn with_context<C, F>(self, context: F) -> Result<T, DomainError>
    where
        F: FnOnce() -> C,
        C: std::fmt::Display,
    {
        self.map_err(|err| {
            let context_str = context().to_string();
            
            // 根據錯誤類型創建不同的 DomainError
            match err.to_string().as_str() {
                msg if msg.contains("not found") || msg.contains("找不到") => 
                    DomainError::DnsService(format!("{}: {}", context_str, err)),
                msg if msg.contains("unauthorized") || msg.contains("unauthorized") => 
                    DomainError::Api(ApiErrorType::AuthorizationError(format!("{}: {}", context_str, err))),
                msg if msg.contains("validation") || msg.contains("invalid") => 
                    DomainError::Validation(format!("{}: {}", context_str, err)),
                msg if msg.contains("conflict") => 
                    DomainError::LogicError(format!("{}: {}", context_str, err)),
                msg if msg.contains("configuration") || msg.contains("config") => 
                    DomainError::Configuration(format!("{}: {}", context_str, err)),
                _ => DomainError::LogicError(format!("{}: {}", context_str, err)),
            }
        })
    }
    
    fn with_resource_id<I: std::fmt::Display>(self, resource_type: &str, id: I) -> Result<T, DomainError> {
        self.with_context(|| format!("操作 {} (ID: {})", resource_type, id))
    }
    
    fn with_operation(self, operation: &str) -> Result<T, DomainError> {
        self.with_context(|| format!("執行操作 '{}'", operation))
    }
} 