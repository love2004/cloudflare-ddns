pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod interfaces;
pub mod constants;

// 重新導出常用類型和結構
pub use application::ServiceFactory;
pub use domain::config::{DdnsConfig, IpType, Settings};
pub use domain::error::DomainError;
pub use application::error::ApplicationError;
pub use application::events::{EventManager, EventType, EventData};
pub use constants::*;

// 重新導出服務啟動函數
pub use actix_web::{web, App, HttpServer};
pub use actix_cors::Cors;
use std::path::Path;
use std::sync::Arc;
use log::info;

/// 啟動 Web 伺服器
/// 
/// # 參數
/// 
/// - `host`: 伺服器監聽的主機地址
/// - `port`: 伺服器監聽的端口
/// 
/// # 返回
/// 
/// - `io::Result<()>`: 成功時返回 `()`，失敗時返回錯誤
/// 
/// # 功能
/// 
/// - 配置並啟動 HTTP 伺服器
/// - 設置日誌中間件
/// - 配置 API 路由
pub async fn run_server(host: &str, port: u16) -> std::io::Result<()> {
    let address = format!("{}:{}", host, port);
    info!("準備在 {} 上啟動 Web 伺服器", address);
    
    // 確保靜態文件目錄存在
    let static_dir = Path::new(STATIC_DIR);
    if !static_dir.exists() {
        std::fs::create_dir_all(static_dir)?;
        info!("Created static directory");
    }
    
    // 確保index.html存在
    let index_path = static_dir.join("index.html");
    if !index_path.exists() {
        info!("Static files not found, web UI may not work correctly");
    } else {
        info!("Found static files, web UI should be available");
    }
    
    // 創建服務工廠
    let service_factory = ServiceFactory::new();
    let service_factory_arc = Arc::new(service_factory);
    
    // 初始化事件監聽系統
    service_factory_arc.init_event_listeners().await;
    info!("事件系統已初始化");
    
    // 包裝為web::Data
    let service_factory_data = web::Data::new(service_factory_arc);
    
    HttpServer::new(move || {
        // 啟用 CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        
        App::new()
            .wrap(cors)
            // 注冊服務工廠
            .app_data(service_factory_data.clone())
            // 配置路由
            .configure(interfaces::api::configure_routes)
            .configure(interfaces::web::configure_routes)
    })
    .bind(address)?
    .run()
    .await
}