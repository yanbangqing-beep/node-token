//! 错误类型定义
//!
//! 定义 node-token 的各种错误类型，使用 thiserror 实现。

use thiserror::Error;

/// node-token 主错误类型
#[derive(Error, Debug)]
pub enum NodeTokenError {
    /// 配置错误
    #[error("Configuration error: {0}")]
    Config(#[from] anyhow::Error),
    
    /// 网络请求错误
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    /// HTTP 请求失败（非网络错误，如 4xx/5xx 响应）
    #[error("HTTP request failed with status {status}: {message}")]
    HttpError {
        status: u16,
        message: String,
    },
    
    /// Ollama 调用错误
    #[error("Ollama error: {0}")]
    Ollama(String),
    
    /// 协议解析错误
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    /// 本地存储错误
    #[error("Storage error: {0}")]
    Storage(String),
    
    /// 任务执行错误
    #[error("Task execution error: {0}")]
    TaskExecution(String),
    
    /// 节点被排除
    #[error("Node has been excluded")]
    NodeExcluded,
    
    /// Session 无效或丢失
    #[error("Invalid or missing session")]
    InvalidSession,
}

/// 存储操作的结果类型
pub type StorageResult<T> = Result<T, NodeTokenError>;

/// 网络请求的结果类型
pub type NetworkResult<T> = Result<T, NodeTokenError>;

/// Ollama 操作的结果类型
pub type OllamaResult<T> = Result<T, NodeTokenError>;
