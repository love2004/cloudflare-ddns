use actix_web::{get, web, HttpResponse};
use crate::ServiceFactory;
use serde::{Serialize, Deserialize};
use std::time::Instant;
use super::common::{ApiResponse, ApiMetricsMiddleware, get_api_metrics, get_connection_count};

/// 健康檢查響應結構
#[derive(Serialize, Deserialize)]
struct HealthResponse {
    /// 服務名稱
    service: String,
    /// 服務狀態
    status: String,
    /// 版本信息
    version: String,
    /// 啟動時間（秒）
    uptime: u64,
    /// 系統信息
    system_info: SystemInfo,
    /// API狀態
    api_status: ApiStatus,
}

/// 系統信息結構
#[derive(Serialize, Deserialize)]
struct SystemInfo {
    /// CPU核心數
    cpu_cores: usize,
    /// 操作系統
    os_info: String,
    /// 主機名
    hostname: String,
}

/// API狀態結構
#[derive(Serialize, Deserialize)]
struct ApiStatus {
    /// 當前活動連接數
    active_connections: usize,
    /// 總API請求次數
    total_requests: usize,
    /// 總錯誤次數
    total_errors: usize,
    /// 平均響應時間（毫秒）
    avg_response_time_ms: f64,
}

/// 服務啟動時間
static mut START_TIME: Option<Instant> = None;

/// 初始化啟動時間
pub fn init_start_time() {
    unsafe {
        START_TIME = Some(Instant::now());
    }
}

/// 健康檢查端點
/// 
/// 返回服務健康狀態信息
#[get("/health")]
pub async fn health_check(service_factory: web::Data<std::sync::Arc<ServiceFactory>>) -> HttpResponse {
    // 記錄API調用開始
    let (endpoint, start_time) = ApiMetricsMiddleware::begin("health_check");
    
    // 獲取啟動時間
    let uptime = unsafe {
        match START_TIME {
            Some(start_time) => start_time.elapsed().as_secs(),
            None => {
                // 如果未初始化，則初始化
                init_start_time();
                0
            }
        }
    };
    
    // 收集API統計數據
    let metrics = get_api_metrics();
    let mut total_requests = 0;
    let mut total_errors = 0;
    let mut total_response_time = 0;
    
    for metric in metrics.values() {
        total_requests += metric.request_count;
        total_errors += metric.error_count;
        total_response_time += metric.total_response_time_ms;
    }
    
    let avg_response_time = if total_requests > 0 {
        total_response_time as f64 / total_requests as f64
    } else {
        0.0
    };
    
    // 構建響應
    let response = HealthResponse {
        service: "Cloudflare DDNS".to_string(),
        status: "operational".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime,
        system_info: SystemInfo {
            cpu_cores: num_cpus::get(),
            os_info: std::env::consts::OS.to_string(),
            hostname: hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
        },
        api_status: ApiStatus {
            active_connections: get_connection_count(),
            total_requests,
            total_errors,
            avg_response_time_ms: avg_response_time,
        },
    };
    
    // 記錄API調用結束
    ApiMetricsMiddleware::end(endpoint, start_time, true);
    
    // 返回響應
    ApiResponse::success(response, Some(start_time)).into_response()
} 