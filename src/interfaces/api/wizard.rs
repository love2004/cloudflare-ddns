use actix_web::{web, post, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use log::{info, error};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

/// CloudFlare API令牌驗證請求
#[derive(Deserialize)]
pub struct TokenValidateRequest {
    token: String,
}

/// 區域響應
#[derive(Serialize)]
pub struct ZoneItem {
    id: String,
    name: String,
}

/// 記錄響應
#[derive(Serialize)]
pub struct DnsRecordItem {
    id: String,
    name: String,
    #[serde(rename = "type")]
    record_type: String,
    content: String,
}

/// CloudFlare API令牌驗證響應
#[derive(Serialize)]
pub struct TokenValidateResponse {
    success: bool,
    message: String,
    zones: Option<Vec<ZoneItem>>,
}

/// DNS記錄查詢請求
#[derive(Deserialize)]
pub struct DnsRecordsRequest {
    token: String,
    zone_id: String,
}

/// DNS記錄查詢響應
#[derive(Serialize)]
pub struct DnsRecordsResponse {
    success: bool,
    message: String,
    records: Option<Vec<DnsRecordItem>>,
}

/// CloudFlare API通用響應
#[derive(Deserialize)]
struct CloudflareResponse<T> {
    success: bool,
    #[serde(default)]
    errors: Vec<serde_json::Value>,
    result: Option<T>,
    #[serde(default)]
    result_info: Option<serde_json::Value>,
}

/// 驗證CloudFlare API令牌
/// 
/// # 路由
/// 
/// - `POST /api/wizard/validate-token`
/// 
/// # 請求體
/// 
/// - 包含CloudFlare API令牌
/// 
/// # 返回
/// 
/// - 令牌驗證結果和區域列表
#[post("/validate-token")]
pub async fn validate_token(
    req: web::Json<TokenValidateRequest>,
) -> impl Responder {
    info!("收到驗證CloudFlare API令牌請求");
    
    let token = &req.token;
    
    // 創建HTTP客戶端
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build() {
        Ok(client) => client,
        Err(e) => {
            error!("創建HTTP客戶端失敗: {}", e);
            return HttpResponse::InternalServerError().json(TokenValidateResponse {
                success: false,
                message: format!("創建HTTP客戶端失敗: {}", e),
                zones: None,
            });
        }
    };
    
    // 設置請求頭
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    
    match HeaderValue::from_str(&format!("Bearer {}", token)) {
        Ok(auth_value) => {
            headers.insert(AUTHORIZATION, auth_value);
        },
        Err(e) => {
            error!("創建授權頭失敗: {}", e);
            return HttpResponse::BadRequest().json(TokenValidateResponse {
                success: false,
                message: format!("無效的API令牌: {}", e),
                zones: None,
            });
        }
    }
    
    // 調用CloudFlare API獲取區域列表
    let zones_url = "https://api.cloudflare.com/client/v4/zones?per_page=50";
    
    let response = match client.get(zones_url)
        .headers(headers)
        .send()
        .await {
        Ok(res) => res,
        Err(e) => {
            error!("請求CloudFlare API失敗: {}", e);
            return HttpResponse::ServiceUnavailable().json(TokenValidateResponse {
                success: false,
                message: format!("無法連接至CloudFlare API: {}", e),
                zones: None,
            });
        }
    };
    
    // 檢查HTTP狀態
    if !response.status().is_success() {
        let status = response.status();
        let error_text = match response.text().await {
            Ok(text) => text,
            Err(_) => "無法讀取錯誤詳情".to_string(),
        };
        
        error!("CloudFlare API返回錯誤狀態碼: {}, 內容: {}", status, error_text);
        
        return HttpResponse::BadRequest().json(TokenValidateResponse {
            success: false,
            message: format!("API令牌驗證失敗: HTTP {}", status),
            zones: None,
        });
    }
    
    // 解析返回結果
    match response.json::<CloudflareResponse<Vec<serde_json::Value>>>().await {
        Ok(cf_response) => {
            if !cf_response.success {
                let error_msg = if !cf_response.errors.is_empty() {
                    format!("CloudFlare API錯誤: {:?}", cf_response.errors)
                } else {
                    "未知CloudFlare API錯誤".to_string()
                };
                
                error!("{}", error_msg);
                
                return HttpResponse::BadRequest().json(TokenValidateResponse {
                    success: false,
                    message: format!("API令牌驗證失敗: {}", error_msg),
                    zones: None,
                });
            }
            
            // 提取區域信息
            if let Some(zones_data) = cf_response.result {
                let mut zones = Vec::new();
                
                for zone in zones_data {
                    if let (Some(id), Some(name)) = (
                        zone.get("id").and_then(|v| v.as_str()),
                        zone.get("name").and_then(|v| v.as_str())
                    ) {
                        zones.push(ZoneItem {
                            id: id.to_string(),
                            name: name.to_string(),
                        });
                    }
                }
                
                info!("API令牌驗證成功，找到{}個區域", zones.len());
                
                return HttpResponse::Ok().json(TokenValidateResponse {
                    success: true,
                    message: format!("API令牌驗證通過，找到{}個區域", zones.len()),
                    zones: Some(zones),
                });
            } else {
                info!("API令牌驗證成功，但未找到區域");
                
                return HttpResponse::Ok().json(TokenValidateResponse {
                    success: true,
                    message: "API令牌驗證通過，但未找到區域".to_string(),
                    zones: Some(Vec::new()),
                });
            }
        },
        Err(e) => {
            error!("解析CloudFlare API響應失敗: {}", e);
            
            return HttpResponse::InternalServerError().json(TokenValidateResponse {
                success: false,
                message: format!("解析API響應失敗: {}", e),
                zones: None,
            });
        }
    }
}

/// 獲取DNS記錄
/// 
/// # 路由
/// 
/// - `POST /api/wizard/get-records`
/// 
/// # 請求體
/// 
/// - 包含CloudFlare API令牌和區域ID
/// 
/// # 返回
/// 
/// - DNS記錄列表
#[post("/get-records")]
pub async fn get_dns_records(
    req: web::Json<DnsRecordsRequest>,
) -> impl Responder {
    info!("收到獲取DNS記錄請求，區域ID: {}", req.zone_id);
    
    let token = &req.token;
    let zone_id = &req.zone_id;
    
    // 創建HTTP客戶端
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build() {
        Ok(client) => client,
        Err(e) => {
            error!("創建HTTP客戶端失敗: {}", e);
            return HttpResponse::InternalServerError().json(DnsRecordsResponse {
                success: false,
                message: format!("創建HTTP客戶端失敗: {}", e),
                records: None,
            });
        }
    };
    
    // 設置請求頭
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    
    match HeaderValue::from_str(&format!("Bearer {}", token)) {
        Ok(auth_value) => {
            headers.insert(AUTHORIZATION, auth_value);
        },
        Err(e) => {
            error!("創建授權頭失敗: {}", e);
            return HttpResponse::BadRequest().json(DnsRecordsResponse {
                success: false,
                message: format!("無效的API令牌: {}", e),
                records: None,
            });
        }
    }
    
    // 調用CloudFlare API獲取DNS記錄
    let records_url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records?per_page=100&type=A,AAAA",
        zone_id
    );
    
    let response = match client.get(&records_url)
        .headers(headers)
        .send()
        .await {
        Ok(res) => res,
        Err(e) => {
            error!("請求CloudFlare API失敗: {}", e);
            return HttpResponse::ServiceUnavailable().json(DnsRecordsResponse {
                success: false,
                message: format!("無法連接至CloudFlare API: {}", e),
                records: None,
            });
        }
    };
    
    // 檢查HTTP狀態
    if !response.status().is_success() {
        let status = response.status();
        let error_text = match response.text().await {
            Ok(text) => text,
            Err(_) => "無法讀取錯誤詳情".to_string(),
        };
        
        error!("CloudFlare API返回錯誤狀態碼: {}, 內容: {}", status, error_text);
        
        return HttpResponse::BadRequest().json(DnsRecordsResponse {
            success: false,
            message: format!("獲取DNS記錄失敗: HTTP {}", status),
            records: None,
        });
    }
    
    // 解析返回結果
    match response.json::<CloudflareResponse<Vec<serde_json::Value>>>().await {
        Ok(cf_response) => {
            if !cf_response.success {
                let error_msg = if !cf_response.errors.is_empty() {
                    format!("CloudFlare API錯誤: {:?}", cf_response.errors)
                } else {
                    "未知CloudFlare API錯誤".to_string()
                };
                
                error!("{}", error_msg);
                
                return HttpResponse::BadRequest().json(DnsRecordsResponse {
                    success: false,
                    message: format!("獲取DNS記錄失敗: {}", error_msg),
                    records: None,
                });
            }
            
            // 提取記錄信息
            if let Some(records_data) = cf_response.result {
                let mut records = Vec::new();
                
                for record in records_data {
                    if let (Some(id), Some(name), Some(record_type), Some(content)) = (
                        record.get("id").and_then(|v| v.as_str()),
                        record.get("name").and_then(|v| v.as_str()),
                        record.get("type").and_then(|v| v.as_str()),
                        record.get("content").and_then(|v| v.as_str())
                    ) {
                        records.push(DnsRecordItem {
                            id: id.to_string(),
                            name: name.to_string(),
                            record_type: record_type.to_string(),
                            content: content.to_string(),
                        });
                    }
                }
                
                info!("成功獲取DNS記錄，找到{}條記錄", records.len());
                
                return HttpResponse::Ok().json(DnsRecordsResponse {
                    success: true,
                    message: format!("成功獲取DNS記錄，找到{}條記錄", records.len()),
                    records: Some(records),
                });
            } else {
                info!("成功獲取DNS記錄，但未找到任何記錄");
                
                return HttpResponse::Ok().json(DnsRecordsResponse {
                    success: true,
                    message: "成功獲取DNS記錄，但未找到任何記錄".to_string(),
                    records: Some(Vec::new()),
                });
            }
        },
        Err(e) => {
            error!("解析CloudFlare API響應失敗: {}", e);
            
            return HttpResponse::InternalServerError().json(DnsRecordsResponse {
                success: false,
                message: format!("解析API響應失敗: {}", e),
                records: None,
            });
        }
    }
} 