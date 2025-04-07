use std::error::Error;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum NetworkErrorType {
    #[error("連接錯誤: {0}")]
    ConnectionError(String),
    
    #[error("超時錯誤: {0}")]
    TimeoutError(String),
    
    #[error("DNS解析錯誤: {0}")]
    DnsError(String),
    
    #[error("TLS/SSL錯誤: {0}")]
    TlsError(String),
    
    #[error("請求錯誤: {0}")]
    RequestError(String),
    
    #[error("響應錯誤: {0}")]
    ResponseError(String),
    
    #[error("HTTP錯誤: 狀態碼 {0}")]
    HttpError(u16),
    
    #[error("未知網絡錯誤: {0}")]
    Unknown(String),
}

#[derive(Debug, Error, Clone)]
pub enum ApiErrorType {
    #[error("API認證錯誤: {0}")]
    AuthenticationError(String),
    
    #[error("API授權錯誤: {0}")]
    AuthorizationError(String),
    
    #[error("API資源不存在: {0}")]
    ResourceNotFoundError(String),
    
    #[error("API無效請求: {0}")]
    BadRequestError(String),
    
    #[error("API請求頻率限制: {0}")]
    RateLimitError(String),
    
    #[error("API內部錯誤: {0}")]
    ServerError(String),
    
    #[error("API未知錯誤: {0}")]
    UnknownError(String),
}

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("網絡錯誤: {0}")]
    Network(NetworkErrorType),
    
    #[error("DNS服務錯誤: {0}")]
    DnsService(String),
    
    #[error("IP服務錯誤: {0}")]
    IpService(String),
    
    #[error("配置錯誤: {0}")]
    Configuration(String),
    
    #[error("API錯誤: {0}")]
    Api(ApiErrorType),
    
    #[error("未知錯誤: {0}")]
    Unknown(String),
    
    #[error("超過最大重試次數: {0}")]
    RetryExhausted(String),
    
    #[error("驗證錯誤: {0}")]
    Validation(String),
    
    #[error("邏輯錯誤: {0}")]
    LogicError(String),
    
    #[error("序列化錯誤: {0}")]
    SerializationError(String),
    
    #[error("{0}")]
    Context(String, Box<DomainError>),
}

impl DomainError {
    pub fn network(msg: impl Into<String>) -> Self {
        DomainError::Network(NetworkErrorType::Unknown(msg.into()))
    }
    
    pub fn dns_service(msg: impl Into<String>) -> Self {
        DomainError::DnsService(msg.into())
    }
    
    pub fn ip_service(msg: impl Into<String>) -> Self {
        DomainError::IpService(msg.into())
    }
    
    pub fn config(msg: impl Into<String>) -> Self {
        DomainError::Configuration(msg.into())
    }
    
    pub fn api(msg: impl Into<String>) -> Self {
        DomainError::Api(ApiErrorType::UnknownError(msg.into()))
    }
    
    pub fn unknown(msg: impl Into<String>) -> Self {
        DomainError::Unknown(msg.into())
    }
    
    pub fn retry_exhausted(msg: impl Into<String>) -> Self {
        DomainError::RetryExhausted(msg.into())
    }
    
    pub fn validation(msg: impl Into<String>) -> Self {
        DomainError::Validation(msg.into())
    }
    
    // 添加上下文到錯誤
    pub fn context<C>(self, context: C) -> Self
    where
        C: Into<String>,
    {
        DomainError::Context(context.into(), Box::new(self))
    }

    /// 檢查錯誤是否可重試
    pub fn is_retryable(&self) -> bool {
        match self {
            DomainError::Network(net_err) => match net_err {
                NetworkErrorType::ConnectionError(_) => true,
                NetworkErrorType::TimeoutError(_) => true,
                NetworkErrorType::HttpError(status) => *status >= 500 && *status < 600,
                _ => false,
            },
            DomainError::Api(api_err) => match api_err {
                ApiErrorType::RateLimitError(_) => true,
                ApiErrorType::ServerError(_) => true,
                _ => false,
            },
            _ => false,
        }
    }
    
    /// 將錯誤轉換為用戶友好的訊息
    pub fn user_friendly_message(&self) -> String {
        match self {
            DomainError::Validation(msg) => format!("輸入資料無效: {}", msg),
            DomainError::Network(NetworkErrorType::ConnectionError(_)) => "網絡連接問題，請檢查您的網絡連接並重試".to_string(),
            DomainError::Network(NetworkErrorType::TimeoutError(_)) => "網絡超時，請檢查您的網絡連接並重試".to_string(),
            DomainError::Api(ApiErrorType::RateLimitError(_)) => "請求太頻繁，請稍後再試".to_string(),
            DomainError::Api(ApiErrorType::ServerError(_)) => "服務暫時不可用，請稍後再試".to_string(),
            _ => "發生錯誤，請稍後重試".to_string(),
        }
    }
    
    /// 獲取錯誤的日誌級別
    pub fn log_level(&self) -> log::Level {
        match self {
            DomainError::Validation(_) => log::Level::Warn,
            DomainError::Network(NetworkErrorType::ConnectionError(_)) => log::Level::Error,
            DomainError::Network(NetworkErrorType::TimeoutError(_)) => log::Level::Error,
            DomainError::Api(ApiErrorType::ServerError(_)) => log::Level::Error,
            _ => log::Level::Error,
        }
    }
}

// 從 reqwest 錯誤轉換
impl From<reqwest::Error> for DomainError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            DomainError::Network(NetworkErrorType::TimeoutError(err.to_string()))
        } else if err.is_connect() {
            DomainError::Network(NetworkErrorType::ConnectionError(err.to_string()))
        } else if err.is_status() {
            let status = err.status().unwrap_or(reqwest::StatusCode::INTERNAL_SERVER_ERROR);
            DomainError::Network(NetworkErrorType::HttpError(status.as_u16()))
        } else {
            DomainError::Network(NetworkErrorType::Unknown(err.to_string()))
        }
    }
}

// 從 std::io::Error 轉換
impl From<std::io::Error> for DomainError {
    fn from(err: std::io::Error) -> Self {
        DomainError::Network(NetworkErrorType::Unknown(err.to_string()))
    }
}

// 從 serde_json::Error 轉換
impl From<serde_json::Error> for DomainError {
    fn from(err: serde_json::Error) -> Self {
        DomainError::Network(NetworkErrorType::Unknown(err.to_string()))
    }
}

// 為任何錯誤類型添加 context 方法
pub trait ErrorExt {
    fn context<C>(self, context: C) -> DomainError
    where
        C: Into<String>;
}

// 為任何實現 Error trait 的類型實現 ErrorExt
impl<E> ErrorExt for E
where
    E: Error + Send + Sync + 'static,
{
    fn context<C>(self, context: C) -> DomainError
    where
        C: Into<String>,
    {
        // 如果錯誤已經是 DomainError，則添加上下文
        if let Some(domain_err) = (&self as &dyn Error).downcast_ref::<DomainError>() {
            match domain_err {
                DomainError::Context(_, _) => {
                    // 已經有上下文，添加新上下文
                    DomainError::Context(context.into(), Box::new(domain_err.clone()))
                }
                _ => {
                    DomainError::Context(context.into(), Box::new(domain_err.clone()))
                }
            }
        } else {
            // 將其他錯誤轉換為 Unknown 並添加上下文
            DomainError::Context(
                context.into(),
                Box::new(DomainError::Unknown(format!("{}", self))),
            )
        }
    }
}

// 實現 Clone trait 以便在 Box 中使用
impl Clone for DomainError {
    fn clone(&self) -> Self {
        match self {
            DomainError::Network(s) => DomainError::Network(s.clone()),
            DomainError::DnsService(s) => DomainError::DnsService(s.clone()),
            DomainError::IpService(s) => DomainError::IpService(s.clone()),
            DomainError::Configuration(s) => DomainError::Configuration(s.clone()),
            DomainError::Api(s) => DomainError::Api(s.clone()),
            DomainError::Unknown(s) => DomainError::Unknown(s.clone()),
            DomainError::RetryExhausted(s) => DomainError::RetryExhausted(s.clone()),
            DomainError::Validation(s) => DomainError::Validation(s.clone()),
            DomainError::LogicError(s) => DomainError::LogicError(s.clone()),
            DomainError::SerializationError(s) => DomainError::SerializationError(s.clone()),
            DomainError::Context(s, e) => DomainError::Context(s.clone(), e.clone()),
        }
    }
} 