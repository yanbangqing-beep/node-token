//! Ollama API 类型定义
//!
//! 定义与 Ollama API 交互所需的请求和响应类型。

use serde::{Deserialize, Serialize};

/// Ollama chat 请求
#[derive(Debug, Clone, Serialize)]
pub struct OllamaChatRequest {
    /// 模型名称
    pub model: String,
    /// 消息列表
    pub messages: Vec<OllamaMessage>,
    /// 流式输出（固定为 false）
    pub stream: bool,
}

/// Ollama 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaMessage {
    /// 角色：system, user, assistant
    pub role: String,
    /// 消息内容
    pub content: String,
}

impl OllamaMessage {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }
}

/// Ollama chat 响应
#[derive(Debug, Clone, Deserialize)]
pub struct OllamaChatResponse {
    /// 模型名称
    pub model: String,
    /// 创建时间
    pub created_at: String,
    /// 响应消息
    pub message: OllamaMessage,
    /// 是否完成
    pub done: bool,
    /// 总耗时（纳秒）
    #[serde(default)]
    pub total_duration: u64,
    /// 加载模型耗时（纳秒）
    #[serde(default)]
    pub load_duration: u64,
    /// prompt token 数量
    #[serde(default)]
    pub prompt_eval_count: u32,
    /// 生成 token 数量
    #[serde(default)]
    pub eval_count: u32,
}

/// Ollama 模型列表响应
#[derive(Debug, Clone, Deserialize)]
pub struct OllamaModelListResponse {
    /// 模型列表
    pub models: Vec<OllamaModelInfo>,
}

/// Ollama 模型信息
#[derive(Debug, Clone, Deserialize)]
pub struct OllamaModelInfo {
    /// 模型名称
    pub name: String,
    /// 模型大小
    #[serde(default)]
    pub size: u64,
    /// 修改时间
    #[serde(default)]
    pub modified_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_chat_request_serialize() {
        let req = OllamaChatRequest {
            model: "deepseek-r1".to_string(),
            messages: vec![OllamaMessage::new("user", "Hello")],
            stream: false,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"model\":\"deepseek-r1\""));
        assert!(json.contains("\"stream\":false"));
    }

    #[test]
    fn test_ollama_chat_response_deserialize() {
        let json = r#"{
            "model": "deepseek-r1",
            "created_at": "2024-01-01T00:00:00Z",
            "message": {
                "role": "assistant",
                "content": "Hello!"
            },
            "done": true,
            "total_duration": 1000000000,
            "prompt_eval_count": 10,
            "eval_count": 20
        }"#;
        let resp: OllamaChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.model, "deepseek-r1");
        assert_eq!(resp.message.content, "Hello!");
        assert_eq!(resp.prompt_eval_count, 10);
        assert_eq!(resp.eval_count, 20);
    }

    #[test]
    fn test_ollama_model_list_deserialize() {
        let json = r#"{
            "models": [
                {"name": "deepseek-r1:latest", "size": 4000000000},
                {"name": "llama2:latest", "size": 3800000000}
            ]
        }"#;
        let resp: OllamaModelListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.models.len(), 2);
        assert_eq!(resp.models[0].name, "deepseek-r1:latest");
    }
}
