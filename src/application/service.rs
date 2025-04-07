impl ServiceFactory {
    /// 根據配置ID查找DDNS服務
    /// 
    /// # 參數
    /// 
    /// - `config_id`: 配置ID，格式為 "zone_id-record_id"
    /// 
    /// # 返回
    /// 
    /// - `Option<Arc<Mutex<DdnsApplicationService>>>`: 找到的服務或None
    pub async fn find_ddns_service(&self, config_id: &str) -> Option<Arc<Mutex<DdnsApplicationService>>> {
        let services = self.ddns_services.read().await;
        services.get(config_id).cloned()
    }
} 