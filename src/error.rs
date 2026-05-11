//! 错误类型定义
//!
//! 定义 node-token 的各种错误类型，使用 thiserror 实现。

use thiserror::Error;

/// node-token 主错误类型
#[derive(Error, Debug)]
#[allow(dead_code)] // 部分变体在后续阶段使用
pub enum NodeTokenError {
    /// 配置错误
    #[error("Configuration error: {0}")]
    Config(#[from] anyhow::Error),

    /// 网络请求错误
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// HTTP 请求失败（非网络错误，如 4xx/5xx 响应）
    #[error("HTTP request failed with status {status}: {message}")]
    HttpError { status: u16, message: String },

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

    /// 注册失败（如未找到模型）
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),
}

// 手动实现 PartialEq（因为某些变体包含不支持 PartialEq 的类型）
impl PartialEq for NodeTokenError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                NodeTokenError::HttpError {
                    status: s1,
                    message: m1,
                },
                NodeTokenError::HttpError {
                    status: s2,
                    message: m2,
                },
            ) => s1 == s2 && m1 == m2,
            (NodeTokenError::Ollama(m1), NodeTokenError::Ollama(m2)) => m1 == m2,
            (NodeTokenError::Protocol(m1), NodeTokenError::Protocol(m2)) => m1 == m2,
            (NodeTokenError::Storage(m1), NodeTokenError::Storage(m2)) => m1 == m2,
            (NodeTokenError::TaskExecution(m1), NodeTokenError::TaskExecution(m2)) => m1 == m2,
            (NodeTokenError::NodeExcluded, NodeTokenError::NodeExcluded) => true,
            (NodeTokenError::InvalidSession, NodeTokenError::InvalidSession) => true,
            (NodeTokenError::RegistrationFailed(m1), NodeTokenError::RegistrationFailed(m2)) => m1 == m2,
            // Config 和 Network 包含不支持 PartialEq 的类型，总是返回 false
            _ => false,
        }
    }
}

/// 存储操作的结果类型
#[allow(dead_code)] // 在后续阶段使用
pub type StorageResult<T> = Result<T, NodeTokenError>;

/// 网络请求的结果类型
pub type NetworkResult<T> = Result<T, NodeTokenError>;

/// Ollama 操作的结果类型
pub type OllamaResult<T> = Result<T, NodeTokenError>;
