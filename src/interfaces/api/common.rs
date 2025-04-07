use actix_web::{HttpResponse, http::StatusCode};
use crate::domain::error::DomainError;
use crate::application::error::ApplicationError;
use log::error;
use serde_json::json;

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