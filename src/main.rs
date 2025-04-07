use cloudflare_ddns::{
    run_server, 
    ServiceFactory, 
    DdnsConfig,
    Settings,
    IpType
};
use log::{info, error, warn};
use std::env;
use std::process;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

// 用於控制 DDNS 服務的全局變量
static DDNS_CONTROL: once_cell::sync::Lazy<Arc<Mutex<Option<mpsc::Sender<()>>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

/// 顯示使用方法說明
fn help() {
    println!("Rust DDNS 更新工具");
    println!("用法: cloudflare-ddns [選項]");
    println!("");
    println!("選項:");
    println!("  --help, -h           顯示這個幫助訊息");
    println!("  --version, -v        顯示版本信息");
    println!("  --ddns               只運行 DDNS 更新服務");
    println!("  --web                只運行 Web 伺服器");
    println!("  --log=<level>        設置日誌級別 (debug, info, warn, error)");
    println!("  --port=<port>        設置 Web 伺服器端口");
    println!("  --host=<host>        設置 Web 伺服器主機地址");
    println!("  無參數                同時運行 DDNS 服務和 Web 伺服器");
}

/// 顯示版本信息
fn version() {
    let version = env!("CARGO_PKG_VERSION");
    println!("Rust DDNS 更新工具 v{}", version);
    println!("作者: zhijing");
    println!("授權: MIT");
}

// 重啟 DDNS 服務的公共函數
pub fn restart_ddns_service() {
    if let Ok(mut ctrl) = DDNS_CONTROL.lock() {
        if let Some(sender) = ctrl.take() {
            // 發送停止信號給當前服務
            let _ = sender.send(());
        }
        
        // 啟動新的 DDNS 服務
        start_ddns_service();
    }
}

/// 啟動 DDNS 服務（作為獨立進程）
fn start_ddns_service() {
    // 使用獨立進程啟動 DDNS 服務
    let mut command = process::Command::new(env::current_exe().unwrap());
    
    // 添加必要的環境變數和參數
    command.env("RUN_MODE", "ddns");
    
    // 傳遞日誌級別
    if let Ok(log_level) = env::var("RUST_LOG") {
        command.env("RUST_LOG", log_level);
    }
    
    // 添加 --ddns 參數
    command.arg("--ddns");
    
    // 啟動進程
    match command.spawn() {
        Ok(child) => {
            info!("DDNS service started in a separate process (PID: {})", child.id());
        },
        Err(e) => {
            error!("Failed to start DDNS service process: {}", e);
        }
    }
}

/// 應用程式入口點
/// 
/// # 功能
/// 
/// - 載入環境變數
/// - 初始化日誌系統
/// - 載入應用程式設置
/// - 配置並啟動 DDNS 服務
/// - 啟動 Web 伺服器
/// 
/// # 環境變數
/// 
/// - `RUST_LOG`: 日誌級別（默認：info）
/// - `CLOUDFLARE_API_TOKEN`: Cloudflare API 令牌
/// - `CLOUDFLARE_ZONE_ID`: Cloudflare 區域 ID
/// - `CLOUDFLARE_RECORD_ID`: IPv4 DNS 記錄 ID
/// - `CLOUDFLARE_RECORD_NAME`: IPv4 DNS 記錄名稱
/// - `CLOUDFLARE_RECORD_ID_V6`: IPv6 DNS 記錄 ID（可選）
/// - `CLOUDFLARE_RECORD_NAME_V6`: IPv6 DNS 記錄名稱（可選）
/// - `DDNS_UPDATE_INTERVAL`: 更新間隔（秒，默認：300）
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 載入 .env 檔案
    dotenv::dotenv().ok();
    
    // 處理命令行參數
    let args: Vec<String> = env::args().collect();
    
    // 解析命令行參數
    let mut run_ddns = false;
    let mut run_web = false;
    let mut log_level = None;
    let mut port = None;
    let mut host = None;
    
    // 檢查是否有任何參數
    if args.len() > 1 {
        // 處理選項參數
        for arg in &args[1..] {
            if arg == "--help" || arg == "-h" {
                help();
                return Ok(());
            } else if arg == "--version" || arg == "-v" {
                version();
                return Ok(());
            } else if arg == "--ddns" {
                run_ddns = true;
            } else if arg == "--web" {
                run_web = true;
            } else if arg.starts_with("--log=") {
                log_level = Some(arg.trim_start_matches("--log=").to_string());
            } else if arg.starts_with("--port=") {
                if let Ok(p) = arg.trim_start_matches("--port=").parse::<u16>() {
                    port = Some(p);
                } else {
                    println!("錯誤: 無效的端口號");
                    help();
                    return Ok(());
                }
            } else if arg.starts_with("--host=") {
                host = Some(arg.trim_start_matches("--host=").to_string());
            } else {
                println!("錯誤: 未知選項: {}", arg);
                help();
                return Ok(());
            }
        }
    }
    
    // 設置日誌級別
    if let Some(level) = log_level {
        env::set_var("RUST_LOG", level);
    } else if env::var("RUST_LOG").is_err() {
        // 如果沒有指定日誌級別且環境變數中也沒有設置，則設置為 info
        env::set_var("RUST_LOG", "info");
    }
    
    // 初始化日誌系統
    env_logger::builder().format_timestamp_millis().init();
    
    // 如果同時指定了 --ddns 和 --web，或都沒有指定，則運行兩個服務
    if (run_ddns && run_web) || (!run_ddns && !run_web) {
        // 在 Web 模式下運行
        // 先啟動 DDNS 服務作為獨立進程
        start_ddns_service();
        
        // 載入設置
        let mut settings = Settings::new().expect("Failed to load settings");
        
        // 如果命令行指定了主機和端口，則覆蓋設置
        if let Some(h) = host {
            settings.server.host = h;
        }
        if let Some(p) = port {
            settings.server.port = p;
        }
        
        // 運行 Web 伺服器
        info!("Starting Web server at {}:{}", settings.server.host, settings.server.port);
        run_server(&settings.server.host, settings.server.port).await?;
    } else if run_ddns {
        // 只運行 DDNS 服務
        return run_ddns_service().await;
    } else if run_web {
        // 只運行 Web 伺服器
        // 載入設置
        let mut settings = Settings::new().expect("Failed to load settings");
        
        // 如果命令行指定了主機和端口，則覆蓋設置
        if let Some(h) = host {
            settings.server.host = h;
        }
        if let Some(p) = port {
            settings.server.port = p;
        }
        
        info!("Starting Web server at {}:{}", settings.server.host, settings.server.port);
        return run_server(&settings.server.host, settings.server.port).await;
    }
    
    Ok(())
}

/// 運行 DDNS 服務
async fn run_ddns_service() -> std::io::Result<()> {
    info!("Starting DDNS service...");
    
    // 建立服務工廠
    let service_factory = Arc::new(ServiceFactory::new());
    
    // 初始化事件監聽系統
    service_factory.init_event_listeners().await;
    
    // 從環境變數和配置文件載入配置
    let mut configs = Vec::new();
    
    // 從環境變數載入配置
    match load_ddns_configs_from_env() {
        Ok(env_configs) => configs.extend(env_configs),
        Err(e) => {
            warn!("Failed to load DDNS configuration from environment: {}", e);
        }
    };

    // 從配置文件載入配置
    match service_factory.get_config_service().get_configs().await {
        Ok(file_configs) => configs.extend(file_configs),
        Err(e) => {
            warn!("Failed to load DDNS configuration from file: {}", e);
        }
    };

    if configs.is_empty() {
        error!("No available DDNS configurations, service exiting");
        return Ok(());
    }
    
    info!("Successfully loaded {} DDNS configurations", configs.len());
    
    // 創建信號通道用於等待終止信號
    let (tx, rx) = mpsc::channel();
    
    // 更新全局控制變量
    if let Ok(mut ctrl) = DDNS_CONTROL.lock() {
        *ctrl = Some(tx);
    }
    
    // 啟動所有配置的 DDNS 服務
    let mut service_handles = Vec::new();
    
    for ddns_config in configs {
        let ip_type = ddns_config.ip_type.clone();
        info!("Starting {} DDNS update service", ip_type);
        
        // 建立 DDNS 應用服務
        let ddns_service = service_factory.create_ddns_service(ddns_config).await;
        
        // 啟動自動更新任務
        let handle = tokio::spawn(async move {
            ddns_service.start_auto_update().await;
        });
        
        service_handles.push(handle);
    }
    
    // 等待終止信號
    let _ = rx.recv();
    info!("接收到終止信號，DDNS服務正在關閉");
    
    // 等待所有任務完成
    for handle in service_handles {
        let _ = handle.await;
    }
    
    info!("DDNS服務已關閉");
    Ok(())
}

/// 從環境變數載入 DDNS 配置
fn load_ddns_configs_from_env() -> Result<Vec<DdnsConfig>, String> {
    let mut configs = Vec::new();
    
    // 載入 IPv4 配置
    if let (Ok(api_token), Ok(zone_id), Ok(record_id), Ok(record_name)) = (
        env::var("CLOUDFLARE_API_TOKEN"),
        env::var("CLOUDFLARE_ZONE_ID"),
        env::var("CLOUDFLARE_RECORD_ID"),
        env::var("CLOUDFLARE_RECORD_NAME")
    ) {
        let update_interval = env::var("DDNS_UPDATE_INTERVAL")
            .map(|s| s.parse::<u64>().unwrap_or(300))
            .unwrap_or(300);
        
        let ipv4_config = DdnsConfig {
            api_token,
            zone_id,
            record_id,
            record_name,
            update_interval,
            ip_type: IpType::IPv4,
        };
        
        configs.push(ipv4_config);
    }
    
    // 載入 IPv6 配置
    if let (Ok(api_token), Ok(zone_id), Ok(record_id), Ok(record_name)) = (
        env::var("CLOUDFLARE_API_TOKEN"),
        env::var("CLOUDFLARE_ZONE_ID"),
        env::var("CLOUDFLARE_RECORD_ID_V6"),
        env::var("CLOUDFLARE_RECORD_NAME_V6")
    ) {
        let update_interval = env::var("DDNS_UPDATE_INTERVAL")
            .map(|s| s.parse::<u64>().unwrap_or(300))
            .unwrap_or(300);
        
        let ipv6_config = DdnsConfig {
            api_token,
            zone_id,
            record_id,
            record_name,
            update_interval,
            ip_type: IpType::IPv6,
        };
        
        configs.push(ipv6_config);
    }
    
    Ok(configs)
}