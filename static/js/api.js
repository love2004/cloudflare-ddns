// 使用立即執行函數表達式(IIFE)來避免全局命名空間污染
(function() {
  // API基礎路徑
  const API_BASE = '/api';

  // API錯誤處理與重試配置
  const API_RETRY_COUNT = 3;
  const API_RETRY_DELAY = 1000; // 毫秒

  // API請求適配器，設為全局可訪問
  window.ApiClient = {
    // 連接狀態
    connectionStatus: 'unknown', // 'unknown', 'online', 'offline'
    
    // 檢測API可用性
    async checkApiAvailability() {
      try {
        const response = await fetch(`${API_BASE}/health`, { 
          method: 'GET',
          cache: 'no-store',
          headers: { 'Cache-Control': 'no-cache' }
        });
        this.connectionStatus = response.ok ? 'online' : 'offline';
        return response.ok;
      } catch (error) {
        this.connectionStatus = 'offline';
        console.error('API連接檢測失敗:', error);
        return false;
      }
    },
    
    // 通用請求函數，支持自動重試
    async request(endpoint, options = {}, retryCount = API_RETRY_COUNT) {
      try {
        const response = await fetch(`${API_BASE}${endpoint}`, options);
        
        if (!response.ok) {
          const errorText = await response.text();
          throw new Error(`API請求失敗 (${response.status}): ${errorText}`);
        }
        
        return response.json();
      } catch (error) {
        if (retryCount > 0 && !navigator.onLine) {
          console.log(`網絡連接中斷，${API_RETRY_DELAY/1000}秒後重試...`);
          await new Promise(resolve => setTimeout(resolve, API_RETRY_DELAY));
          return this.request(endpoint, options, retryCount - 1);
        }
        throw error;
      }
    },

    // 獲取所有配置
    async getConfigs() {
      return this.request('/configs');
    },
    
    // 保存配置
    async saveConfigs(configs) {
      return this.request('/configs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ configs })
      });
    },
    
    // 立即更新DDNS記錄
    async updateDdns() {
      return this.request('/update', {
        method: 'POST'
      });
    },
    
    // 獲取當前狀態
    async getStatus() {
      return this.request('/status');
    },
    
    // 驗證配置
    async validateConfig(config) {
      return this.request('/configs/validate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ config })
      });
    },
    
    // 獲取當前IPv4地址
    async getIpv4() {
      return this.request('/ip/v4');
    },
    
    // 獲取當前IPv6地址
    async getIpv6() {
      return this.request('/ip/v6');
    }
  };

  // 數據轉換適配器，設為全局可訪問
  window.DataAdapter = {
    // 轉換API配置為前端顯示格式
    adaptConfig(config) {
      return {
        record_id: config.record_id || '',
        zone_id: config.zone_id || '',
        api_token: config.api_token || '',
        record_name: config.record_name || '',
        record_type: config.ip_type === 'ipv4' ? 'A' : 'AAAA',
        ip_type: config.ip_type || 'ipv4',
        current_ip: config.current_ip || '未知',
        update_interval: config.update_interval || 300,
        last_update_time: config.last_update_time || null,
        status: 'active' // 默認狀態
      };
    },
    
    // 轉換前端表單數據為API格式
    prepareConfigForSave(formData) {
      return {
        record_id: formData.record_id,
        zone_id: formData.zone_id,
        api_token: formData.api_token,
        record_name: formData.record_name,
        ip_type: formData.record_type === 'A' ? 'ipv4' : 'ipv6',
        update_interval: parseInt(formData.update_interval) || 300
      };
    },
    
    // 處理配置列表
    adaptConfigList(response) {
      if (!response || !response.configs) {
        return [];
      }
      return response.configs.map(this.adaptConfig);
    }
  };
})(); 