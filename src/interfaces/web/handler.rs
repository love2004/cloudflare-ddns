use actix_web::{web, get, Error, HttpResponse, dev::HttpServiceFactory};
use actix_files::NamedFile;
use actix_files::Files;
use std::path::PathBuf;
use log::{info, error};
use std::sync::Once;
use crate::constants::{STATIC_INDEX_FILE, STATIC_DIR, STATIC_CSS_DIR, STATIC_JS_DIR};

/// 首頁處理器
/// 
/// # 返回
/// 
/// - `impl Responder`: 返回首頁 HTML 文件
#[get("/")]
pub async fn index() -> Result<NamedFile, Error> {
    info!("接收到首頁請求");
    
    let path: PathBuf = PathBuf::from(STATIC_INDEX_FILE);
    match NamedFile::open(path) {
        Ok(file) => {
            info!("成功返回首頁文件");
            
            // 設置適當的標頭
            Ok(file
                // 禁用緩存
                .disable_content_disposition()
                .set_content_type(mime::TEXT_HTML_UTF_8))
        },
        Err(e) => {
            error!("無法打開首頁文件: {}", e);
            Err(Error::from(e))
        }
    }
}

/// 處理靜態資源
/// 
/// # 參數
/// 
/// - `path`: 資源路徑
/// 
/// # 返回
/// 
/// - `impl Responder`: 返回靜態資源文件
#[get("/{filename:.*}")]
pub async fn static_files(path: web::Path<String>) -> Result<NamedFile, Error> {
    let filename = path.into_inner();
    info!("接收到靜態文件請求: {}", filename);
    
    let path = PathBuf::from(format!("{}/{}", STATIC_DIR, filename));
    
    match NamedFile::open(path) {
        Ok(file) => {
            info!("成功返回靜態文件: {}", filename);
            
            // 根據文件類型配置回應
            let file = file.disable_content_disposition();
            
            // 設置不同類型文件的內容類型
            let file = if filename.ends_with(".js") {
                file.set_content_type(mime::APPLICATION_JAVASCRIPT)
            } else if filename.ends_with(".css") {
                file.set_content_type(mime::TEXT_CSS)
            } else if filename.ends_with(".png") {
                file.set_content_type(mime::IMAGE_PNG)
            } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
                file.set_content_type(mime::IMAGE_JPEG)
            } else if filename.ends_with(".svg") {
                file.set_content_type(mime::IMAGE_SVG)
            } else {
                file
            };
            
            Ok(file)
        },
        Err(e) => {
            error!("無法打開靜態文件 {}: {}", filename, e);
            Err(Error::from(e))
        }
    }
}

/// 配置 Web UI 路由
/// 
/// # 參數
/// 
/// - `cfg`: 服務配置
/// 
/// # 功能
/// 
/// - 註冊 Web UI 路由
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // 使用一個靜態變數確保只輸出一次日誌
    static LOGGED: Once = Once::new();
    LOGGED.call_once(|| {
        info!("配置Web UI路由");
    });
    
    // 啟用壓縮的靜態文件服務
    let files_css = Files::new("/css", STATIC_CSS_DIR)
        .prefer_utf8(true)
        .use_etag(true)
        .use_last_modified(true);
        
    let files_js = Files::new("/js", STATIC_JS_DIR)
        .prefer_utf8(true)
        .use_etag(true)
        .use_last_modified(true);
    
    cfg.service(index)
        .service(files_css)
        .service(files_js);
} 