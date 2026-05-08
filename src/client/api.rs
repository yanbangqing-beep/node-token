//! KeyCompute API 客户端
//!
//! 负责与 KeyCompute 服务端通信，包括注册、心跳、轮询和任务完成。

use crate::error::{NetworkResult, NodeTokenError};
use crate::protocol::types::{
    NodeHeartbeatRequest, NodeHeartbeatResponse, NodePollRequest, NodePollResponse,
    NodeRegisterRequest, NodeRegisterResponse, NodeTaskCompleteRequest, NodeTaskCompleteResponse,
};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// KeyCompute API 客户端
#[allow(dead_code)] // 在后续阶段使用
pub struct KeyComputeClient {
    /// 服务端基础 URL
    base_url: String,
    /// HTTP 客户端（连接池）
    http_client: Client,
    /// Session token（注册后设置，使用 RwLock 支持内部可变性）
    session_token: Arc<RwLock<Option<String>>>,
}

impl KeyComputeClient {
    /// 创建新的 KeyCompute 客户端
    #[allow(dead_code)] // 在后续阶段使用
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // 5 分钟超时（支持长轮询）
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url,
            http_client,
            session_token: Arc::new(RwLock::new(None)),
        }
    }

    /// 创建新的 KeyCompute 客户端（带初始 session token）
    #[allow(dead_code)] // 在后续阶段使用
    pub fn new_with_token(base_url: impl Into<String>, token: String) -> Self {
        let base_url = base_url.into();
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // 5 分钟超时（支持长轮询）
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url,
            http_client,
            session_token: Arc::new(RwLock::new(Some(token))),
        }
    }

    /// 设置 session token（注册后调用）
    #[allow(dead_code)] // 在后续阶段使用
    pub async fn set_session_token(&self, token: String) {
        let mut token_guard = self.session_token.write().await;
        *token_guard = Some(token);
    }

    /// 获取 session token
    #[allow(dead_code)] // 在后续阶段使用
    pub async fn get_session_token(&self) -> Option<String> {
        let token_guard = self.session_token.read().await;
        token_guard.clone()
    }

    /// 节点注册
    /// POST /node/v1/register
    /// 不需要 session token 认证
    #[allow(dead_code)] // 在后续阶段使用
    pub async fn register(
        &self,
        request: &NodeRegisterRequest,
    ) -> NetworkResult<NodeRegisterResponse> {
        let url = format!("{}/node/v1/register", self.base_url);

        info!("Registering node with server");
        debug!("Register request: {:?}", request);

        let response = self
            .http_client
            .post(&url)
            .json(request)
            .send()
            .await
            .map_err(|e| {
                error!("Register request failed: {}", e);
                NodeTokenError::Network(e)
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            error!("Register failed with status {}: {}", status, body);
            return Err(NodeTokenError::HttpError {
                status,
                message: format!("Register failed: {}", body),
            });
        }

        let response_body: NodeRegisterResponse = response.json().await.map_err(|e| {
            error!("Failed to parse register response: {}", e);
            NodeTokenError::Network(e)
        })?;

        info!(
            "Node registered successfully: node_id={}",
            response_body.node_id
        );
        Ok(response_body)
    }

    /// 节点心跳
    /// POST /node/v1/heartbeat
    /// 需要 session token 认证
    #[allow(dead_code)] // 在后续阶段使用
    pub async fn heartbeat(
        &self,
        request: &NodeHeartbeatRequest,
    ) -> NetworkResult<NodeHeartbeatResponse> {
        let url = format!("{}/node/v1/heartbeat", self.base_url);

        debug!("Sending heartbeat");

        let response = self
            .http_client
            .post(&url)
            .json(request)
            .header(
                "Authorization",
                format!("Bearer {}", self.require_session_token().await?),
            )
            .send()
            .await
            .map_err(|e| {
                error!("Heartbeat request failed: {}", e);
                NodeTokenError::Network(e)
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            error!("Heartbeat failed with status {}: {}", status, body);
            return Err(NodeTokenError::HttpError {
                status,
                message: format!("Heartbeat failed: {}", body),
            });
        }

        let response_body: NodeHeartbeatResponse = response.json().await.map_err(|e| {
            error!("Failed to parse heartbeat response: {}", e);
            NodeTokenError::Network(e)
        })?;

        debug!(
            "Heartbeat response: accepted={}, status={}",
            response_body.accepted, response_body.node_status
        );
        Ok(response_body)
    }

    /// 任务轮询（长轮询）
    /// POST /node/v1/tasks/poll
    /// 需要 session token 认证
    #[allow(dead_code)] // 在后续阶段使用
    pub async fn poll(&self, request: &NodePollRequest) -> NetworkResult<NodePollResponse> {
        let url = format!("{}/node/v1/tasks/poll", self.base_url);

        debug!("Polling for tasks");

        let response = self
            .http_client
            .post(&url)
            .json(request)
            .header(
                "Authorization",
                format!("Bearer {}", self.require_session_token().await?),
            )
            .send()
            .await
            .map_err(|e| {
                error!("Poll request failed: {}", e);
                NodeTokenError::Network(e)
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            error!("Poll failed with status {}: {}", status, body);
            return Err(NodeTokenError::HttpError {
                status,
                message: format!("Poll failed: {}", body),
            });
        }

        let response_body: NodePollResponse = response.json().await.map_err(|e| {
            error!("Failed to parse poll response: {}", e);
            NodeTokenError::Network(e)
        })?;

        if response_body.task.is_some() {
            info!("Received task from poll");
        } else {
            debug!("No task available from poll");
        }
        Ok(response_body)
    }

    /// 完成任务
    /// POST /node/v1/tasks/{task_id}/complete
    /// 需要 session token 认证
    #[allow(dead_code)] // 在后续阶段使用
    pub async fn complete(
        &self,
        task_id: uuid::Uuid,
        request: &NodeTaskCompleteRequest,
    ) -> NetworkResult<NodeTaskCompleteResponse> {
        let url = format!("{}/node/v1/tasks/{}/complete", self.base_url, task_id);

        debug!("Completing task: {}", task_id);

        let response = self
            .http_client
            .post(&url)
            .json(request)
            .header(
                "Authorization",
                format!("Bearer {}", self.require_session_token().await?),
            )
            .send()
            .await
            .map_err(|e| {
                error!("Complete request failed for task {}: {}", task_id, e);
                NodeTokenError::Network(e)
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            error!(
                "Complete failed for task {} with status {}: {}",
                task_id, status, body
            );
            return Err(NodeTokenError::HttpError {
                status,
                message: format!("Complete failed: {}", body),
            });
        }

        let response_body: NodeTaskCompleteResponse = response.json().await.map_err(|e| {
            error!(
                "Failed to parse complete response for task {}: {}",
                task_id, e
            );
            NodeTokenError::Network(e)
        })?;

        info!(
            "Task {} completed: action={:?}, node_status={}",
            task_id, response_body.action, response_body.node_status
        );
        Ok(response_body)
    }

    /// 获取 session token，如果不存在则返回错误
    async fn require_session_token(&self) -> Result<Arc<String>, NodeTokenError> {
        let token_guard = self.session_token.read().await;
        token_guard
            .clone()
            .map(Arc::new)
            .ok_or_else(|| NodeTokenError::InvalidSession)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_client_creation() {
        let client = KeyComputeClient::new("http://localhost:3000");
        assert_eq!(client.base_url, "http://localhost:3000");
        assert!(client.get_session_token().await.is_none());
    }

    #[tokio::test]
    async fn test_set_session_token() {
        let client = KeyComputeClient::new("http://localhost:3000");
        assert!(client.get_session_token().await.is_none());

        client.set_session_token("test-token".to_string()).await;
        assert_eq!(
            client.get_session_token().await,
            Some("test-token".to_string())
        );
    }

    #[tokio::test]
    async fn test_require_session_token() {
        let client = KeyComputeClient::new("http://localhost:3000");

        // 未设置 token 时应该返回错误
        assert!(client.require_session_token().await.is_err());

        // 设置 token 后应该返回成功
        client.set_session_token("test-token".to_string()).await;
        let token = client.require_session_token().await.unwrap();
        assert_eq!(*token, "test-token");
    }

    #[tokio::test]
    /// 测试节点注册成功场景
    async fn test_register_success() {
        let mock_server = MockServer::start().await;

        // Mock 注册响应
        Mock::given(method("POST"))
            .and(path("/node/v1/register"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "protocol_version": "node.v1",
                "node_id": "00000000-0000-0000-0000-000000000001",
                "session_id": "00000000-0000-0000-0000-000000000002",
                "session_token": "test-session-token",
                "heartbeat_interval_secs": 30,
                "poll_timeout_secs": 10
            })))
            .mount(&mock_server)
            .await;

        let client = KeyComputeClient::new(mock_server.uri());
        let request = crate::protocol::types::NodeRegisterRequest {
            protocol_version: "node.v1".to_string(),
            client_instance_id: "test-instance".to_string(),
            display_name: "Test Node".to_string(),
            registration_token: "test-token".to_string(),
            capabilities: crate::protocol::types::NodeCapabilities {
                runtime: "ollama".to_string(),
                models: vec![crate::protocol::types::NodeModelCapability {
                    model: "deepseek-chat".to_string(),
                }],
            },
        };

        let response = client.register(&request).await.unwrap();
        assert_eq!(
            response.node_id.to_string(),
            "00000000-0000-0000-0000-000000000001"
        );
        assert_eq!(response.session_token, "test-session-token");
        assert_eq!(response.heartbeat_interval_secs, 30);
        assert_eq!(response.poll_timeout_secs, 10);
    }

    #[tokio::test]
    /// 测试节点注册失败场景（HTTP 500）
    async fn test_register_http_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/node/v1/register"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let client = KeyComputeClient::new(mock_server.uri());
        let request = crate::protocol::types::NodeRegisterRequest {
            protocol_version: "node.v1".to_string(),
            client_instance_id: "test-instance".to_string(),
            display_name: "Test Node".to_string(),
            registration_token: "test-token".to_string(),
            capabilities: crate::protocol::types::NodeCapabilities {
                runtime: "ollama".to_string(),
                models: vec![],
            },
        };

        let result = client.register(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    /// 测试心跳成功场景
    async fn test_heartbeat_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/node/v1/heartbeat"))
            .and(header("Authorization", "Bearer test-session-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "protocol_version": "node.v1",
                "accepted": true,
                "node_status": "online",
                "server_failure_count": 0,
                "failure_threshold": 3
            })))
            .mount(&mock_server)
            .await;

        let client =
            KeyComputeClient::new_with_token(mock_server.uri(), "test-session-token".to_string());
        let request = crate::protocol::types::NodeHeartbeatRequest {
            protocol_version: "node.v1".to_string(),
            node_id: uuid::Uuid::new_v4(),
            session_id: uuid::Uuid::new_v4(),
            accepted_models: vec!["deepseek-chat".to_string()],
        };

        let response = client.heartbeat(&request).await.unwrap();
        assert!(response.accepted);
        assert_eq!(response.node_status, "online");
        assert_eq!(response.server_failure_count, 0);
        assert_eq!(response.failure_threshold, 3);
    }

    #[tokio::test]
    /// 测试心跳缺少 token 时返回错误
    async fn test_heartbeat_missing_token() {
        let client = KeyComputeClient::new("http://localhost:3000");
        let request = crate::protocol::types::NodeHeartbeatRequest {
            protocol_version: "node.v1".to_string(),
            node_id: uuid::Uuid::new_v4(),
            session_id: uuid::Uuid::new_v4(),
            accepted_models: vec![],
        };

        let result = client.heartbeat(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    /// 测试轮询成功场景（有任务）
    async fn test_poll_with_task() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/node/v1/tasks/poll"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "protocol_version": "node.v1",
                "task": {
                    "task_id": "00000000-0000-0000-0000-000000000003",
                    "lease_id": "00000000-0000-0000-0000-000000000004",
                    "model": "deepseek-chat",
                    "deadline_unix_ms": 9999999999999_i64,
                    "complete_grace_until_unix_ms": 9999999999999_i64,
                    "payload": {
                        "request_id": "00000000-0000-0000-0000-000000000005",
                        "chat": {
                            "model": "deepseek-chat",
                            "messages": [{"role": "user", "content": "Hello"}],
                            "stream": false
                        }
                    }
                },
                "retry_after_ms": null
            })))
            .mount(&mock_server)
            .await;

        let client = KeyComputeClient::new_with_token(mock_server.uri(), "test-token".to_string());
        let request = crate::protocol::types::NodePollRequest {
            protocol_version: "node.v1".to_string(),
            node_id: uuid::Uuid::new_v4(),
            session_id: uuid::Uuid::new_v4(),
        };

        let response = client.poll(&request).await.unwrap();
        assert!(response.task.is_some());
        let task = response.task.unwrap();
        assert_eq!(task.model, "deepseek-chat");
    }

    #[tokio::test]
    /// 测试轮询成功场景（无任务）
    async fn test_poll_no_task() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/node/v1/tasks/poll"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "protocol_version": "node.v1",
                "task": null,
                "retry_after_ms": 1000
            })))
            .mount(&mock_server)
            .await;

        let client = KeyComputeClient::new_with_token(mock_server.uri(), "test-token".to_string());
        let request = crate::protocol::types::NodePollRequest {
            protocol_version: "node.v1".to_string(),
            node_id: uuid::Uuid::new_v4(),
            session_id: uuid::Uuid::new_v4(),
        };

        let response = client.poll(&request).await.unwrap();
        assert!(response.task.is_none());
        assert_eq!(response.retry_after_ms, Some(1000));
    }

    #[tokio::test]
    /// 测试任务完成成功场景
    async fn test_complete_success() {
        let mock_server = MockServer::start().await;
        let task_id = uuid::Uuid::new_v4();

        Mock::given(method("POST"))
            .and(path(format!("/node/v1/tasks/{}/complete", task_id)))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "action": "succeeded",
                "task_status": "succeeded",
                "node_status": "online",
                "server_failure_count": 0,
                "failure_threshold": 3
            })))
            .mount(&mock_server)
            .await;

        let client = KeyComputeClient::new_with_token(mock_server.uri(), "test-token".to_string());
        let request = crate::protocol::types::NodeTaskCompleteRequest {
            protocol_version: "node.v1".to_string(),
            node_id: uuid::Uuid::new_v4(),
            session_id: uuid::Uuid::new_v4(),
            task_id,
            lease_id: uuid::Uuid::new_v4(),
            result: crate::protocol::types::NodeTaskResult::Succeeded {
                response: crate::protocol::types::ChatCompletionResponse {
                    id: "resp-001".to_string(),
                    object: "chat.completion".to_string(),
                    created: 1234567890,
                    model: "deepseek-chat".to_string(),
                    choices: vec![crate::protocol::types::CompletionChoice {
                        index: 0,
                        message: crate::protocol::types::MessageContent {
                            role: "assistant".to_string(),
                            content: "Hello!".to_string(),
                        },
                        finish_reason: Some("stop".to_string()),
                    }],
                    usage: crate::protocol::types::Usage {
                        prompt_tokens: 10,
                        completion_tokens: 20,
                        total_tokens: 30,
                    },
                },
            },
        };

        let response = client.complete(task_id, &request).await.unwrap();
        assert_eq!(
            response.action,
            crate::protocol::types::NodeTaskCompleteAction::Succeeded
        );
        assert_eq!(response.node_status, "online");
    }
}
