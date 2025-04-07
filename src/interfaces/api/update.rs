use actix_web::{web, HttpResponse, Responder, post};
use crate::application::ServiceFactory;
use serde::{Deserialize, Serialize};
use log::{info, error};
use std::sync::Arc;

/// 更新響應結構
#[derive(Serialize)]
pub struct UpdateResponse {
    success: bool,
    message: String,
    ip_address: Option<String>,
    domain: Option<String>,
    updated: bool,
}

/// 特定記錄更新請求
#[derive(Deserialize)]
pub struct UpdateSpecificRequest {
    domain: Option<String>,
    record_id: Option<String>,
    /// 是否等待結果
    wait_for_result: Option<bool>,
}

/// 強制更新 DNS 記錄處理器
/// 
/// # 參數
/// 
/// - `service_factory`: 服務工廠
/// 
/// # 返回
/// 
/// - `impl Responder`: 返回更新結果
#[post("/update")]
pub async fn force_update(
    service_factory: web::Data<Arc<ServiceFactory>>,
    req: Option<web::Json<UpdateSpecificRequest>>
) -> impl Responder {
    info!("收到強制更新DNS記錄請求");
    
    let wait_for_result = req.as_ref()
        .map(|r| r.wait_for_result.unwrap_or(false))
        .unwrap_or(false);
    
    // 通過事件系統觸發強制更新
    let event_manager = service_factory.get_event_manager();
    
    // 檢查是否有特定的域名或記錄ID
    if let Some(req) = req {
        if let Some(domain) = &req.domain {
            info!("強制更新特定域名: {}", domain);
            
            // 查找相關的配置和服務以獲取當前IP
            let config_service = service_factory.get_config_service();
            let mut ip_address = None;
            
            if let Ok(configs) = config_service.get_configs().await {
                for config in configs {
                    if config.record_name == *domain {
                        // 構造配置ID
                        let config_id = format!("{}-{}", config.zone_id, config.record_id);
                        if let Some(service) = service_factory.find_ddns_service(&config_id).await {
                            let service_guard = service.lock().await;
                            
                            // 獲取當前IP (無論是否需要更新)
                            if let Ok(current_ip) = service_guard.get_last_or_current_ip(&config_id).await {
                                ip_address = Some(current_ip);
                                break;
                            }
                        }
                    }
                }
            }
            
            if wait_for_result {
                let mut updated = false;
                
                if let Ok(configs) = config_service.get_configs().await {
                    for config in configs {
                        if config.record_name == *domain {
                            // 構造配置ID
                            let config_id = format!("{}-{}", config.zone_id, config.record_id);
                            if let Some(service) = service_factory.find_ddns_service(&config_id).await {
                                let service_guard = service.lock().await;
                                
                                // 直接調用服務的強制更新方法並等待結果
                                match service_guard.force_update().await {
                                    Ok((_, current_ip)) => {
                                        updated = true;
                                        ip_address = Some(current_ip);
                                        break;
                                    }
                                    Err(e) => {
                                        error!("更新域名 {} 失敗: {}", domain, e);
                                        return HttpResponse::InternalServerError().json(UpdateResponse {
                                            success: false,
                                            message: format!("更新域名 {} 失敗: {}", domain, e),
                                            ip_address,
                                            domain: Some(domain.clone()),
                                            updated: false,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                
                if updated {
                    return HttpResponse::Ok().json(UpdateResponse {
                        success: true,
                        message: format!("DNS記錄 {} 已成功更新", domain),
                        ip_address,
                        domain: Some(domain.clone()),
                        updated: true,
                    });
                } else if ip_address.is_some() {
                    // 找到域名但沒有更新 (IP未變化)
                    return HttpResponse::Ok().json(UpdateResponse {
                        success: true,
                        message: format!("DNS記錄 {} 未變化，無需更新", domain),
                        ip_address,
                        domain: Some(domain.clone()),
                        updated: false,
                    });
                } else {
                    // 找不到域名或更新失敗
                    return HttpResponse::BadRequest().json(UpdateResponse {
                        success: false,
                        message: format!("未找到域名 {} 的配置或無法獲取IP", domain),
                        ip_address: None,
                        domain: Some(domain.clone()),
                        updated: false,
                    });
                }
            } else {
                // 不等待結果，立即返回但仍提供當前IP
                event_manager.force_update_dns(Some(domain.clone())).await;
                
                return HttpResponse::Ok().json(UpdateResponse {
                    success: true,
                    message: format!("DNS記錄 {} 更新請求已發送", domain),
                    ip_address,
                    domain: Some(domain.clone()),
                    updated: false,
                });
            }
        } else if let Some(record_id) = &req.record_id {
            info!("強制更新特定記錄ID: {}", record_id);
            
            // 先查找配置獲取域名
            let config_service = service_factory.get_config_service();
            if let Ok(configs) = config_service.get_configs().await {
                for config in configs {
                    if config.record_id == *record_id {
                        let domain = config.record_name.clone();
                        let config_id = format!("{}-{}", config.zone_id, config.record_id);
                        let mut ip_address = None;
                        
                        // 獲取當前IP (無論是否需要更新)
                        if let Some(service) = service_factory.find_ddns_service(&config_id).await {
                            let service_guard = service.lock().await;
                            if let Ok(current_ip) = service_guard.get_last_or_current_ip(&config_id).await {
                                ip_address = Some(current_ip);
                            }
                        }
                        
                        if wait_for_result {
                            if let Some(service) = service_factory.find_ddns_service(&config_id).await {
                                let service_guard = service.lock().await;
                                
                                // 直接調用服務的強制更新方法並等待結果
                                match service_guard.force_update().await {
                                    Ok((_, current_ip)) => {
                                        return HttpResponse::Ok().json(UpdateResponse {
                                            success: true,
                                            message: format!("DNS記錄 {} (ID: {}) 已成功更新", domain, record_id),
                                            ip_address: Some(current_ip),
                                            domain: Some(domain),
                                            updated: true,
                                        });
                                    }
                                    Err(e) => {
                                        error!("更新記錄ID {} 失敗: {}", record_id, e);
                                        return HttpResponse::InternalServerError().json(UpdateResponse {
                                            success: false,
                                            message: format!("更新記錄ID {} 失敗: {}", record_id, e),
                                            ip_address,
                                            domain: Some(domain),
                                            updated: false,
                                        });
                                    }
                                }
                            }
                        } else {
                            // 不等待結果，立即返回但仍提供當前IP
                            event_manager.force_update_dns(Some(domain.clone())).await;
                            
                            return HttpResponse::Ok().json(UpdateResponse {
                                success: true,
                                message: format!("DNS記錄 {} (ID: {}) 更新請求已發送", domain, record_id),
                                ip_address,
                                domain: Some(domain),
                                updated: false,
                            });
                        }
                    }
                }
                
                // 未找到記錄
                return HttpResponse::BadRequest().json(UpdateResponse {
                    success: false,
                    message: format!("未找到ID為 {} 的DNS記錄", record_id),
                    ip_address: None,
                    domain: None,
                    updated: false,
                });
            }
        }
    }
    
    // 如果沒有特定記錄，更新所有記錄
    // 獲取所有配置的當前IP
    let config_service = service_factory.get_config_service();
    let mut ips_info = Vec::new();
    
    if let Ok(configs) = config_service.get_configs().await {
        for config in configs {
            let config_id = format!("{}-{}", config.zone_id, config.record_id);
            let domain = config.record_name.clone();
            
            if let Some(service) = service_factory.find_ddns_service(&config_id).await {
                let service_guard = service.lock().await;
                
                if let Ok(current_ip) = service_guard.get_last_or_current_ip(&config_id).await {
                    ips_info.push(format!("{}: {}", domain, current_ip));
                }
            }
        }
    }
    
    let current_ips_message = if ips_info.is_empty() {
        "無法獲取任何IP信息".to_string()
    } else {
        format!("當前IP信息: {}", ips_info.join(", "))
    };
    
    if wait_for_result {
        let mut results = Vec::new();
        let mut all_success = true;
        
        if let Ok(configs) = config_service.get_configs().await {
            for config in configs {
                let config_id = format!("{}-{}", config.zone_id, config.record_id);
                let domain = config.record_name.clone();
                
                if let Some(service) = service_factory.find_ddns_service(&config_id).await {
                    let service_guard = service.lock().await;
                    
                    match service_guard.force_update().await {
                        Ok((_, ip)) => {
                            results.push(format!("{}: 更新成功 ({})", domain, ip));
                        }
                        Err(e) => {
                            all_success = false;
                            results.push(format!("{}: 更新失敗 - {}", domain, e));
                        }
                    }
                }
            }
        }
        
        let result_message = if results.is_empty() {
            "沒有找到DNS記錄需要更新".to_string()
        } else {
            results.join("; ")
        };
        
        return HttpResponse::Ok().json(UpdateResponse {
            success: all_success,
            message: format!("{}. {}", result_message, current_ips_message),
            ip_address: None,
            domain: None,
            updated: all_success && !results.is_empty(),
        });
    } else {
        // 不等待結果，使用事件系統
        event_manager.force_update_dns(None).await;
        
        let response = UpdateResponse {
            success: true,
            message: format!("所有DNS記錄更新請求已發送. {}", current_ips_message),
            ip_address: None,
            domain: None,
            updated: false,
        };
        
        return HttpResponse::Ok().json(response);
    }
}

/// 更新所有 DDNS 記錄處理器
/// 
/// # 參數
/// 
/// - `service_factory`: 服務工廠
/// - `wait_for_result`: 是否等待結果 (可選)
/// 
/// # 返回
/// 
/// - `impl Responder`: 返回更新結果
#[post("/update-all")]
pub async fn update_all_records(
    service_factory: web::Data<Arc<ServiceFactory>>,
    req: Option<web::Json<UpdateSpecificRequest>>
) -> impl Responder {
    info!("收到更新所有DDNS記錄請求");
    
    let wait_for_result = req.as_ref()
        .map(|r| r.wait_for_result.unwrap_or(false))
        .unwrap_or(false);
    
    // 獲取所有配置的當前IP
    let config_service = service_factory.get_config_service();
    let mut ips_info = Vec::new();
    
    if let Ok(configs) = config_service.get_configs().await {
        for config in configs {
            let config_id = format!("{}-{}", config.zone_id, config.record_id);
            let domain = config.record_name.clone();
            
            if let Some(service) = service_factory.find_ddns_service(&config_id).await {
                let service_guard = service.lock().await;
                
                if let Ok(current_ip) = service_guard.get_last_or_current_ip(&config_id).await {
                    ips_info.push(format!("{}: {}", domain, current_ip));
                }
            }
        }
    }
    
    let current_ips_message = if ips_info.is_empty() {
        "無法獲取任何IP信息".to_string()
    } else {
        format!("當前IP信息: {}", ips_info.join(", "))
    };
    
    if wait_for_result {
        let mut results = Vec::new();
        let mut all_success = true;
        
        if let Ok(configs) = config_service.get_configs().await {
            for config in configs {
                let config_id = format!("{}-{}", config.zone_id, config.record_id);
                let domain = config.record_name.clone();
                
                if let Some(service) = service_factory.find_ddns_service(&config_id).await {
                    let service_guard = service.lock().await;
                    
                    match service_guard.force_update().await {
                        Ok((_, ip)) => {
                            results.push(format!("{}: 更新成功 ({})", domain, ip));
                        }
                        Err(e) => {
                            all_success = false;
                            results.push(format!("{}: 更新失敗 - {}", domain, e));
                        }
                    }
                }
            }
        }
        
        let result_message = if results.is_empty() {
            "沒有找到DNS記錄需要更新".to_string()
        } else {
            results.join("; ")
        };
        
        return HttpResponse::Ok().json(UpdateResponse {
            success: all_success,
            message: format!("{}. {}", result_message, current_ips_message),
            ip_address: None,
            domain: None,
            updated: all_success && !results.is_empty(),
        });
    } else {
        // 通過事件系統觸發所有記錄更新
        let event_manager = service_factory.get_event_manager();
        event_manager.force_update_all_dns().await;
        
        let response = UpdateResponse {
            success: true,
            message: format!("所有DNS記錄更新請求已發送. {}", current_ips_message),
            ip_address: None,
            domain: None,
            updated: false,
        };
        
        HttpResponse::Ok().json(response)
    }
}

/// 重啟 DDNS 服務處理器
/// 
/// # 參數
/// 
/// - `service_factory`: 服務工廠
/// 
/// # 返回
/// 
/// - `impl Responder`: 返回重啟結果
#[post("/restart")]
pub async fn restart_service(service_factory: web::Data<Arc<ServiceFactory>>) -> impl Responder {
    info!("收到重啟DDNS服務請求");
    
    // 通過事件系統觸發重啟
    let event_manager = service_factory.get_event_manager();
    event_manager.restart_ddns_service().await;
    
    let response = UpdateResponse {
        success: true,
        message: "DDNS服務重啟請求已發送".to_string(),
        ip_address: None,
        domain: None,
        updated: false,
    };
    
    HttpResponse::Ok().json(response)
} 