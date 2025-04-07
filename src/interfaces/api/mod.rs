// API 路由模塊
pub mod router;
pub mod common;

// API 處理器模塊
mod config;
pub mod health;
mod ip;
mod status;
mod update;
mod wizard;

// 重新導出配置路由函數
pub use router::configure_routes; 