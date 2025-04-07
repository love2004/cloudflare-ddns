pub mod config;
pub mod error;
pub mod error_context;
pub mod dns;
pub mod http;
pub mod ip;
pub mod state;

pub use error::DomainError;
pub use error_context::ResultExt; 