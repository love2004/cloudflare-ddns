use actix_web::{web, get, post, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::domain::config::DdnsConfig;
use crate::application::ServiceFactory;
use crate::application::error::ApplicationError;
use log::{info, error};
use std::sync::Arc;
use super::common::handle_application_error;
use chrono::{DateTime, Utc};

/// 擴展的DDNS配置響應
#[derive(Serialize)]
pub struct EnhancedDdnsConfig {
    #[serde(flatten)]
    config: DdnsConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_update_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
}

/// 配置響應
#[derive(Serialize)]
struct ConfigResponse {
    success: bool,
    message: String,
    configs: Option<Vec<EnhancedDdnsConfig>>,
}

/// 配置保存請求
#[derive(Deserialize)]
pub struct SaveConfigRequest {
    configs: Vec<DdnsConfig>,
}

/// 配置驗證請求
#[derive(Deserialize)]
pub struct ValidateConfigRequest {
    config: DdnsConfig,
}

/// 配置驗證響應
#[derive(Serialize)]
struct ValidateConfigResponse {
    success: bool,
    message: String,
    is_valid: bool,
}

/// 獲取當前配置
/// 
/// # 路由
/// 
/// - `GET /api/configs`
#[get("")]
pub async fn get_configs(
    service_factory: web::Data<Arc<ServiceFactory>>
) -> impl Responder {
    info!("收到獲取配置請求");
    
    let config_service = service_factory.get_config_service();
    
    match config_service.get_configs().await {
        Ok(configs) => {
            // 獲取每個配置的擴展信息
            let mut enhanced_configs = Vec::new();
            
            for config in configs {
                let config_id = format!("{}-{}", config.zone_id, config.record_id);
                let mut enhanced = EnhancedDdnsConfig {
                    config,
                    current_ip: None,
                    last_update_time: None,
                    status: Some("ok".to_string()),
                };
                
                // 尋找對應的DDNS服務獲取更多信息
                if let Some(service) = service_factory.find_ddns_service(&config_id).await {
                    let service_guard = service.lock().await;
                    
                    // 獲取當前IP
                    if let Ok(ip) = service_guard.get_last_or_current_ip(&config_id).await {
                        enhanced.current_ip = Some(ip);
                    }
                    
                    // 獲取最後更新時間
                    if let Ok(Some(time)) = service_guard.get_last_update_for_api(&config_id).await {
                        enhanced.last_update_time = Some(format_datetime(time));
                    }
                }
                
                enhanced_configs.push(enhanced);
            }
            
            HttpResponse::Ok().json(ConfigResponse {
                success: true,
                message: format!("成功獲取 {} 個配置", enhanced_configs.len()),
                configs: Some(enhanced_configs),
            })
        },
        Err(e) => {
            let app_error: ApplicationError = ApplicationError::DomainError(e);
            handle_application_error(app_error, "獲取配置失敗")
        }
    }
}

/// 保存配置
/// 
/// # 路由
/// 
/// - `POST /api/configs`
/// 
/// # 請求體
/// 
/// - 要保存的配置
/// 
/// # 返回
/// 
/// - 配置保存結果
#[post("")]
pub async fn save_configs(
    service_factory: web::Data<Arc<ServiceFactory>>,
    req: web::Json<SaveConfigRequest>
) -> impl Responder {
    info!("收到保存配置請求，共 {} 個配置", req.configs.len());
    
    let result = service_factory.save_configs_and_apply(req.configs.clone()).await;
    
    match result {
        Ok(_) => {
            // 使用相同的邏輯豐富返回的配置信息
            let mut enhanced_configs = Vec::new();
            
            for config in &req.configs {
                let config_id = format!("{}-{}", config.zone_id, config.record_id);
                let mut enhanced = EnhancedDdnsConfig {
                    config: config.clone(),
                    current_ip: None,
                    last_update_time: None,
                    status: Some("ok".to_string()),
                };
                
                // 嘗試查找服務獲取更多信息 (雖然剛保存可能還沒完全初始化)
                if let Some(service) = service_factory.find_ddns_service(&config_id).await {
                    let service_guard = service.lock().await;
                    
                    // 獲取當前IP
                    if let Ok(ip) = service_guard.get_last_or_current_ip(&config_id).await {
                        enhanced.current_ip = Some(ip);
                    }
                }
                
                enhanced_configs.push(enhanced);
            }
            
            HttpResponse::Ok().json(ConfigResponse {
                success: true,
                message: format!("成功保存 {} 個配置", req.configs.len()),
                configs: Some(enhanced_configs),
            })
        },
        Err(e) => {
            let app_error: ApplicationError = ApplicationError::DomainError(e);
            handle_application_error(app_error, "保存配置失敗")
        }
    }
}

/// 驗證配置
/// 
/// # 路由
/// 
/// - `POST /api/config/validate`
/// 
/// # 請求體
/// 
/// - 要驗證的配置
/// 
/// # 返回
/// 
/// - 配置驗證結果
#[post("/validate")]
pub async fn validate_config(
    req: web::Json<ValidateConfigRequest>
) -> impl Responder {
    info!("收到驗證配置請求: {}", req.config.record_name);
    
    // 驗證配置
    match req.config.validate() {
        Ok(_) => {
            info!("配置驗證通過: {}", req.config.record_name);
            HttpResponse::Ok().json(ValidateConfigResponse {
                success: true,
                message: "配置驗證通過".to_string(),
                is_valid: true,
            })
        },
        Err(e) => {
            // 驗證錯誤是預期的情況，不使用錯誤處理函數
            error!("配置驗證失敗: {}", e);
            HttpResponse::Ok().json(ValidateConfigResponse {
                success: false,
                message: format!("配置驗證失敗: {}", e),
                is_valid: false,
            })
        }
    }
}

/// 格式化日期時間
fn format_datetime(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
} 