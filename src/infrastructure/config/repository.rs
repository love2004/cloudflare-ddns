use crate::domain::config::DdnsConfig;
use crate::domain::error::DomainError;
use crate::constants::CONFIG_FILE_PATH;
use log::{info, error, warn};
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{RwLock, watch};
use tokio::time::{sleep, Duration};
use tokio::task;
use serde_json;
use serde_yaml;

/// 文件配置存儲庫
pub struct FileConfigRepository {
    /// 配置文件路徑
    config_path: PathBuf,
    /// 配置變更發送器
    change_sender: watch::Sender<()>,
    /// 配置變更接收器
    change_receiver: watch::Receiver<()>,
    /// 最後修改時間
    last_modified: Arc<RwLock<Option<std::time::SystemTime>>>,
    /// 配置文件監視任務是否運行
    is_watching: Arc<RwLock<bool>>,
}

impl FileConfigRepository {
    /// 創建新的文件配置存儲庫
    pub fn new() -> Self {
        let config_path = PathBuf::from(CONFIG_FILE_PATH);
        // 確保配置目錄存在
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                match fs::create_dir_all(parent) {
                    Ok(_) => info!("Created config directory: {:?}", parent),
                    Err(e) => error!("Failed to create config directory: {}", e),
                }
            }
        }
        
        let (change_sender, change_receiver) = watch::channel(());
        
        Self {
            config_path,
            change_sender,
            change_receiver,
            last_modified: Arc::new(RwLock::new(None)),
            is_watching: Arc::new(RwLock::new(false)),
        }
    }
    
    /// 從文件加載 DDNS 配置
    pub async fn load_configs(&self) -> Result<Vec<DdnsConfig>, DomainError> {
        if !self.config_path.exists() {
            return Ok(Vec::new());
        }
        
        // 讀取配置文件
        let contents = match fs::read_to_string(&self.config_path) {
            Ok(contents) => contents,
            Err(e) => return Err(DomainError::config(format!("Failed to read config file: {}", e))),
        };
        
        // 解析 JSON
        let configs: Result<Vec<DdnsConfig>, _> = serde_json::from_str(&contents);
        match configs {
            Ok(configs) => {
                // 更新最後修改時間
                if let Ok(metadata) = fs::metadata(&self.config_path) {
                    if let Ok(modified) = metadata.modified() {
                        let mut last_modified = self.last_modified.write().await;
                        *last_modified = Some(modified);
                    }
                }
                
                Ok(configs)
            },
            Err(e) => {
                // 嘗試解析為單個配置
                let config: Result<DdnsConfig, _> = serde_json::from_str(&contents);
                match config {
                    Ok(config) => {
                        // 更新最後修改時間
                        if let Ok(metadata) = fs::metadata(&self.config_path) {
                            if let Ok(modified) = metadata.modified() {
                                let mut last_modified = self.last_modified.write().await;
                                *last_modified = Some(modified);
                            }
                        }
                        
                        Ok(vec![config])
                    },
                    Err(_) => Err(DomainError::config(format!("Failed to parse config file: {}", e))),
                }
            }
        }
    }
    
    /// 保存 DDNS 配置到文件
    pub async fn save_configs(&self, configs: &[DdnsConfig]) -> Result<(), DomainError> {
        // 將配置序列化為 JSON
        let json = match serde_json::to_string_pretty(configs) {
            Ok(json) => json,
            Err(e) => return Err(DomainError::config(format!("Failed to serialize configs: {}", e))),
        };
        
        // 寫入文件
        let mut file = match File::create(&self.config_path) {
            Ok(file) => file,
            Err(e) => return Err(DomainError::config(format!("Failed to create config file: {}", e))),
        };
        
        if let Err(e) = file.write_all(json.as_bytes()) {
            return Err(DomainError::config(format!("Failed to write config file: {}", e)));
        }
        
        // 更新最後修改時間
        if let Ok(metadata) = fs::metadata(&self.config_path) {
            if let Ok(modified) = metadata.modified() {
                let mut last_modified = self.last_modified.write().await;
                *last_modified = Some(modified);
            }
        }
        
        // 通知配置變更
        let _ = self.change_sender.send(());
        
        Ok(())
    }
    
    /// 獲取配置變更接收器
    pub fn get_change_receiver(&self) -> watch::Receiver<()> {
        self.change_receiver.clone()
    }
    
    /// 開始監視配置文件變更
    pub async fn start_watching(&self) -> Result<(), DomainError> {
        // 確保只有一個監視任務
        let mut is_watching = self.is_watching.write().await;
        if *is_watching {
            return Ok(());
        }
        
        *is_watching = true;
        
        let config_path = self.config_path.clone();
        let last_modified = self.last_modified.clone();
        let change_sender = self.change_sender.clone();
        let is_watching_clone = self.is_watching.clone();
        
        // 啟動監視任務
        task::spawn(async move {
            info!("Starting config file watcher for: {:?}", config_path);
            
            let check_interval = Duration::from_secs(5);
            
            while {
                let is_watching = is_watching_clone.read().await;
                *is_watching
            } {
                // 檢查文件是否存在
                if !config_path.exists() {
                    sleep(check_interval).await;
                    continue;
                }
                
                // 檢查文件是否被修改
                match fs::metadata(&config_path) {
                    Ok(metadata) => {
                        match metadata.modified() {
                            Ok(current_modified) => {
                                let should_notify = {
                                    let last_mod = last_modified.read().await;
                                    match *last_mod {
                                        Some(last_time) => current_modified > last_time,
                                        None => true,
                                    }
                                };
                                
                                if should_notify {
                                    info!("Detected config file change, notifying listeners");
                                    
                                    // 更新最後修改時間
                                    {
                                        let mut last_mod = last_modified.write().await;
                                        *last_mod = Some(current_modified);
                                    }
                                    
                                    // 通知變更
                                    let _ = change_sender.send(());
                                }
                            },
                            Err(e) => warn!("Failed to get file modified time: {}", e),
                        }
                    },
                    Err(e) => warn!("Failed to get file metadata: {}", e),
                }
                
                sleep(check_interval).await;
            }
            
            info!("Config file watcher stopped");
        });
        
        Ok(())
    }
    
    /// 停止監視配置文件變更
    pub async fn stop_watching(&self) {
        let mut is_watching = self.is_watching.write().await;
        *is_watching = false;
        info!("Config file watcher will stop on next check");
    }

    /// 從指定路徑讀取配置文件
    pub fn read(&self, path: &PathBuf) -> Result<DdnsConfig, DomainError> {
        if !path.exists() {
            return Err(DomainError::config(format!("Config file not found: {}", path.display())));
        }
        
        let content = std::fs::read_to_string(path)
            .map_err(|e| DomainError::config(format!("Failed to read config file: {}", e)))?;
        
        self.parse(&content)
    }
    
    /// 解析配置文件內容
    pub fn parse(&self, content: &str) -> Result<DdnsConfig, DomainError> {
        // 嘗試解析為 YAML
        if let Ok(config) = serde_yaml::from_str::<DdnsConfig>(content) {
            return Ok(config);
        }
        
        // 嘗試解析為 JSON
        if let Ok(config) = serde_json::from_str::<DdnsConfig>(content) {
            return Ok(config);
        }
        
        // 如果都解析失敗
        Err(DomainError::config("Failed to parse config file, not valid JSON or YAML format".to_string()))
    }
}

impl Default for FileConfigRepository {
    fn default() -> Self {
        Self::new()
    }
} 