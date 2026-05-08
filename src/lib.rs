//! node-token - KeyCompute 个人 PC 节点客户端库
//!
//! 这是一个库 crate，提供节点客户端的核心功能：
//! - 协议类型定义（protocol）
//! - HTTP 客户端（client）
//! - 本地持久化（storage）
//! - 配置管理（config）
//! - 运行时逻辑（runtime）
//!
//! 二进制可执行文件在 `main.rs` 中定义。

// 公开模块，供集成测试和外部使用
pub mod client;
pub mod config;
pub mod error;
pub mod protocol;
pub mod runtime;
pub mod storage;

// 重新导出常用类型，方便用户使用
pub use config::NodeTokenConfig;
pub use config::load_config;
pub use error::NodeTokenError;
pub use storage::LocalStorage;
