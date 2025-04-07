use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use std::collections::HashMap;
use std::fmt;
use log::{info, warn};
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};

/// 事件類型枚舉
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    /// 重啟 DDNS 服務
    RestartDdnsService,
    /// 強制更新 DNS 記錄
    ForceUpdateDns,
    /// 配置改變
    ConfigChanged,
    /// 強制更新所有DNS記錄
    ForceUpdateAll,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::RestartDdnsService => write!(f, "RestartDdnsService"),
            EventType::ForceUpdateDns => write!(f, "ForceUpdateDns"),
            EventType::ConfigChanged => write!(f, "ConfigChanged"),
            EventType::ForceUpdateAll => write!(f, "ForceUpdateAll"),
        }
    }
}

/// 事件數據
#[derive(Debug, Clone)]
pub struct EventData {
    /// 事件類型
    pub event_type: EventType,
    /// 附加數據（可選）
    pub data: Option<String>,
}

/// 事件監聽器特性
#[async_trait::async_trait]
pub trait EventListener: Send + Sync {
    /// 處理事件
    ///
    /// # 參數
    ///
    /// - `event`: 事件數據
    async fn handle_event(&self, event: EventData);
}

/// 事件管理器
pub struct EventManager {
    /// 事件發送器
    senders: RwLock<HashMap<EventType, Sender<EventData>>>,
    /// 事件監聽器
    listeners: RwLock<Vec<Arc<Mutex<dyn EventListener>>>>,
}

impl EventManager {
    /// 創建新的事件管理器
    pub fn new() -> Self {
        Self {
            senders: RwLock::new(HashMap::new()),
            listeners: RwLock::new(Vec::new()),
        }
    }
    
    /// 註冊事件監聽器
    ///
    /// # 參數
    ///
    /// - `listener`: 事件監聽器
    pub async fn register_listener(&self, listener: Arc<Mutex<dyn EventListener>>) {
        let mut listeners = self.listeners.write().await;
        listeners.push(listener);
        info!("註冊了新的事件監聽器，當前監聽器數量: {}", listeners.len());
    }
    
    /// 訂閱事件
    ///
    /// # 參數
    ///
    /// - `event_type`: 事件類型
    ///
    /// # 返回
    ///
    /// - 事件接收器
    pub async fn subscribe(&self, event_type: EventType) -> Option<Receiver<EventData>> {
        // 嘗試從現有發送器中獲取
        {
            let senders = self.senders.read().await;
            if let Some(sender) = senders.get(&event_type) {
                return Some(sender.subscribe());
            }
        }
        
        // 如果沒有找到發送器，創建一個新的
        let mut senders = self.senders.write().await;
        let (sender, receiver) = broadcast::channel(16);
        senders.insert(event_type.clone(), sender);
        info!("創建了新的事件發送器: {}", event_type);
        Some(receiver)
    }
    
    /// 發布事件
    ///
    /// # 參數
    ///
    /// - `event`: 事件數據
    pub async fn publish(&self, event: EventData) {
        info!("發布事件: {}", event.event_type);
        
        // 1. 通過發送器發送事件
        {
            let senders = self.senders.read().await;
            if let Some(sender) = senders.get(&event.event_type) {
                if let Err(e) = sender.send(event.clone()) {
                    warn!("發送事件 {} 失敗: {}", event.event_type, e);
                }
            }
        }
        
        // 2. 通知所有註冊的監聽器
        let listeners = self.listeners.read().await;
        for listener in listeners.iter() {
            let listener_guard = listener.lock().await;
            listener_guard.handle_event(event.clone()).await;
        }
    }
    
    /// 發布重啟 DDNS 服務事件
    pub async fn restart_ddns_service(&self) {
        let event = EventData {
            event_type: EventType::RestartDdnsService,
            data: None,
        };
        self.publish(event).await;
    }
    
    /// 發布強制更新 DNS 記錄事件
    ///
    /// # 參數
    ///
    /// - `record_name`: 記錄名稱（可選）
    pub async fn force_update_dns(&self, record_name: Option<String>) {
        let event = EventData {
            event_type: EventType::ForceUpdateDns,
            data: record_name,
        };
        self.publish(event).await;
    }
    
    /// 強制更新所有DNS記錄
    pub async fn force_update_all_dns(&self) {
        info!("觸發事件: 強制更新所有DNS記錄");
        
        let event = EventData {
            event_type: EventType::ForceUpdateAll,
            data: None,
        };
        
        self.publish(event).await;
    }
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
} 