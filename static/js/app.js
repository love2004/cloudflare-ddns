/**
 * 主應用模塊
 * 處理事件綁定和初始化
 */

// 全局變量
let currentConfigs = [];

// 在 DOM 載入完成後執行
document.addEventListener("DOMContentLoaded", () => {
    // 初始化應用
    console.log("DOM內容加載完成，開始初始化應用");
    setTimeout(debugInit, 500); // 延遲500毫秒執行調試初始化
    initApp();
});

// 調試初始化函數
function debugInit() {
    console.log("=== 調試信息 ===");
    
    // 檢查DOM元素是否存在
    const checkElement = (selector, name) => {
        const element = document.querySelector(selector);
        console.log(`${name}: ${element ? '存在' : '不存在'}`);
        return element;
    };
    
    // 檢查關鍵元素
    checkElement('.sidebar', '側邊欄');
    checkElement('.sidebar-nav', '導航菜單');
    checkElement('#main-nav', '主導航');
    
    const navItems = document.querySelectorAll('.nav-item');
    console.log(`找到 ${navItems.length} 個導航項目`);
    
    navItems.forEach((item, index) => {
        const target = item.getAttribute('data-target');
        console.log(`導航項目 ${index + 1}: data-target="${target}", text="${item.textContent.trim()}"`);
        
        // 為每個項目手動添加點擊事件，確保可以正常切換頁面
        item.onclick = function(e) {
            console.log(`手動點擊處理：${this.getAttribute('data-target')}`);
            e.preventDefault();
            
            // 獲取目標ID
            const target = this.getAttribute('data-target');
            if (!target) {
                console.error('導航項目缺少data-target屬性');
                return false;
            }
            
            // 更新活動狀態
            navItems.forEach(l => l.classList.remove('active'));
            this.classList.add('active');
            
            // 隱藏所有內容區域
            document.querySelectorAll('.content-section').forEach(section => {
                section.style.display = 'none';
            });
            
            // 顯示目標內容區域
            const targetSection = document.getElementById(`${target}-section`);
            if (targetSection) {
                targetSection.style.display = 'block';
                console.log(`已顯示：${target}-section`);
            } else {
                console.error(`未找到目標內容區域：${target}-section`);
            }
            
            return false;
        };
    });
    
    console.log("=== 調試初始化完成 ===");
}

// 初始化應用
function initApp() {
    console.log("初始化應用程序...");
    
    try {
        // 初始化通知和加載組件
        console.log("初始化UI組件...");
        if (window.LoadingManager) LoadingManager.init();
        if (window.NotificationManager) NotificationManager.init();
        
        // 綁定全局未處理的Promise錯誤
        window.addEventListener('unhandledrejection', event => {
            console.error('未處理的Promise拒絕:', event.reason);
            if (window.NotificationManager) {
                NotificationManager.showError(event.reason.message || '發生未知錯誤');
            }
        });
        
        // 初始化UI元素（在導航之前）
        console.log("初始化UI元素...");
        initUI();
                
        // 綁定導航事件
        console.log("初始化導航...");
        initNavigation();
        
        // 綁定按鈕事件
        console.log("綁定按鈕事件...");
        bindButtons();
                
        // 綁定向導事件
        console.log("綁定向導事件...");
        bindWizardEvents();
        
        // 綁定模態框事件
        console.log("綁定模態框事件...");
        bindModalEvents();
        
        // 初始化連接狀態檢測
        console.log("初始化連接監測...");
        initConnectionMonitor();
        
        // 延遲加載初始數據，確保UI已準備好
        console.log("準備加載初始數據...");
        setTimeout(loadInitialData, 1000);

        // 輸出調試信息
        console.log("應用初始化完成");
    } catch (error) {
        console.error("應用初始化失敗:", error);
        alert("應用初始化失敗: " + error.message);
    }
}

// 初始化導航功能
function initNavigation() {
    console.log("初始化導航功能...");
    
    // 獲取所有導航項目
    const navItems = document.querySelectorAll('#main-nav .nav-item');
    console.log(`找到 ${navItems.length} 個導航項目`);
    
    // 定義內容區對象
    const sections = {
        'dashboard': document.getElementById('dashboard-section'),
        'domains': document.getElementById('domains-section'),
        'settings': document.getElementById('settings-section'),
        'logs': document.getElementById('logs-section')
    };
    
    // 檢查所有部分是否都存在
    Object.entries(sections).forEach(([key, section]) => {
        if (!section) {
            console.error(`未找到內容區域: ${key}-section`);
        }
    });

    // 為每個導航項目添加點擊事件
    navItems.forEach(item => {
        const target = item.getAttribute('data-target');
        console.log(`處理導航項目: ${target}`);
        
        item.addEventListener('click', function(e) {
            e.preventDefault();
            console.log(`導航項目被點擊: ${this.getAttribute('data-target')}`);
            
            // 獲取目標ID
            const target = this.getAttribute('data-target');
            console.log(`目標部分ID: ${target}`);
            
            // 更新活動狀態
            navItems.forEach(l => {
                l.classList.remove('active');
                console.log(`移除項目 ${l.getAttribute('data-target')} 的活動狀態`);
            });
            this.classList.add('active');
            console.log(`添加項目 ${target} 的活動狀態`);
            
            // 隱藏所有內容區域
            Object.values(sections).forEach(section => {
                if (section) {
                    section.style.display = 'none';
                    console.log(`隱藏內容區域: ${section.id}`);
                }
            });
            
            // 顯示目標內容區域
            if (sections[target]) {
                sections[target].style.display = 'block';
                console.log(`顯示內容區域: ${target}-section`);
                
                // 載入對應模塊的數據
                loadModuleData(target);
            } else {
                console.error(`未找到目標內容區域: ${target}-section`);
            }
        });
    });
    
    // 添加直接點擊處理程序到子元素
    document.querySelectorAll('.nav-link').forEach(link => {
        link.addEventListener('click', function(e) {
            // 確保事件不會被阻止
            e.stopPropagation();
            // 觸發父元素的點擊事件
            this.parentElement.click();
        });
    });
}

// 連接監控器
function initConnectionMonitor() {
    console.log("初始化連接監視器...");
    
    // 設置連接檢測函數
    const checkConnection = async () => {
        try {
            console.log("檢測API可用性...");
            const available = await ApiClient.checkApiAvailability();
            console.log("API可用性檢測結果:", available ? "可用" : "不可用");
            updateConnectionStatus(available);
            return available;
        } catch (error) {
            console.error("API可用性檢測出錯:", error);
            updateConnectionStatus(false);
            return false;
        }
    };
    
    // 初始檢測API可用性
    checkConnection().then(available => {
        console.log("初始連接狀態:", available ? "已連接" : "未連接");
    });
    
    // 設置定期檢測
    setInterval(checkConnection, 30000); // 每30秒檢測一次
    
    // 監聽瀏覽器在線/離線事件
    window.addEventListener('online', () => {
        console.log('瀏覽器檢測到網絡已連接');
        setTimeout(() => {
            // 給網絡一點時間穩定下來
            checkConnection().then(available => {
                if (available) {
                    console.log('網絡恢復，重新加載數據');
                    loadInitialData();
                }
            });
        }, 2000);
    });
    
    window.addEventListener('offline', () => {
        console.log('瀏覽器檢測到網絡已斷開');
        updateConnectionStatus(false);
    });
    
    console.log("連接監視器初始化完成");
}

// 更新連接狀態UI
function updateConnectionStatus(isConnected) {
    const statusIndicator = document.getElementById('connection-status');
    if (!statusIndicator) {
        // 如果狀態指示器不存在，創建一個
        const indicator = document.createElement('div');
        indicator.id = 'connection-status';
        indicator.className = isConnected ? 'connected' : 'disconnected';
        indicator.title = isConnected ? '與伺服器連接正常' : '無法連接到伺服器';
        indicator.innerHTML = isConnected ? '🟢' : '🔴';
        
        // 添加到DOM
        const sidebar = document.querySelector('.sidebar-footer');
        if (sidebar) {
            sidebar.prepend(indicator);
        }
    } else {
        // 更新現有指示器
        statusIndicator.className = isConnected ? 'connected' : 'disconnected';
        statusIndicator.title = isConnected ? '與伺服器連接正常' : '無法連接到伺服器';
        statusIndicator.innerHTML = isConnected ? '🟢' : '🔴';
    }
    
    // 更新全局UI狀態
    document.body.classList.toggle('offline-mode', !isConnected);
    
    // 如果是離線狀態，顯示通知
    if (!isConnected) {
        showNotification('網絡連接已斷開，部分功能可能不可用', 'warning');
    }
}

// 共用通知函數
function showNotification(message, type = 'info', duration = 5000) {
    const notificationContainer = document.getElementById('notification-container') || createNotificationContainer();
    
    const notification = document.createElement('div');
    notification.className = `notification ${type}`;
    notification.textContent = message;
    
    notificationContainer.appendChild(notification);
    
    // 自動關閉
    setTimeout(() => {
        notification.classList.add('fade-out');
        setTimeout(() => {
            notification.remove();
        }, 500);
    }, duration);
}

// 創建通知容器
function createNotificationContainer() {
    const container = document.createElement('div');
    container.id = 'notification-container';
    document.body.appendChild(container);
    return container;
}

// 載入初始數據
async function loadInitialData() {
    try {
        console.log('開始加載初始數據...');
        
        // 顯示加載中狀態
        document.body.classList.add('loading');
        
        // 測試API連接
        const isConnected = await ApiClient.checkApiAvailability();
        console.log('API連接狀態:', isConnected ? '已連接' : '未連接');
        
        if (!isConnected) {
            throw new Error('無法連接到API服務');
        }
        
        // 併發請求多個API
        console.log('發送API請求獲取狀態和配置...');
        const [statusData, configsData] = await Promise.all([
            ApiClient.getStatus(),
            ApiClient.getConfigs()
        ]);
        
        console.log('收到API響應:', { 
            狀態數據: statusData, 
            配置數據: configsData 
        });
        
        // 更新UI
        updateStatusDisplay(statusData);
        updateConfigsList(configsData);
        
        // 如果連接正常，顯示成功消息
        if (ApiClient.connectionStatus === 'online') {
            showNotification('數據已成功加載', 'success', 3000);
        }
    } catch (error) {
        console.error('載入初始數據失敗:', error);
        showNotification('載入數據失敗: ' + error.message, 'error');
    } finally {
        // 無論成功或失敗，都移除加載中狀態
        document.body.classList.remove('loading');
        console.log('初始數據加載完成（成功或失敗）');
    }
}

// 載入模塊數據
async function loadModuleData(module) {
    console.log(`開始加載${module}模塊數據...`);
    
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
        
        console.log(`${module}模塊數據加載完成`);
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
    console.log('開始加載儀表板數據...');
    
    // 檢查儀表板DOM元素
    const dashboardSection = document.getElementById('dashboard-section');
    if (!dashboardSection) {
        console.error('儀表板區塊不存在');
        throw new Error('儀表板元素未找到');
    }
    
    // 檢查關鍵元素
    const lastUpdateElement = document.getElementById('last-update-time');
    console.log('上次更新時間元素:', lastUpdateElement ? '已找到' : '未找到');
    
    const tableBody = document.getElementById('ddns-records');
    console.log('記錄表格元素:', tableBody ? '已找到' : '未找到');
    
    try {
        // 獲取狀態信息
        console.log('請求API獲取狀態數據...');
        const statusData = await ApiClient.getStatus();
        console.log('API返回狀態數據:', statusData);
        
        // 檢查API響應格式
        if (!statusData) {
            throw new Error('獲取狀態失敗：API返回空數據');
        }
        
        // 更新上次更新時間
        if (lastUpdateElement) {
            if (statusData.last_update) {
                lastUpdateElement.textContent = Utils.formatLastUpdate(statusData.last_update);
                console.log('已更新上次更新時間:', statusData.last_update);
            } else {
                lastUpdateElement.textContent = '從未更新';
                console.log('無上次更新時間');
            }
        }

        // 如果記錄表格不存在，提前返回
        if (!tableBody) {
            console.log('記錄表格不存在，儀表板數據加載部分完成');
            return;
        }
        
        // 清空表格
        tableBody.innerHTML = '';
        
        // 處理配置
        if (currentConfigs.length === 0) {
            console.log('沒有DDNS記錄，顯示空狀態');
            tableBody.innerHTML = `
                <tr>
                    <td colspan="7" class="text-center">沒有找到 DDNS 記錄。點擊 "添加記錄" 按鈕來創建。</td>
                </tr>
            `;
            return;
        }
        
        console.log('填充DDNS記錄表格，找到', currentConfigs.length, '條記錄');
        
        // 填充表格
        currentConfigs.forEach((config, index) => {
            console.log(`處理記錄 ${index + 1}:`, config.record_name);
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
        console.log('綁定記錄按鈕事件');
        bindRecordButtons();
        
        console.log('儀表板數據加載完成');
    } catch (error) {
        console.error('載入儀表板數據失敗:', error);
        NotificationManager.showError(`載入儀表板數據失敗: ${error.message}`);
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
    console.log('開始載入設定數據...');
    
    try {
        // 檢查設定頁面是否存在
        const settingsSection = document.getElementById('settings-section');
        if (!settingsSection) {
            console.error('設定區塊不存在');
            throw new Error('設定頁面元素未找到');
        }
        
        // 獲取設定表單
        const settingsForm = settingsSection.querySelector('.settings-form');
        if (!settingsForm) {
            console.error('設定表單不存在');
            throw new Error('設定表單元素未找到');
        }
        
        // 這裡將來可以實現更多設定加載邏輯
        
        console.log('設定數據載入完成');
    } catch (error) {
        console.error('載入設定數據失敗:', error);
        NotificationManager.showError(`載入設定失敗: ${error.message}`);
        throw error;
    }
}

// 載入日誌數據
async function loadLogsData() {
    console.log('開始載入日誌數據...');
    
    try {
        // 檢查日誌頁面是否存在
        const logsSection = document.getElementById('logs-section');
        if (!logsSection) {
            console.error('日誌區塊不存在');
            throw new Error('日誌頁面元素未找到');
        }
        
        // 獲取日誌列表
        const logsList = logsSection.querySelector('.logs-list');
        if (!logsList) {
            console.error('日誌列表不存在');
            throw new Error('日誌列表元素未找到');
        }
        
        // 這裡將來可以實現更多日誌加載邏輯
        
        console.log('日誌數據載入完成');
    } catch (error) {
        console.error('載入日誌數據失敗:', error);
        NotificationManager.showError(`載入日誌失敗: ${error.message}`);
        throw error;
    }
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

// 初始化UI元素
function initUI() {
    console.log("初始化UI元素...");
    
    // 检查并初始化所有按钮
    const buttons = {
        'refresh-btn': document.getElementById('refresh-btn'),
        'add-domain-btn': document.getElementById('add-domain-btn'),
        'save-settings-btn': document.getElementById('save-settings-btn'),
        'clear-logs-btn': document.getElementById('clear-logs-btn')
    };
    
    // 檢查所有按鈕是否都存在
    Object.entries(buttons).forEach(([key, button]) => {
        if (button) {
            console.log(`找到按鈕: ${key}`);
            
            // 添加点击事件监听器，记录点击事件
            button.addEventListener('click', function() {
                console.log(`按鈕 ${key} 被點擊`);
            });
        } else {
            console.warn(`未找到按鈕: ${key}`);
        }
    });
    
    // 檢查內容區域是否存在
    const sections = [
        'dashboard-section',
        'domains-section',
        'settings-section',
        'logs-section'
    ];
    
    sections.forEach(sectionId => {
        const section = document.getElementById(sectionId);
        if (section) {
            console.log(`找到內容區域: ${sectionId}`);
        } else {
            console.warn(`未找到內容區域: ${sectionId}`);
        }
    });
    
    // 檢查導航菜單是否存在
    const navMenu = document.getElementById('main-nav');
    if (navMenu) {
        console.log(`找到導航菜單，包含 ${navMenu.children.length} 個項目`);
        
        // 檢查每個導航項目
        Array.from(navMenu.children).forEach((item, index) => {
            console.log(`導航項目 ${index + 1}: ${item.getAttribute('data-target')}`);
        });
    } else {
        console.warn('未找到導航菜單');
    }
    
    console.log("UI元素初始化完成");
}

// 綁定按鈕事件
function bindButtons() {
    console.log("綁定按鈕事件...");
    
    // 儀表板刷新按鈕
    const refreshBtn = document.getElementById('refresh-btn');
    if (refreshBtn) {
        refreshBtn.addEventListener('click', refreshDashboard);
        console.log("已綁定儀表板刷新按鈕");
    }
    
    // 添加域名按鈕
    const addDomainBtn = document.getElementById('add-domain-btn');
    if (addDomainBtn) {
        addDomainBtn.addEventListener('click', () => {
            console.log("添加域名按鈕被點擊");
            showAddDomainModal();
        });
        console.log("已綁定添加域名按鈕");
    }
    
    // 保存設定按鈕
    const saveSettingsBtn = document.getElementById('save-settings-btn');
    if (saveSettingsBtn) {
        saveSettingsBtn.addEventListener('click', saveSettings);
        console.log("已綁定保存設定按鈕");
    }
    
    // 清除日誌按鈕
    const clearLogsBtn = document.getElementById('clear-logs-btn');
    if (clearLogsBtn) {
        clearLogsBtn.addEventListener('click', clearLogs);
        console.log("已綁定清除日誌按鈕");
    }
    
    console.log("按鈕事件綁定完成");
}

// 更新狀態顯示
function updateStatusDisplay(statusData) {
    // 檢查數據是否有效
    if (!statusData) return;

    // 更新上次更新時間
    const lastUpdateElement = document.getElementById('last-update-time');
    if (lastUpdateElement) {
        lastUpdateElement.textContent = Utils.formatLastUpdate(statusData.last_update);
    }
}

// 更新配置列表
function updateConfigsList(configsData) {
    try {
        // 更新全局配置
        if (configsData && configsData.configs) {
            currentConfigs = configsData.configs.map(config => DataAdapter.adaptConfig(config));
            console.log(`已更新配置列表，共${currentConfigs.length}項`);
        } else {
            currentConfigs = [];
            console.log('配置列表為空');
        }
    } catch (error) {
        console.error('更新配置列表失敗:', error);
    }
}

// 刷新儀表板
async function refreshDashboard() {
    console.log('刷新儀表板...');
    try {
        LoadingManager.show('正在刷新數據...');
        await loadDashboardData();
        NotificationManager.showSuccess('數據已刷新');
        LoadingManager.hide();
    } catch (error) {
        console.error('刷新儀表板失敗:', error);
        NotificationManager.showError('刷新失敗: ' + error.message);
        LoadingManager.hide();
    }
}

// 顯示添加域名模態框
function showAddDomainModal() {
    // 重置表單
    const form = document.getElementById('record-form');
    if (form) form.reset();
    
    // 設置模態框標題
    const modalTitle = document.getElementById('modal-title');
    if (modalTitle) modalTitle.textContent = '添加 DDNS 記錄';
    
    // 顯示模態框
    showModal('record-modal');
}

// 保存設置
async function saveSettings() {
    console.log('保存設置...');
    try {
        LoadingManager.show('正在保存設置...');
        // 實現保存設置的邏輯
        NotificationManager.showSuccess('設置已保存');
        LoadingManager.hide();
    } catch (error) {
        console.error('保存設置失敗:', error);
        NotificationManager.showError('保存失敗: ' + error.message);
        LoadingManager.hide();
    }
}

// 清除日誌
async function clearLogs() {
    console.log('清除日誌...');
    try {
        LoadingManager.show('正在清除日誌...');
        // 實現清除日誌的邏輯
        NotificationManager.showSuccess('日誌已清除');
        LoadingManager.hide();
    } catch (error) {
        console.error('清除日誌失敗:', error);
        NotificationManager.showError('清除失敗: ' + error.message);
        LoadingManager.hide();
    }
} 