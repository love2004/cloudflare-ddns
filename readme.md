# Cloudflare DDNS 更新工具

一個高效、穩定的 Cloudflare DDNS 更新工具，使用 Rust 開發，支持 IPv4 和 IPv6 記錄更新，並提供直觀的 Web 管理界面。

## 主要特性

- 支持 IPv4 和 IPv6 DDNS 記錄自動更新
- 使用 Rust 實現高性能、低資源佔用
- 提供直觀的 Web 管理界面
- 支持多域名同時管理
- 提供 API 接口，方便與其他系統集成
- 多種配置方式：環境變量、配置文件

## 技術架構

### 前端

- 純 JavaScript 實現，無需額外框架
- 模塊化設計，清晰的關注點分離
- 響應式布局，適配不同設備尺寸
- 資源優化，保證快速加載和渲染

### 後端

- Rust + Actix Web 框架
- 領域驅動設計 (DDD) 架構
- 異步處理，高效利用系統資源
- RESTful API 設計

## 安裝與使用

### 從源碼構建

```bash
# 克隆倉庫
git clone https://github.com/yourusername/cloudflare-ddns.git
cd cloudflare-ddns

# 構建項目
cargo build --release

# 運行
./target/release/cloudflare-ddns
```

## 配置

通過以下方式之一配置 DDNS 服務：

1. 環境變量
2. .env 文件
3. 配置文件 (JSON/YAML)
4. Web 界面配置

基本配置項：

```
CLOUDFLARE_API_TOKEN=your_cloudflare_api_token
CLOUDFLARE_ZONE_ID=your_cloudflare_zone_id
CLOUDFLARE_RECORD_ID=your_cloudflare_record_id
CLOUDFLARE_RECORD_NAME=example.com
DDNS_UPDATE_INTERVAL=300
```

## 貢獻

歡迎提交 Issue 或 Pull Request 參與項目開發。

## 授權

MIT License 