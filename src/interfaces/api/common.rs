use actix_web::{HttpResponse, http::StatusCode};
use crate::domain::error::DomainError;
use crate::application::error::ApplicationError;
use log::{error, debug};
use serde_json::json;
use actix_web::{web, Error};
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use once_cell::sync::Lazy;

/// API性能度量數據
static API_METRICS: Lazy<Arc<RwLock<HashMap<String, ApiMetric>>>> = 
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

// 活動連接計數器
static ACTIVE_CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

/// API度量數據結構
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMetric {
    /// 總請求數
    pub request_count: usize,
    /// 總響應時間（毫秒）
    pub total_response_time_ms: u64,
    /// 最快響應時間（毫秒）
    pub min_response_time_ms: u64,
    /// 最慢響應時間（毫秒）
    pub max_response_time_ms: u64,
    /// 錯誤次數
    pub error_count: usize,
}

impl Default for ApiMetric {
    fn default() -> Self {
        Self {
            request_count: 0,
            total_response_time_ms: 0,
            min_response_time_ms: u64::MAX,
            max_response_time_ms: 0,
            error_count: 0,
        }
    }
}

/// 增加活動連接計數
pub fn increment_connection_count() {
    ACTIVE_CONNECTIONS.fetch_add(1, Ordering::SeqCst);
}

/// 減少活動連接計數
pub fn decrement_connection_count() {
    ACTIVE_CONNECTIONS.fetch_sub(1, Ordering::SeqCst);
}

/// 獲取當前活動連接數
pub fn get_connection_count() -> usize {
    ACTIVE_CONNECTIONS.load(Ordering::SeqCst)
}

/// 記錄API調用度量
pub fn record_api_metric(endpoint: &str, start_time: Instant, success: bool) {
    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_millis() as u64;
    
    let mut metrics = API_METRICS.write().unwrap();
    let metric = metrics.entry(endpoint.to_string()).or_insert_with(ApiMetric::default);
    
    metric.request_count += 1;
    metric.total_response_time_ms += elapsed_ms;
    metric.min_response_time_ms = metric.min_response_time_ms.min(elapsed_ms);
    metric.max_response_time_ms = metric.max_response_time_ms.max(elapsed_ms);
    
    if !success {
        metric.error_count += 1;
    }
    
    debug!("API 調用 '{}': 耗時 {}ms", endpoint, elapsed_ms);
}

/// 獲取所有API度量數據
pub fn get_api_metrics() -> HashMap<String, ApiMetric> {
    API_METRICS.read().unwrap().clone()
}

/// API性能中間件
pub struct ApiMetricsMiddleware;

impl ApiMetricsMiddleware {
    /// 記錄API調用開始
    pub fn begin(endpoint: &str) -> (String, Instant) {
        let endpoint = endpoint.to_string();
        increment_connection_count();
        (endpoint, Instant::now())
    }
    
    /// 記錄API調用結束
    pub fn end(endpoint: String, start_time: Instant, success: bool) {
        record_api_metric(&endpoint, start_time, success);
        decrement_connection_count();
    }
}

/// 標準化的API響應格式
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// 是否成功
    pub success: bool,
    /// 數據
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// 錯誤訊息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 響應代碼
    pub code: u16,
    /// 響應時間
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_time_ms: Option<u64>,
}

impl<T: Serialize> ApiResponse<T> {
    /// 創建成功響應
    pub fn success(data: T, start_time: Option<Instant>) -> Self {
        let response_time = start_time.map(|t| t.elapsed().as_millis() as u64);
        Self {
            success: true,
            data: Some(data),
            error: None,
            code: 200,
            response_time_ms: response_time,
        }
    }
    
    /// 創建錯誤響應
    pub fn error(msg: impl Into<String>, code: u16, start_time: Option<Instant>) -> Self {
        let response_time = start_time.map(|t| t.elapsed().as_millis() as u64);
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
            code,
            response_time_ms: response_time,
        }
    }
    
    /// 轉換為HTTP響應
    pub fn into_response(self) -> HttpResponse {
        let status_code = match self.code {
            0 => 500, // 如果未設置，默認為500
            code => code,
        };
        
        HttpResponse::build(
            actix_web::http::StatusCode::from_u16(status_code).unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
        )
        .insert_header(("X-Response-Time", self.response_time_ms.unwrap_or(0).to_string()))
        .json(self)
    }
}

/// 處理領域錯誤，轉換為 HTTP 響應
/// 
/// # 參數
/// 
/// - `err`: 領域錯誤
/// - `context`: 錯誤上下文描述
/// 
/// # 返回
/// 
/// - `HttpResponse`: HTTP 錯誤響應
pub fn handle_domain_error(err: DomainError, context: &str) -> HttpResponse {
    error!("{}: {}", context, err);
    
    match err {
        DomainError::Network(_) => HttpResponse::ServiceUnavailable().json(json!({
            "error": "service_unavailable",
            "message": format!("{}", err)
        })),
        DomainError::Api(_) => HttpResponse::BadGateway().json(json!({
            "error": "bad_gateway", 
            "message": format!("{}", err)
        })),
        DomainError::DnsService(_) => HttpResponse::BadGateway().json(json!({
            "error": "dns_service_error",
            "message": format!("{}", err)
        })),
        DomainError::IpService(_) => HttpResponse::ServiceUnavailable().json(json!({
            "error": "ip_service_error",
            "message": format!("{}", err)
        })),
        DomainError::Configuration(_) => HttpResponse::InternalServerError().json(json!({
            "error": "configuration_error",
            "message": format!("{}", err)
        })),
        DomainError::Unknown(_) => HttpResponse::InternalServerError().json(json!({
            "error": "unknown_error",
            "message": format!("{}", err)
        })),
        DomainError::RetryExhausted(_) => HttpResponse::ServiceUnavailable().json(json!({
            "error": "retry_exhausted",
            "message": format!("{}", err)
        })),
        DomainError::Validation(_) => HttpResponse::BadRequest().json(json!({
            "error": "validation_error",
            "message": format!("{}", err)
        })),
        DomainError::LogicError(_) => HttpResponse::InternalServerError().json(json!({
            "error": "logic_error",
            "message": format!("{}", err)
        })),
        DomainError::SerializationError(_) => HttpResponse::InternalServerError().json(json!({
            "error": "serialization_error",
            "message": format!("{}", err)
        })),
        DomainError::Context(_, _) => HttpResponse::InternalServerError().json(json!({
            "error": "context_error",
            "message": format!("{}", err)
        })),
    }
}

/// 處理應用錯誤，轉換為 HTTP 響應
/// 
/// # 參數
/// 
/// - `err`: 應用錯誤
/// - `context`: 錯誤上下文描述
/// 
/// # 返回
/// 
/// - `HttpResponse`: HTTP 錯誤響應
pub fn handle_application_error(err: ApplicationError, context: &str) -> HttpResponse {
    error!("{}: {}", context, err);
    
    match err {
        ApplicationError::DomainError(domain_err) => handle_domain_error(domain_err, context),
        ApplicationError::ConfigError(_) => HttpResponse::InternalServerError().json(json!({
            "error": "configuration_error",
            "message": format!("{}", err)
        })),
        ApplicationError::InfrastructureError(msg) => HttpResponse::InternalServerError().json(json!({
            "error": "infrastructure_error",
            "message": msg
        })),
        ApplicationError::ApplicationError(msg) => HttpResponse::InternalServerError().json(json!({
            "error": "application_error",
            "message": msg
        })),
    }
}

/// 處理通用錯誤，轉換為 HTTP 響應
/// 
/// # 參數
/// 
/// - `message`: 錯誤消息
/// - `status`: HTTP 狀態碼
/// 
/// # 返回
/// 
/// - `HttpResponse`: HTTP 錯誤響應
pub fn handle_error(message: String, status: StatusCode) -> HttpResponse {
    error!("API錯誤: {}", message);
    
    HttpResponse::build(status).json(json!({
        "error": status.canonical_reason().unwrap_or("unknown_error"),
        "message": message
    }))
} 