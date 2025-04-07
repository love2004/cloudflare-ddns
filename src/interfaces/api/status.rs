use actix_web::{web, HttpResponse, Responder, get};
use crate::application::ServiceFactory;
use serde::Serialize;
use log::{info, warn};
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// 狀態響應結構
#[derive(Serialize)]
pub struct StatusResponse {
    status: String,
    version: String,
    last_update: Option<String>,
    ip_address: Option<String>,
    domain: Option<String>,
}

/// 服務狀態處理器
/// 
/// # 參數
/// 
/// - `service_factory`: 服務工廠
/// 
/// # 返回
/// 
/// - `impl Responder`: 返回服務狀態信息
#[get("/status")]
pub async fn get_status(service_factory: web::Data<Arc<ServiceFactory>>) -> impl Responder {
    info!("接收到狀態查詢請求");
    
    // 嘗試從服務工廠獲取運行中的DDNS服務實例
    let mut response = StatusResponse {
        status: "running".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        last_update: None,
        ip_address: None,
        domain: None,
    };
    
    // 如果找到服務實例，從中獲取狀態信息
    if let Some(service) = service_factory.get_first_ddns_service().await {
        let service_guard = service.lock().await;
        
        // 獲取配置信息
        let config = service_guard.config();
        
        // 設置域名信息
        response.domain = Some(config.record_name.clone());
        
        // 嘗試從狀態存儲獲取更多信息
        let config_id = format!("{}-{}", config.zone_id, config.record_id);
        
        // 使用服務獲取當前IP
        match service_guard.get_last_or_current_ip(&config_id).await {
            Ok(ip) => {
                response.ip_address = Some(ip);
            },
            Err(e) => {
                warn!("獲取當前IP失敗: {}", e);
            }
        }
        
        // 獲取最後更新時間
        if let Ok(last_update) = service_guard.get_last_update_for_api(&config_id).await {
            if let Some(time) = last_update {
                response.last_update = Some(format_datetime(time));
            }
        }
    } else {
        warn!("找不到運行中的DDNS服務實例");
    }
    
    info!("返回服務狀態: {}, 版本: {}", response.status, response.version);
    HttpResponse::Ok().json(response)
}

/// 格式化日期時間
fn format_datetime(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
} 