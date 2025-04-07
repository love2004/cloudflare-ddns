use actix_web::{web, HttpResponse, Responder, get};
use crate::application::ServiceFactory;
use crate::domain::error::DomainError;
use serde::Serialize;
use std::sync::Arc;
use log::info;

/// IP 地址響應結構
#[derive(Serialize)]
pub struct IpResponse {
    pub ip: String,
}

/// IPv4 地址處理器
/// 
/// # 參數
/// 
/// - `service_factory`: 服務工廠
/// 
/// # 返回
/// 
/// - `impl Responder`: 返回 IPv4 地址或錯誤
#[get("/ipv4")]
pub async fn get_ipv4(service_factory: web::Data<Arc<ServiceFactory>>) -> impl Responder {
    info!("接收到 IPv4 地址請求");
    let ip_service = service_factory.get_ip_service();
    
    match ip_service.get_ipv4().await {
        Ok(ip) => {
            info!("成功獲取 IPv4 地址: {}", ip);
            HttpResponse::Ok().json(IpResponse { ip })
        },
        Err(err) => {
            info!("獲取 IPv4 地址失敗: {}", err);
            handle_error(err)
        },
    }
}

/// IPv6 地址處理器
/// 
/// # 參數
/// 
/// - `service_factory`: 服務工廠
/// 
/// # 返回
/// 
/// - `impl Responder`: 返回 IPv6 地址或錯誤
#[get("/ipv6")]
pub async fn get_ipv6(service_factory: web::Data<Arc<ServiceFactory>>) -> impl Responder {
    info!("接收到 IPv6 地址請求");
    let ip_service = service_factory.get_ip_service();
    
    match ip_service.get_ipv6().await {
        Ok(ip) => {
            info!("成功獲取 IPv6 地址: {}", ip);
            HttpResponse::Ok().json(IpResponse { ip })
        },
        Err(err) => {
            info!("獲取 IPv6 地址失敗: {}", err);
            handle_error(err)
        },
    }
}

/// 處理錯誤
/// 
/// # 參數
/// 
/// - `err`: 錯誤
/// 
/// # 返回
/// 
/// - `HttpResponse`: 返回適當的錯誤響應
fn handle_error(err: DomainError) -> HttpResponse {
    match err {
        DomainError::Validation(_) => HttpResponse::BadRequest().json(format!("{}", err)),
        _ => HttpResponse::InternalServerError().json(format!("{}", err)),
    }
} 