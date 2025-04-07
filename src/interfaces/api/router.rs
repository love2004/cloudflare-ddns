use actix_web::web;
use super::ip::{get_ipv4, get_ipv6};
use super::health::health_check;
use super::status::get_status;
use super::update::{force_update, restart_service, update_all_records};
use super::config::{get_configs, save_configs, validate_config};
use super::wizard::{validate_token, get_dns_records};
use log::info;

/// 配置 API 路由
/// 
/// # 參數
/// 
/// - `cfg`: 服務配置
/// 
/// # 功能
/// 
/// - 註冊 API 路由
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // 使用一個靜態變數確保只輸出一次日誌
    static LOGGED: std::sync::Once = std::sync::Once::new();
    LOGGED.call_once(|| {
        info!("註冊API路由: /api/ip, /api/health, /api/status, /api/update, /api/update-all, /api/configs, /api/wizard");
    });
    
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/ip")
                    .service(get_ipv4)
                    .service(get_ipv6)
            )
            .service(health_check)
            .service(get_status)
            .service(force_update)
            .service(update_all_records)
            .service(restart_service)
            .service(
                web::scope("/configs")
                    .service(get_configs)
                    .service(save_configs)
                    .service(validate_config)
            )
            .service(
                web::scope("/wizard")
                    .service(validate_token)
                    .service(get_dns_records)
            )
    );
} 