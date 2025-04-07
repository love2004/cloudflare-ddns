// 使用立即執行函數表達式(IIFE)來避免全局命名空間污染
(function() {
    // 加載狀態管理器，設為全局可訪問
    window.LoadingManager = {
      count: 0,
      element: null,
      
      // 初始化加載指示器
      init() {
        if (this.element) return;
        
        this.element = document.createElement('div');
        this.element.className = 'loading-indicator hidden';
        this.element.innerHTML = `
          <div class="spinner"></div>
          <div class="loading-text">載入中...</div>
        `;
        document.body.appendChild(this.element);
      },
      
      // 顯示加載中
      show(message = '載入中...') {
        this.init();
        this.count++;
        this.element.querySelector('.loading-text').textContent = message;
        this.element.classList.remove('hidden');
      },
      
      // 隱藏加載中
      hide() {
        if (!this.element) return;
        
        this.count--;
        if (this.count <= 0) {
          this.count = 0;
          this.element.classList.add('hidden');
        }
      }
    };

    // 通知管理器，設為全局可訪問
    window.NotificationManager = {
      container: null,
      
      // 初始化通知容器
      init() {
        if (this.container) return;
        
        this.container = document.createElement('div');
        this.container.className = 'notification-container';
        document.body.appendChild(this.container);
      },
      
      // 顯示錯誤通知
      showError(message, duration = 5000) {
        this.init();
        this._showNotification(message, 'error', duration);
      },
      
      // 顯示成功通知
      showSuccess(message, duration = 3000) {
        this.init();
        this._showNotification(message, 'success', duration);
      },
      
      // 顯示信息通知
      showInfo(message, duration = 3000) {
        this.init();
        this._showNotification(message, 'info', duration);
      },
      
      // 內部方法：創建並顯示通知
      _showNotification(message, type, duration) {
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        
        const content = document.createElement('div');
        content.className = 'notification-content';
        content.textContent = message;
        
        const closeBtn = document.createElement('button');
        closeBtn.className = 'notification-close';
        closeBtn.innerHTML = '&times;';
        closeBtn.addEventListener('click', () => {
          notification.classList.add('notification-hiding');
          setTimeout(() => notification.remove(), 300);
        });
        
        notification.appendChild(content);
        notification.appendChild(closeBtn);
        this.container.appendChild(notification);
        
        // 自動消失
        if (duration > 0) {
          setTimeout(() => {
            if (notification.parentNode) {
              notification.classList.add('notification-hiding');
              setTimeout(() => notification.remove(), 300);
            }
          }, duration);
        }
        
        // 動畫效果
        setTimeout(() => notification.classList.add('notification-show'), 10);
      }
    };

    // 實用函數，設為全局可訪問
    window.Utils = {
      // 格式化日期時間
      formatDateTime(dateString) {
        if (!dateString) return '未知';
        
        const date = new Date(dateString);
        if (isNaN(date.getTime())) return dateString;
        
        return date.toLocaleString('zh-TW', {
          year: 'numeric',
          month: '2-digit',
          day: '2-digit',
          hour: '2-digit',
          minute: '2-digit',
          second: '2-digit'
        });
      },
      
      // 格式化時間間隔（秒數轉為友好顯示）
      formatInterval(seconds) {
        if (!seconds) return '未知';
        
        if (seconds < 60) {
          return `${seconds}秒`;
        } else if (seconds < 3600) {
          return `${Math.floor(seconds / 60)}分鐘`;
        } else if (seconds < 86400) {
          return `${Math.floor(seconds / 3600)}小時`;
        } else {
          return `${Math.floor(seconds / 86400)}天`;
        }
      },
      
      // 格式化上次更新時間
      formatLastUpdate(dateString) {
        if (!dateString) return '從未更新';
        
        const date = new Date(dateString);
        if (isNaN(date.getTime())) return '無效日期';
        
        const now = new Date();
        const diffMs = now - date;
        const diffSec = Math.floor(diffMs / 1000);
        
        if (diffSec < 60) {
          return '剛剛更新';
        } else if (diffSec < 3600) {
          return `${Math.floor(diffSec / 60)}分鐘前`;
        } else if (diffSec < 86400) {
          return `${Math.floor(diffSec / 3600)}小時前`;
        } else {
          return `${Math.floor(diffSec / 86400)}天前`;
        }
      },
      
      // 獲取狀態徽章HTML
      getStatusBadge(status) {
        switch(status) {
          case 'active':
            return '<span class="badge success">活躍</span>';
          case 'error':
            return '<span class="badge danger">錯誤</span>';
          case 'updating':
            return '<span class="badge info">更新中</span>';
          default:
            return '<span class="badge secondary">未知</span>';
        }
      }
    };
})(); 