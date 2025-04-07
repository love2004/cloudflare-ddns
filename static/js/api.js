// API基礎路徑
const API_BASE = '/api';

// API請求適配器
const ApiClient = {
  // 獲取所有配置
  async getConfigs() {
    const response = await fetch(`${API_BASE}/configs`);
    if (!response.ok) throw new Error('獲取配置失敗');
    return response.json();
  },
  
  // 保存配置
  async saveConfigs(configs) {
    const response = await fetch(`${API_BASE}/configs`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ configs })
    });
    if (!response.ok) throw new Error('保存配置失敗');
    return response.json();
  },
  
  // 立即更新DDNS記錄
  async updateDdns() {
    const response = await fetch(`${API_BASE}/update`, {
      method: 'POST'
    });
    if (!response.ok) throw new Error('DDNS更新失敗');
    return response.json();
  },
  
  // 獲取當前狀態
  async getStatus() {
    const response = await fetch(`${API_BASE}/status`);
    if (!response.ok) throw new Error('獲取狀態失敗');
    return response.json();
  },
  
  // 驗證配置
  async validateConfig(config) {
    const response = await fetch(`${API_BASE}/configs/validate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ config })
    });
    if (!response.ok) throw new Error('配置驗證失敗');
    return response.json();
  },
  
  // 獲取當前IPv4地址
  async getIpv4() {
    const response = await fetch(`${API_BASE}/ip/v4`);
    if (!response.ok) throw new Error('獲取IPv4地址失敗');
    return response.json();
  },
  
  // 獲取當前IPv6地址
  async getIpv6() {
    const response = await fetch(`${API_BASE}/ip/v6`);
    if (!response.ok) throw new Error('獲取IPv6地址失敗');
    return response.json();
  }
};

// 數據轉換適配器
const DataAdapter = {
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