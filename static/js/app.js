/**
 * 主應用模塊
 * 處理事件綁定和初始化
 */

// API 基礎路徑
const API_BASE = '/api';  // 恢復 API 前綴

// 全局變量
let currentConfigs = [];

// 在 DOM 載入完成後執行
document.addEventListener("DOMContentLoaded", () => {
    // 初始化應用
    initApp();
});

// 初始化應用
function initApp() {
    // 初始化加載和通知組件
    LoadingManager.init();
    NotificationManager.init();
    
    // 綁定全局未處理的Promise錯誤
    window.addEventListener('unhandledrejection', event => {
        console.error('未處理的Promise拒絕:', event.reason);
        NotificationManager.showError(event.reason.message || '發生未知錯誤');
    });
            
    // 綁定導航事件
    initNavigation();
            
    // 綁定向導事件
    bindWizardEvents();
    
    // 綁定模態框事件
    bindModalEvents();
    
    // 載入初始數據
    loadInitialData();
}

// 初始化導航功能
function initNavigation() {
    const navLinks = document.querySelectorAll('.sidebar-nav a');
    const sections = {
        'dashboard': document.getElementById('dashboard-section'),
        'domains': document.getElementById('domains-section'),
        'settings': document.getElementById('settings-section'),
        'logs': document.getElementById('logs-section')
    };

    navLinks.forEach(link => {
        link.addEventListener('click', (e) => {
            e.preventDefault();
            const target = link.getAttribute('href').substring(1);
            
            // 更新活動狀態
            navLinks.forEach(l => l.parentElement.classList.remove('active'));
            link.parentElement.classList.add('active');
            
            // 顯示對應的內容區域
            Object.values(sections).forEach(section => {
                if (section) section.style.display = 'none';
            });
            
            if (sections[target]) {
                sections[target].style.display = 'block';
                // 載入對應模塊的數據
                loadModuleData(target);
            }
        });
    });
}

// 載入初始數據
async function loadInitialData() {
    try {
        LoadingManager.show('正在載入數據...');
        
        // 載入配置
        await loadConfigs();
        
        // 載入儀表板數據
        await loadDashboardData();
        
        // 檢查是否需要顯示設定向導
        await checkFirstVisit();
        
        LoadingManager.hide();
    } catch (error) {
        console.error('載入初始數據失敗:', error);
        NotificationManager.showError('載入數據失敗: ' + error.message);
        LoadingManager.hide();
    }
}

// 載入模塊數據
async function loadModuleData(module) {
    try {
        LoadingManager.show(`載入${module}數據...`);
        
    switch(module) {
        case 'dashboard':
            await loadDashboardData();
            break;
        case 'domains':
            await loadDomainsData();
            break;
        case 'settings':
            await loadSettingsData();
            break;
        case 'logs':
            await loadLogsData();
            break;
    }
        
        LoadingManager.hide();
    } catch (error) {
        console.error(`載入${module}數據失敗:`, error);
        NotificationManager.showError(`載入${module}數據失敗: ${error.message}`);
        LoadingManager.hide();
    }
}

// 載入配置
async function loadConfigs() {
    try {
        const response = await ApiClient.getConfigs();
        
        if (response.success && response.configs) {
            currentConfigs = response.configs.map(config => DataAdapter.adaptConfig(config));
        } else {
            currentConfigs = [];
        }
        
        return currentConfigs;
    } catch (error) {
        console.error('載入配置失敗:', error);
        throw error;
    }
}

// 載入儀表板數據
async function loadDashboardData() {
    try {
        // 獲取狀態信息
        const statusData = await ApiClient.getStatus();
        
        if (!statusData.success) {
            throw new Error('獲取狀態失敗');
        }
        
        // 更新上次更新時間
        const lastUpdateElement = document.getElementById('last-update-time');
        if (lastUpdateElement) {
            if (statusData.last_update) {
                lastUpdateElement.textContent = Utils.formatLastUpdate(statusData.last_update);
            } else {
                lastUpdateElement.textContent = '從未更新';
    }
}

        // 更新DDNS記錄表格
        const tableBody = document.getElementById('ddns-records');
        if (!tableBody) return;
        
        tableBody.innerHTML = '';
        
        if (currentConfigs.length === 0) {
            tableBody.innerHTML = `
                <tr>
                    <td colspan="7" class="text-center">沒有找到 DDNS 記錄。點擊 "添加記錄" 按鈕來創建。</td>
                </tr>
            `;
            return;
        }
        
        // 填充表格
        currentConfigs.forEach(config => {
            const row = document.createElement('tr');
            row.innerHTML = `
                <td>${config.record_name}</td>
                <td>${config.record_type}</td>
                <td>${config.current_ip || '未知'}</td>
                <td>${Utils.formatInterval(config.update_interval)}</td>
                <td>${Utils.formatLastUpdate(config.last_update_time)}</td>
                <td>${Utils.getStatusBadge(config.status)}</td>
                <td>
                    <button class="btn primary sm update-record" data-id="${config.record_id}">
                        <span class="icon">🔄</span>更新
                    </button>
                    <button class="btn sm edit-record" data-id="${config.record_id}">
                        <span class="icon">✏️</span>編輯
                    </button>
                    <button class="btn danger sm delete-record" data-id="${config.record_id}">
                        <span class="icon">🗑️</span>刪除
                    </button>
                </td>
            `;
            tableBody.appendChild(row);
        });
        
        // 綁定記錄操作按鈕事件
        bindRecordButtons();
    } catch (error) {
        console.error('載入儀表板數據失敗:', error);
        throw error;
    }
}

// 載入域名管理數據
async function loadDomainsData() {
    // 實現域名管理頁面數據載入
    const domainsList = document.querySelector('.domains-list');
    
    if (!domainsList) return;
    
    domainsList.innerHTML = '';
    
    if (currentConfigs.length === 0) {
        domainsList.innerHTML = `
            <div class="empty-state">
                <p>沒有找到任何域名記錄。點擊"添加域名"按鈕來創建。</p>
            </div>
        `;
        return;
    }
    
    currentConfigs.forEach(config => {
        const card = document.createElement('div');
        card.className = 'card domain-card';
        card.innerHTML = `
            <div class="card-header">
                <h3>${config.record_name}</h3>
                <div class="card-actions">
                    <button class="btn primary sm update-record" data-id="${config.record_id}">
                        <span class="icon">🔄</span>更新
                    </button>
                    <button class="btn sm edit-record" data-id="${config.record_id}">
                        <span class="icon">✏️</span>編輯
                    </button>
                    <button class="btn danger sm delete-record" data-id="${config.record_id}">
                        <span class="icon">🗑️</span>刪除
                    </button>
                </div>
            </div>
            <div class="card-body">
                <div class="info-row">
                    <span class="info-label">類型:</span>
                    <span class="info-value">${config.record_type}</span>
                </div>
                <div class="info-row">
                    <span class="info-label">當前IP:</span>
                    <span class="info-value">${config.current_ip || '未知'}</span>
                </div>
                <div class="info-row">
                    <span class="info-label">更新間隔:</span>
                    <span class="info-value">${Utils.formatInterval(config.update_interval)}</span>
                </div>
                <div class="info-row">
                    <span class="info-label">上次更新:</span>
                    <span class="info-value">${Utils.formatLastUpdate(config.last_update_time)}</span>
                </div>
                <div class="info-row">
                    <span class="info-label">狀態:</span>
                    <span class="info-value">${Utils.getStatusBadge(config.status)}</span>
                </div>
            </div>
        `;
        
        domainsList.appendChild(card);
    });
    
    // 綁定域名卡片上的按鈕
    bindRecordButtons();
}

// 載入設定數據
async function loadSettingsData() {
    // 實現設定頁面數據載入
    // ...
}

// 載入日誌數據
async function loadLogsData() {
    // 實現日誌頁面數據載入
    // ...
}

// 綁定記錄按鈕事件
function bindRecordButtons() {
    // 更新記錄按鈕
    document.querySelectorAll('.update-record').forEach(button => {
        button.addEventListener('click', async function() {
            const recordId = this.getAttribute('data-id');
            await updateRecord(recordId);
        });
    });
    
    // 編輯記錄按鈕
    document.querySelectorAll('.edit-record').forEach(button => {
        button.addEventListener('click', function() {
            const recordId = this.getAttribute('data-id');
            editRecord(recordId);
        });
    });
    
    // 刪除記錄按鈕
    document.querySelectorAll('.delete-record').forEach(button => {
        button.addEventListener('click', async function() {
            const recordId = this.getAttribute('data-id');
            await deleteRecord(recordId);
        });
    });
}

// 更新記錄
async function updateRecord(recordId) {
    try {
        LoadingManager.show('正在更新 DDNS 記錄...');
        
        const result = await ApiClient.updateDdns();
        
        if (result.success) {
            NotificationManager.showSuccess('DDNS 記錄更新成功');
            
            // 重新載入數據
            await loadConfigs();
            await loadDashboardData();
        } else {
            throw new Error('更新失敗: ' + result.message);
        }
        
        LoadingManager.hide();
    } catch (error) {
        console.error('更新記錄失敗:', error);
        NotificationManager.showError('更新記錄失敗: ' + error.message);
        LoadingManager.hide();
    }
}

// 編輯記錄
function editRecord(recordId) {
    // 查找記錄
    const config = currentConfigs.find(c => c.record_id === recordId);
        
    if (!config) {
        NotificationManager.showError('找不到指定的記錄');
        return;
    }
    
    // 填充編輯表單
    const form = document.getElementById('record-form');
    if (!form) return;
    
    document.getElementById('record-id').value = config.record_id;
    document.getElementById('zone-id').value = config.zone_id;
    document.getElementById('record-name').value = config.record_name;
    
    // 設置記錄類型
    const typeSelect = document.getElementById('new-record-type');
    if (typeSelect) {
        typeSelect.value = config.record_type;
        }
        
    // 設置更新間隔
    const intervalInput = document.getElementById('new-record-ttl');
    if (intervalInput) {
        intervalInput.value = config.update_interval;
    }
    
    // 設置代理狀態
    const proxiedCheckbox = document.getElementById('new-record-proxied');
    if (proxiedCheckbox) {
        proxiedCheckbox.checked = config.proxied || false;
    }
    
    // 更新對話框標題
    const modalTitle = document.getElementById('modal-title');
    if (modalTitle) {
        modalTitle.textContent = '編輯 DDNS 記錄';
    }
    
    // 顯示對話框
    showModal('record-modal');
}

// 刪除記錄
async function deleteRecord(recordId) {
    if (!confirm('確定要刪除此 DDNS 記錄嗎？此操作無法撤銷。')) {
        return;
    }
    
    try {
        LoadingManager.show('正在刪除記錄...');
        
        // 過濾掉要刪除的記錄
        const newConfigs = currentConfigs.filter(c => c.record_id !== recordId);
        
        // 保存新的配置列表
        const result = await ApiClient.saveConfigs(
            newConfigs.map(c => DataAdapter.prepareConfigForSave(c))
        );
        
        if (result.success) {
            NotificationManager.showSuccess('記錄已成功刪除');
        
            // 更新當前配置
            currentConfigs = newConfigs;
            
            // 重新載入數據
            await loadDashboardData();
        } else {
            throw new Error('刪除失敗: ' + result.message);
        }
        
        LoadingManager.hide();
    } catch (error) {
        console.error('刪除記錄失敗:', error);
        NotificationManager.showError('刪除記錄失敗: ' + error.message);
        LoadingManager.hide();
    }
}

// 綁定模態框事件
function bindModalEvents() {
    // 關閉按鈕
    document.querySelectorAll('.modal .close-btn').forEach(button => {
        button.addEventListener('click', function() {
            const modal = this.closest('.modal');
            if (modal) {
                hideModal(modal.id);
            }
        });
    });
    
    // 點擊背景關閉
    document.querySelectorAll('.modal .modal-backdrop').forEach(backdrop => {
        backdrop.addEventListener('click', function() {
            const modal = this.closest('.modal');
            if (modal) {
                hideModal(modal.id);
            }
        });
    });
        
    // 記錄表單提交
    const recordForm = document.getElementById('record-form');
    if (recordForm) {
        recordForm.addEventListener('submit', async function(e) {
            e.preventDefault();
            await saveRecord();
        });
    }
    
    // 保存按鈕
    const saveBtn = document.getElementById('save-btn');
    if (saveBtn) {
        saveBtn.addEventListener('click', async function() {
            await saveRecord();
        });
}

    // 取消按鈕
    const cancelBtn = document.getElementById('cancel-btn');
    if (cancelBtn) {
        cancelBtn.addEventListener('click', function() {
            hideModal('record-modal');
        });
    }
}

// 保存記錄
async function saveRecord() {
    // 獲取表單數據
    const form = document.getElementById('record-form');
    if (!form) return;
    
    const recordId = document.getElementById('record-id').value;
    const zoneId = document.getElementById('zone-id').value;
    const recordName = document.getElementById('record-name').value;
    const recordType = document.getElementById('new-record-type')?.value || 'A';
    const updateInterval = parseInt(document.getElementById('new-record-ttl')?.value || '300');
    const proxied = document.getElementById('new-record-proxied')?.checked || false;
    
    // 基本驗證
    if (!zoneId || !recordName) {
        NotificationManager.showError('請填寫所有必填欄位');
        return;
    }
    
    try {
        LoadingManager.show('正在保存記錄...');
        
        const formData = {
            record_id: recordId || undefined,
                    zone_id: zoneId,
                    record_name: recordName,
            record_type: recordType,
            update_interval: updateInterval,
            proxied: proxied
        };
        
        // 準備配置對象
        const configToSave = DataAdapter.prepareConfigForSave(formData);
        
        // 更新或添加配置
        let newConfigs;
        
        if (recordId) {
            // 更新現有配置
            newConfigs = currentConfigs.map(c => 
                c.record_id === recordId ? {...c, ...formData} : c
            );
        } else {
            // 添加新配置
            newConfigs = [...currentConfigs, formData];
        }
        
        // 保存配置
        const apiConfigs = newConfigs.map(c => DataAdapter.prepareConfigForSave(c));
        const result = await ApiClient.saveConfigs(apiConfigs);
        
        if (result.success) {
            NotificationManager.showSuccess(recordId ? '記錄更新成功' : '記錄創建成功');
            
            // 更新當前配置
            currentConfigs = newConfigs;
        
        // 隱藏對話框
        hideModal('record-modal');
        
            // 重新載入數據
            await loadDashboardData();
        } else {
            throw new Error('保存失敗: ' + result.message);
        }
        
        LoadingManager.hide();
    } catch (error) {
        console.error('保存記錄失敗:', error);
        NotificationManager.showError('保存記錄失敗: ' + error.message);
        LoadingManager.hide();
    }
}

// 檢查是否首次訪問
async function checkFirstVisit() {
    try {
        if (currentConfigs.length === 0) {
            showModal('setup-wizard-modal');
        }
    } catch (error) {
        console.error('檢查首次訪問失敗:', error);
    }
}

// 綁定向導事件
function bindWizardEvents() {
    // 步驟1: 移至步驟2
    const nextStep1Btn = document.getElementById('wizard-validate-token');
    if (nextStep1Btn) {
        nextStep1Btn.addEventListener('click', function() {
            const apiToken = document.getElementById('wizard-api-token').value;
            
            if (!apiToken) {
                NotificationManager.showError('請輸入 Cloudflare API 令牌');
                return;
}

            // 存儲API令牌供後續步驟使用
            window.wizardData = {
                api_token: apiToken
            };
            
            hideElement('wizard-step-1');
            showElement('wizard-step-2');
        });
    }
    
    // 步驟2: 返回步驟1
    const backStep1Btn = document.getElementById('wizard-back-step-1');
    if (backStep1Btn) {
        backStep1Btn.addEventListener('click', function() {
            hideElement('wizard-step-2');
            showElement('wizard-step-1');
    });
}

    // 步驟2: 移至步驟3
    const nextStep3Btn = document.getElementById('wizard-next-step-3');
    if (nextStep3Btn) {
        nextStep3Btn.addEventListener('click', function() {
            const zoneId = document.getElementById('wizard-zone-id').value;
            const recordName = document.getElementById('wizard-record-name').value;
            const recordId = document.getElementById('wizard-record-id').value;
            
            if (!zoneId || !recordName || !recordId) {
                NotificationManager.showError('請填寫所有必填欄位');
                return;
}

            // 更新向導數據
            window.wizardData = {
                ...window.wizardData,
                zone_id: zoneId,
                record_name: recordName,
                record_id: recordId
            };
            
            hideElement('wizard-step-2');
            showElement('wizard-step-3');
        });
    }
    
    // 步驟3: 返回步驟2
    const backStep2Btn = document.getElementById('wizard-back-step-2');
    if (backStep2Btn) {
        backStep2Btn.addEventListener('click', function() {
            hideElement('wizard-step-3');
            showElement('wizard-step-2');
        });
}

    // 完成設定
    const finishBtn = document.getElementById('wizard-finish');
    if (finishBtn) {
        finishBtn.addEventListener('click', async function() {
            const ipType = document.querySelector('input[name="wizard-ip-type"]:checked')?.value || 'ipv4';
            const updateInterval = parseInt(document.getElementById('wizard-update-interval').value || '300');
            
            if (isNaN(updateInterval) || updateInterval < 60) {
                NotificationManager.showError('更新間隔必須至少為60秒');
                return;
            }
            
            // 完成向導數據
            window.wizardData = {
                ...window.wizardData,
                ip_type: ipType,
                update_interval: updateInterval
            };
            
            await saveWizardConfig();
        });
    }
}

// 保存向導配置
async function saveWizardConfig() {
    try {
        if (!window.wizardData) {
            throw new Error('向導數據未初始化');
        }
        
        LoadingManager.show('正在保存配置...');
        
        const config = {
            api_token: window.wizardData.api_token,
            zone_id: window.wizardData.zone_id,
            record_id: window.wizardData.record_id,
            record_name: window.wizardData.record_name,
            ip_type: window.wizardData.ip_type,
            update_interval: window.wizardData.update_interval
        };
        
        // 驗證配置
        const validateResult = await ApiClient.validateConfig(config);
        
        if (!validateResult.success || !validateResult.is_valid) {
            throw new Error('配置驗證失敗: ' + validateResult.message);
        }
        
        // 保存配置
        const saveResult = await ApiClient.saveConfigs([config]);
        
        if (!saveResult.success) {
            throw new Error('保存配置失敗: ' + saveResult.message);
        }
        
        NotificationManager.showSuccess('DDNS 配置已成功設置');
        
        // 關閉向導
        hideModal('setup-wizard-modal');
        
        // 重新載入配置
        await loadConfigs();
        await loadDashboardData();
        
        LoadingManager.hide();
    } catch (error) {
        console.error('保存向導配置失敗:', error);
        NotificationManager.showError('保存配置失敗: ' + error.message);
        LoadingManager.hide();
    }
}

// 顯示模態框
function showModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) {
        modal.style.display = 'block';
        setTimeout(() => {
            modal.classList.add('show');
        }, 10);
    }
}

// 隱藏模態框
function hideModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) {
        modal.classList.remove('show');
        setTimeout(() => {
            modal.style.display = 'none';
        }, 300);
    }
}

// 顯示元素
function showElement(elementId) {
    const element = document.getElementById(elementId);
    if (element) {
        element.style.display = 'block';
    }
}

// 隱藏元素
function hideElement(elementId) {
    const element = document.getElementById(elementId);
    if (element) {
        element.style.display = 'none';
    }
} 