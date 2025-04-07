use crate::domain::error::DomainError;
use crate::domain::state::{StateRepository, StateEntry};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 內存狀態存儲庫
pub struct InMemoryStateRepository {
    state: Arc<RwLock<HashMap<String, StateEntry>>>,
}

impl Default for InMemoryStateRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryStateRepository {
    /// 創建新的內存狀態存儲庫
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl StateRepository for InMemoryStateRepository {
    async fn get_last_ip(&self, config_id: &str) -> Result<Option<String>, DomainError> {
        match self.state.read() {
            Ok(state) => {
                if let Some(entry) = state.get(config_id) {
                    Ok(entry.last_ip.clone())
                } else {
                    Ok(None)
                }
            },
            Err(_) => Err(DomainError::LogicError("Failed to read state".to_string())),
        }
    }
    
    async fn set_last_ip(&self, config_id: &str, ip: &str) -> Result<(), DomainError> {
        match self.state.write() {
            Ok(mut state) => {
                let entry = state.entry(config_id.to_string())
                    .or_insert(StateEntry {
                        last_ip: None,
                        last_update_time: None,
                    });
                
                entry.last_ip = Some(ip.to_string());
                Ok(())
            },
            Err(_) => Err(DomainError::LogicError("Failed to write state".to_string())),
        }
    }
    
    async fn get_last_update_time(&self, config_id: &str) -> Result<Option<DateTime<Utc>>, DomainError> {
        match self.state.read() {
            Ok(state) => {
                if let Some(entry) = state.get(config_id) {
                    Ok(entry.last_update_time)
                } else {
                    Ok(None)
                }
            },
            Err(_) => Err(DomainError::LogicError("Failed to read state".to_string())),
        }
    }
    
    async fn set_last_update_time(&self, config_id: &str, time: DateTime<Utc>) -> Result<(), DomainError> {
        match self.state.write() {
            Ok(mut state) => {
                let entry = state.entry(config_id.to_string())
                    .or_insert(StateEntry {
                        last_ip: None,
                        last_update_time: None,
                    });
                
                entry.last_update_time = Some(time);
                Ok(())
            },
            Err(_) => Err(DomainError::LogicError("Failed to write state".to_string())),
        }
    }
    
    async fn get_state(&self, config_id: &str) -> Result<Option<StateEntry>, DomainError> {
        match self.state.read() {
            Ok(state) => {
                if let Some(entry) = state.get(config_id) {
                    Ok(Some(entry.clone()))
                } else {
                    Ok(None)
                }
            },
            Err(_) => Err(DomainError::LogicError("Failed to read state".to_string())),
        }
    }
    
    async fn set_state(&self, config_id: &str, state_entry: StateEntry) -> Result<(), DomainError> {
        match self.state.write() {
            Ok(mut state) => {
                state.insert(config_id.to_string(), state_entry);
                Ok(())
            },
            Err(_) => Err(DomainError::LogicError("Failed to write state".to_string())),
        }
    }
} 