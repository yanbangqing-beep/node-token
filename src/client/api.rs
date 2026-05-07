//! KeyCompute API 客户端
//!
//! 负责与 KeyCompute 服务端通信，包括注册、心跳、轮询和任务完成。

use crate::error::{NetworkResult, NodeTokenError};
use crate::protocol::types::{
    NodeHeartbeatRequest, NodeHeartbeatResponse, NodePollRequest, NodePollResponse,
    NodeRegisterRequest, NodeRegisterResponse, NodeTaskCompleteRequest, NodeTaskCompleteResponse,
};
use reqwest::Client;
use tracing::{debug, error, info};

/// KeyCompute API 客户端
#[allow(dead_code)] // 在后续阶段使用
pub struct KeyComputeClient {
    /// 服务端基础 URL
    base_url: String,
    /// HTTP 客户端（连接池）
    http_client: Client,
    /// Session token（注册后设置）
    session_token: Option<String>,
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
            session_token: None,
        }
    }

    /// 设置 session token（注册后调用）
    #[allow(dead_code)] // 在后续阶段使用
    pub fn set_session_token(&mut self, token: String) {
        self.session_token = Some(token);
    }

    /// 获取 session token
    #[allow(dead_code)] // 在后续阶段使用
    pub fn session_token(&self) -> Option<&str> {
        self.session_token.as_deref()
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
                format!("Bearer {}", self.require_session_token()?),
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
                format!("Bearer {}", self.require_session_token()?),
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
                format!("Bearer {}", self.require_session_token()?),
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
    fn require_session_token(&self) -> Result<&str, NodeTokenError> {
        self.session_token
            .as_deref()
            .ok_or_else(|| NodeTokenError::InvalidSession)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = KeyComputeClient::new("http://localhost:3000");
        assert_eq!(client.base_url, "http://localhost:3000");
        assert!(client.session_token.is_none());
    }

    #[test]
    fn test_set_session_token() {
        let mut client = KeyComputeClient::new("http://localhost:3000");
        assert!(client.session_token.is_none());

        client.set_session_token("test-token".to_string());
        assert_eq!(client.session_token(), Some("test-token"));
    }

    #[test]
    fn test_require_session_token() {
        let mut client = KeyComputeClient::new("http://localhost:3000");

        // 未设置 token 时应该返回错误
        assert!(client.require_session_token().is_err());

        // 设置 token 后应该返回成功
        client.set_session_token("test-token".to_string());
        assert!(client.require_session_token().is_ok());
        assert_eq!(client.require_session_token().unwrap(), "test-token");
    }
}
