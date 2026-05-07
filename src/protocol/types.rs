//! 节点协议类型定义
//!
//! 从 keycompute-types 复制的协议类型，node-token 作为独立项目不依赖 workspace。
//! 本协议版本固定为 `node.v1`，所有公开 JSON 字段使用 `snake_case`。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// 类型别名
// ============================================================================

/// 节点 ID
pub type NodeId = Uuid;

/// 节点会话 ID
pub type NodeSessionId = Uuid;

/// 节点任务 ID
pub type NodeTaskId = Uuid;

/// 节点租约 ID
pub type NodeLeaseId = Uuid;

// ============================================================================
// 节点能力
// ============================================================================

/// 节点模型能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeModelCapability {
    /// 模型名称
    pub model: String,
}

/// 节点能力声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapabilities {
    /// 运行时类型（MVP 固定为 "ollama"）
    pub runtime: String,
    /// 支持的模型列表
    pub models: Vec<NodeModelCapability>,
}

// ============================================================================
// 注册协议
// ============================================================================

/// 节点注册请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRegisterRequest {
    /// 协议版本（固定为 "node.v1"）
    pub protocol_version: String,
    /// 客户端实例 ID
    pub client_instance_id: String,
    /// 节点显示名称
    pub display_name: String,
    /// 注册 token
    pub registration_token: String,
    /// 节点能力声明
    pub capabilities: NodeCapabilities,
}

/// 节点注册响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRegisterResponse {
    /// 协议版本
    pub protocol_version: String,
    /// 节点 ID
    pub node_id: NodeId,
    /// 会话 ID
    pub session_id: NodeSessionId,
    /// 会话 token（只返回一次，服务端只保存 hash）
    pub session_token: String,
    /// 心跳间隔（秒）
    pub heartbeat_interval_secs: u64,
    /// 轮询超时（秒）
    pub poll_timeout_secs: u64,
}

// ============================================================================
// 心跳协议
// ============================================================================

/// 节点心跳请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHeartbeatRequest {
    /// 协议版本
    pub protocol_version: String,
    /// 节点 ID
    pub node_id: NodeId,
    /// 会话 ID
    pub session_id: NodeSessionId,
    /// 当前可接受模型列表
    pub accepted_models: Vec<String>,
}

/// 节点心跳响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHeartbeatResponse {
    /// 协议版本
    pub protocol_version: String,
    /// 是否接受（session 与请求体身份校验通过）
    pub accepted: bool,
    /// 节点状态（online/offline/excluded）
    pub node_status: String,
    /// 服务端失败计数
    pub server_failure_count: u32,
    /// 失败阈值
    pub failure_threshold: u32,
}

// ============================================================================
// 轮询协议
// ============================================================================

/// 节点轮询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePollRequest {
    /// 协议版本
    pub protocol_version: String,
    /// 节点 ID
    pub node_id: NodeId,
    /// 会话 ID
    pub session_id: NodeSessionId,
}

/// 节点轮询响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePollResponse {
    /// 协议版本
    pub protocol_version: String,
    /// 任务信封（如果有任务）
    pub task: Option<NodeTaskEnvelope>,
    /// 重试间隔（毫秒）
    pub retry_after_ms: Option<u64>,
}

// ============================================================================
// 任务协议
// ============================================================================

/// 节点任务信封
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTaskEnvelope {
    /// 任务 ID
    pub task_id: NodeTaskId,
    /// 租约 ID
    pub lease_id: NodeLeaseId,
    /// 模型名称（去掉 node: 前缀后的实际模型名）
    pub model: String,
    /// 任务截止时间（Unix 毫秒时间戳）
    pub deadline_unix_ms: i64,
    /// 完成宽限期（Unix 毫秒时间戳）
    pub complete_grace_until_unix_ms: i64,
    /// 任务载荷
    pub payload: NodeTaskPayload,
}

/// 节点任务载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTaskPayload {
    /// 请求 ID
    pub request_id: Uuid,
    /// Chat 完成请求
    pub chat: ChatCompletionRequest,
}

// ============================================================================
// 提交结果协议
// ============================================================================

/// 节点任务完成请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTaskCompleteRequest {
    /// 协议版本
    pub protocol_version: String,
    /// 节点 ID
    pub node_id: NodeId,
    /// 会话 ID
    pub session_id: NodeSessionId,
    /// 任务 ID
    pub task_id: NodeTaskId,
    /// 租约 ID
    pub lease_id: NodeLeaseId,
    /// 任务结果
    pub result: NodeTaskResult,
}

/// 节点任务结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum NodeTaskResult {
    /// 任务成功
    Succeeded {
        /// Chat 完成响应
        response: ChatCompletionResponse,
    },
    /// 任务失败
    Failed {
        /// 错误码
        code: String,
        /// 错误消息
        message: String,
    },
}

/// 节点任务完成响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTaskCompleteResponse {
    /// 执行动作
    pub action: NodeTaskCompleteAction,
    /// 任务状态
    pub task_status: String,
    /// 节点状态
    pub node_status: String,
    /// 服务端失败计数
    pub server_failure_count: u32,
    /// 失败阈值
    pub failure_threshold: u32,
}

/// 节点任务完成动作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeTaskCompleteAction {
    /// 任务成功完成
    Succeeded,
    /// 任务恢复为 queued（重新入队）
    Requeued,
    /// 任务失败
    Failed,
    /// 任务过期
    Expired,
}

// ============================================================================
// OpenAI 兼容的请求/响应类型（用于与 Ollama 交互）
// ============================================================================

/// 消息角色枚举
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    #[default]
    User,
    Assistant,
    Tool,
}

impl MessageRole {
    /// 获取角色字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Tool => "tool",
        }
    }
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

impl Message {
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self::new(MessageRole::System, content)
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new(MessageRole::User, content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(MessageRole::Assistant, content)
    }

    pub fn tool(content: impl Into<String>) -> Self {
        Self::new(MessageRole::Tool, content)
    }
}

/// OpenAI 兼容的请求体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

impl ChatCompletionRequest {
    pub fn new(model: impl Into<String>, messages: Vec<Message>) -> Self {
        Self {
            model: model.into(),
            messages,
            stream: None,
            max_tokens: None,
            temperature: None,
            top_p: None,
            n: None,
            stop: None,
        }
    }
}

/// OpenAI 兼容的非流式响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<CompletionChoice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChoice {
    pub index: u32,
    pub message: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContent {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_role_serialize() {
        let role = MessageRole::Assistant;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"assistant\"");
    }

    #[test]
    fn test_message_role_deserialize() {
        let json = "\"system\"";
        let role: MessageRole = serde_json::from_str(json).unwrap();
        assert_eq!(role, MessageRole::System);
    }

    #[test]
    fn test_chat_completion_request_serialize() {
        let req = ChatCompletionRequest::new(
            "deepseek-chat",
            vec![Message::user("Hello")],
        );
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"model\":\"deepseek-chat\""));
        assert!(json.contains("\"role\":\"user\""));
    }
}
