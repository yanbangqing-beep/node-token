/// 任务执行器
///
/// 负责执行从服务端领取的任务，调用本机 Ollama 完成推理，
/// 并将结果提交回服务端。
use std::sync::Arc;

use chrono::Utc;
use tokio_retry::strategy::ExponentialBackoff;
use tokio_retry::Retry;
use tracing::{error, info, warn};

use crate::client::{KeyComputeClient, OllamaClient};
use crate::error::NodeTokenError;
use crate::protocol::ollama::OllamaChatRequest;
use crate::protocol::types::{
    NodeTaskCompleteRequest, NodeTaskCompleteResponse, NodeTaskEnvelope, NodeTaskResult,
};
use crate::storage::SessionData;

/// 结果类型别名
#[allow(dead_code)] // 在后续阶段使用
type Result<T> = std::result::Result<T, NodeTokenError>;

/// 任务执行器
///
/// 执行领取的任务，调用 Ollama 推理，提交结果。
#[allow(dead_code)] // 在阶段五使用
pub struct TaskExecutor {
    /// KeyCompute HTTP 客户端
    client: Arc<KeyComputeClient>,
    /// Ollama HTTP 客户端
    ollama_client: Arc<OllamaClient>,
    /// 当前 session 信息
    session: SessionData,
    /// 镜像的服务端节点状态
    node_status: Arc<tokio::sync::Mutex<String>>,
    /// 镜像的服务端失败计数
    server_failure_count: Arc<tokio::sync::Mutex<u32>>,
    /// 镜像的服务端失败阈值
    failure_threshold: Arc<tokio::sync::Mutex<u32>>,
}

impl TaskExecutor {
    /// 创建新的任务执行器
    #[allow(dead_code)] // 在阶段五使用
    pub fn new(
        client: Arc<KeyComputeClient>,
        ollama_client: Arc<OllamaClient>,
        session: SessionData,
    ) -> Self {
        Self {
            client,
            ollama_client,
            session,
            node_status: Arc::new(tokio::sync::Mutex::new("unknown".to_string())),
            server_failure_count: Arc::new(tokio::sync::Mutex::new(0)),
            failure_threshold: Arc::new(tokio::sync::Mutex::new(3)),
        }
    }

    /// 执行单个任务
    ///
    /// # 流程
    /// 1. 从 envelope 中提取任务信息
    /// 2. 将任务转换为 Ollama 请求
    /// 3. 调用 Ollama 执行推理
    /// 4. 将结果转换为 NodeTaskResult
    /// 5. 提交结果到服务端（带重试）
    #[allow(dead_code)] // 在阶段五使用
    pub async fn execute(&self, envelope: NodeTaskEnvelope) {
        let task_id = envelope.task_id;
        let lease_id = envelope.lease_id;
        let deadline_ms = envelope.deadline_unix_ms;
        let grace_until_ms = envelope.complete_grace_until_unix_ms;

        info!(
            "Executing task: task_id={}, model={}, deadline_ms={}, grace_until_ms={}",
            task_id, envelope.model, deadline_ms, grace_until_ms
        );

        // 1. 调用 Ollama 执行任务
        let result = match self.execute_ollama(&envelope).await {
            Ok(response) => {
                info!("Task {} executed successfully", task_id);
                NodeTaskResult::Succeeded { response }
            }
            Err(e) => {
                error!("Task {} execution failed: {}", task_id, e);
                NodeTaskResult::Failed {
                    code: "ollama_error".to_string(),
                    message: e.to_string(),
                }
            }
        };

        // 2. 提交结果（带重试，根据 deadline 和 grace period 控制）
        self.complete_with_retry(task_id, lease_id, result, deadline_ms, grace_until_ms)
            .await;
    }

    /// 调用 Ollama 执行推理
    ///
    /// 将 NodeTaskPayload 转换为 Ollama 请求，调用 Ollama API，
    /// 将响应转换为 ChatCompletionResponse。
    #[allow(dead_code)] // 在阶段五使用
    async fn execute_ollama(
        &self,
        envelope: &NodeTaskEnvelope,
    ) -> Result<crate::protocol::types::ChatCompletionResponse> {
        let chat_req = &envelope.payload.chat;

        // 转换为 Ollama 请求格式
        let ollama_req = self.chat_request_to_ollama(chat_req, &envelope.model);

        // 调用 Ollama API
        let ollama_resp = self.ollama_client.chat(&ollama_req).await?;

        // 转换 Ollama 响应为 ChatCompletionResponse
        Ok(self.ollama_response_to_chat(&ollama_resp, &envelope.model))
    }

    /// 将 ChatCompletionRequest 转换为 Ollama 请求格式
    #[allow(dead_code)] // 在阶段五使用
    fn chat_request_to_ollama(
        &self,
        chat_req: &crate::protocol::types::ChatCompletionRequest,
        model: &str,
    ) -> OllamaChatRequest {
        // 转换 messages 格式
        let messages: Vec<crate::protocol::ollama::OllamaMessage> = chat_req
            .messages
            .iter()
            .map(|msg| crate::protocol::ollama::OllamaMessage {
                role: msg.role.to_string(),
                content: msg.content.clone(),
            })
            .collect();

        OllamaChatRequest {
            model: model.to_string(),
            messages,
            stream: false, // MVP 只支持非流式
        }
    }

    /// 将 Ollama 响应转换为 ChatCompletionResponse
    #[allow(dead_code)] // 在阶段五使用
    fn ollama_response_to_chat(
        &self,
        ollama_resp: &crate::protocol::ollama::OllamaChatResponse,
        model: &str,
    ) -> crate::protocol::types::ChatCompletionResponse {
        use crate::protocol::types::{
            ChatCompletionResponse, CompletionChoice, MessageContent, Usage,
        };
        use uuid::Uuid;

        // 生成唯一的 choice id
        let choice_id = format!("chatcmpl-{}", Uuid::new_v4().simple());

        ChatCompletionResponse {
            id: choice_id,
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model.to_string(),
            choices: vec![CompletionChoice {
                index: 0,
                message: MessageContent {
                    role: "assistant".to_string(),
                    content: ollama_resp.message.content.clone(),
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Usage {
                prompt_tokens: ollama_resp.prompt_eval_count,
                completion_tokens: ollama_resp.eval_count,
                total_tokens: ollama_resp.prompt_eval_count + ollama_resp.eval_count,
            },
        }
    }

    /// 提交结果到服务端（带重试）
    ///
    /// - deadline 前积极重试
    /// - deadline 后到 grace_until 前，仍可重试获取 ACK
    /// - grace_until 后不再重试（但服务端已有 submission 时仍可获取 ACK）
    #[allow(dead_code)] // 在阶段五使用
    async fn complete_with_retry(
        &self,
        task_id: uuid::Uuid,
        lease_id: uuid::Uuid,
        result: NodeTaskResult,
        deadline_ms: i64,
        grace_until_ms: i64,
    ) {
        let req = NodeTaskCompleteRequest {
            protocol_version: "node.v1".to_string(),
            node_id: self.session.node_id,
            session_id: self.session.session_id,
            task_id,
            lease_id,
            result,
        };

        let now_ms = Utc::now().timestamp_millis();

        // 确定重试截止时间
        let retry_deadline = if now_ms < deadline_ms {
            deadline_ms
        } else {
            grace_until_ms
        };

        let max_retry_duration = std::cmp::max(0, retry_deadline - now_ms);

        if max_retry_duration <= 0 {
            // 已经超过 grace period，尝试一次提交（可能命中幂等路径）
            warn!(
                "Task {} past grace period, attempting one-shot complete",
                task_id
            );
            match self.client.complete(task_id, &req).await {
                Ok(resp) => {
                    info!(
                        "Task {} completed (one-shot): action={:?}",
                        task_id, resp.action
                    );

                    // 镜像服务端状态
                    *self.node_status.lock().await = resp.node_status.clone();
                    *self.server_failure_count.lock().await = resp.server_failure_count;
                    *self.failure_threshold.lock().await = resp.failure_threshold;

                    self.log_complete_response(task_id, &resp);
                }
                Err(e) => {
                    error!("Task {} one-shot complete failed: {}", task_id, e);
                }
            }
            return;
        }

        // 使用指数退避重试，最大时长为到 retry_deadline 的剩余时间
        let max_retries =
            std::cmp::max(1, (max_retry_duration as f64 / 1000.0).ceil() as usize / 5);

        let retry_strategy = ExponentialBackoff::from_millis(100)
            .max_delay(std::time::Duration::from_secs(5))
            .take(max_retries);

        info!(
            "Starting complete retry for task {}: max_duration={}ms, max_retries={}",
            task_id, max_retry_duration, max_retries
        );

        match Retry::spawn(retry_strategy, || async {
            match self.client.complete(task_id, &req).await {
                Ok(resp) => {
                    info!("Task {} completed: action={:?}", task_id, resp.action);

                    // 镜像服务端状态
                    *self.node_status.lock().await = resp.node_status.clone();
                    *self.server_failure_count.lock().await = resp.server_failure_count;
                    *self.failure_threshold.lock().await = resp.failure_threshold;

                    self.log_complete_response(task_id, &resp);
                    Ok(resp)
                }
                Err(e) => {
                    warn!("Complete failed for task {}: {}", task_id, e);
                    // 网络错误不增加失败计数
                    Err(e)
                }
            }
        })
        .await
        {
            Ok(_) => {
                info!("Task {} complete succeeded", task_id);
            }
            Err(e) => {
                error!("Task {} complete failed after retries: {}", task_id, e);
            }
        }
    }

    /// 记录 complete 响应信息
    #[allow(dead_code)] // 在阶段五使用
    fn log_complete_response(&self, task_id: uuid::Uuid, resp: &NodeTaskCompleteResponse) {
        info!(
            "Complete response for task {}: action={:?}, task_status={}, node_status={}, failure_count={}/{}",
            task_id,
            resp.action,
            resp.task_status,
            resp.node_status,
            resp.server_failure_count,
            resp.failure_threshold
        );

        // 如果节点被 excluded，发出警告
        if resp.node_status == "excluded" {
            warn!(
                "Node EXCLUDED after task {} complete, will stop poll but continue heartbeat",
                task_id
            );
        }
    }
}

// 实现 Clone 以便在 tokio::spawn 中使用
impl Clone for TaskExecutor {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            ollama_client: self.ollama_client.clone(),
            session: self.session.clone(),
            node_status: self.node_status.clone(),
            server_failure_count: self.server_failure_count.clone(),
            failure_threshold: self.failure_threshold.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::{ChatCompletionRequest, Message, MessageRole};

    #[test]
    fn test_chat_request_to_ollama_conversion() {
        let chat_req = ChatCompletionRequest {
            model: "test-model".to_string(),
            messages: vec![
                Message {
                    role: MessageRole::System,
                    content: "You are a helpful assistant.".to_string(),
                },
                Message {
                    role: MessageRole::User,
                    content: "Hello!".to_string(),
                },
            ],
            stream: Some(false),
            max_tokens: None,
            temperature: None,
            top_p: None,
            n: None,
            stop: None,
        };

        let executor = create_test_executor();
        let ollama_req = executor.chat_request_to_ollama(&chat_req, "test-model");

        assert_eq!(ollama_req.model, "test-model");
        assert_eq!(ollama_req.messages.len(), 2);
        assert_eq!(ollama_req.messages[0].role, "system");
        assert_eq!(ollama_req.messages[1].role, "user");
        assert!(!ollama_req.stream);
    }

    fn create_test_executor() -> TaskExecutor {
        use crate::client::{KeyComputeClient, OllamaClient};
        use crate::protocol::types::NodeCapabilities;
        use crate::storage::SessionData;
        use std::sync::Arc;

        let client = Arc::new(KeyComputeClient::new("http://localhost:3000"));
        let ollama_client = Arc::new(OllamaClient::new("http://localhost:11434"));
        let session = SessionData {
            node_id: uuid::Uuid::new_v4(),
            session_id: uuid::Uuid::new_v4(),
            session_token: "test-token".to_string(),
            capabilities: NodeCapabilities {
                runtime: "ollama".to_string(),
                models: vec![],
            },
            poll_timeout_secs: 30,
        };

        TaskExecutor::new(client, ollama_client, session)
    }
}
