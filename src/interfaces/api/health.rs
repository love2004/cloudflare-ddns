use actix_web::{HttpResponse, Responder, get};
use serde::Serialize;

/// 健康狀態響應結構
#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
}

/// 健康檢查處理器
/// 
/// # 返回
/// 
/// - `impl Responder`: 返回健康狀態信息
#[get("/health")]
pub async fn health_check() -> impl Responder {
    let response = HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    
    HttpResponse::Ok().json(response)
} 