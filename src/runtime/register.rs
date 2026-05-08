/// 节点注册逻辑
///
/// 负责扫描 Ollama 模型、构建注册请求、调用注册 API 并持久化 session。
use tracing::{debug, info, warn};

use crate::client::{KeyComputeClient, OllamaClient};
use crate::config::NodeTokenConfig;
use crate::error::NodeTokenError;
use crate::protocol::types::{
    NodeCapabilities, NodeModelCapability, NodeRegisterRequest, NodeRegisterResponse,
};
use crate::storage::{LocalStorage, SessionData};

/// 结果类型别名
#[allow(dead_code)] // 在阶段五使用
pub type Result<T> = std::result::Result<T, NodeTokenError>;

/// 注册节点到 KeyCompute 服务端
///
/// # 流程
/// 1. 扫描本机 Ollama 模型列表
/// 2. 构建注册请求（runtime="ollama" + 模型列表）
/// 3. 调用服务端注册 API
/// 4. 持久化 session 信息到本地
///
/// # 返回
/// 注册成功返回 `NodeRegisterResponse`，包含 node_id、session_id、session_token 等
///
/// # 错误
/// - Ollama 扫描失败
/// - 注册 API 调用失败
/// - Session 持久化失败
#[allow(dead_code)] // 在阶段五使用
pub async fn register_node(
    client: &KeyComputeClient,
    ollama_client: &OllamaClient,
    config: &NodeTokenConfig,
    storage: &LocalStorage,
) -> Result<NodeRegisterResponse> {
    info!("Starting node registration...");

    // 1. 扫描 Ollama 模型
    debug!("Scanning Ollama models...");
    let models: Vec<String> = ollama_client.list_models().await?;

    if models.is_empty() {
        warn!("No Ollama models found on this machine");
    } else {
        info!("Found {} Ollama models: {:?}", models.len(), models);
    }

    // 2. 构建注册请求
    let req = NodeRegisterRequest {
        protocol_version: "node.v1".to_string(),
        client_instance_id: config.client_instance_id.clone(),
        display_name: config.display_name.clone(),
        registration_token: config.registration_token.clone(),
        capabilities: NodeCapabilities {
            runtime: "ollama".to_string(),
            models: models
                .into_iter()
                .map(|m| NodeModelCapability { model: m })
                .collect(),
        },
    };

    debug!(
        "Registration request: client_instance_id={}, display_name={}, runtime=ollama, models_count={}",
        req.client_instance_id,
        req.display_name,
        req.capabilities.models.len()
    );

    // 3. 调用注册 API
    info!("Calling register API...");
    let resp = client.register(&req).await?;

    // 4. 持久化 session
    let session = SessionData {
        node_id: resp.node_id,
        session_id: resp.session_id,
        session_token: resp.session_token.clone(),
        capabilities: req.capabilities.clone(), // 使用注册请求中的 capabilities
        poll_timeout_secs: resp.poll_timeout_secs, // 保存服务端返回的 poll 超时
    };

    storage.save_session(&session)?;

    info!(
        "Registration successful: node_id={}, session_id={}, heartbeat_interval={}s, poll_timeout={}s",
        resp.node_id, resp.session_id, resp.heartbeat_interval_secs, resp.poll_timeout_secs
    );

    // 注意：日志中不得输出 session_token 明文
    debug!("Session token saved to local storage (not logged for security)");

    Ok(resp)
}

/// 尝试加载本地 session
///
/// 如果本地存在有效的 session，则直接返回，跳过注册流程。
/// “重启后优先复用会话继续 heartbeat/poll”。
///
/// # 返回
/// - `Some(SessionData)`: 找到本地 session
/// - `None`: 无本地 session，需要执行新注册
#[allow(dead_code)] // 在阶段五使用
pub fn try_load_session(storage: &LocalStorage) -> Result<Option<SessionData>> {
    debug!("Attempting to load session from local storage...");

    match storage.load_session()? {
        Some(session) => {
            info!(
                "Loaded existing session: node_id={}, session_id={}",
                session.node_id, session.session_id
            );
            debug!(
                "Session capabilities: runtime={}, models_count={}",
                session.capabilities.runtime,
                session.capabilities.models.len()
            );
            Ok(Some(session))
        }
        None => {
            debug!("No existing session found, will register new node");
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    /// 验证无本地 session 时应返回 None，需要执行新注册
    fn test_try_load_session_no_existing() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalStorage::new(Some(temp_dir.path().to_str().unwrap())).unwrap();

        let result = try_load_session(&storage).unwrap();
        assert!(result.is_none());
    }

    #[test]
    /// 验证有本地 session 时应正确加载并返回 session 信息
    fn test_try_load_session_existing() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalStorage::new(Some(temp_dir.path().to_str().unwrap())).unwrap();

        // 创建一个测试 session
        let session = SessionData {
            node_id: uuid::Uuid::new_v4(),
            session_id: uuid::Uuid::new_v4(),
            session_token: "test-token".to_string(),
            capabilities: NodeCapabilities {
                runtime: "ollama".to_string(),
                models: vec![NodeModelCapability {
                    model: "test-model".to_string(),
                }],
            },
            poll_timeout_secs: 30,
        };

        storage.save_session(&session).unwrap();

        let result = try_load_session(&storage).unwrap();
        assert!(result.is_some());

        let loaded = result.unwrap();
        assert_eq!(loaded.node_id, session.node_id);
        assert_eq!(loaded.session_id, session.session_id);
        assert_eq!(loaded.capabilities.runtime, "ollama");
        assert_eq!(loaded.capabilities.models.len(), 1);
    }

    #[test]
    /// 验证注册请求构建逻辑：从配置和 Ollama 模型列表构建完整的注册请求
    fn test_register_request_building() {
        // 测试注册请求的数据结构构建
        let config = NodeTokenConfig {
            server_url: "http://localhost:3000".to_string(),
            registration_token: "test-token".to_string(),
            client_instance_id: "test-instance-001".to_string(),
            display_name: "Test Node".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            heartbeat_interval_secs: 30,
            excluded_poll_check_interval_secs: 30,
            data_dir: None,
        };

        let models = vec![
            "deepseek-chat:latest".to_string(),
            "llama3:latest".to_string(),
        ];

        // 构建注册请求（模拟 register_node 中的逻辑）
        let req = NodeRegisterRequest {
            protocol_version: "node.v1".to_string(),
            client_instance_id: config.client_instance_id.clone(),
            display_name: config.display_name.clone(),
            registration_token: config.registration_token.clone(),
            capabilities: NodeCapabilities {
                runtime: "ollama".to_string(),
                models: models
                    .into_iter()
                    .map(|m| NodeModelCapability { model: m })
                    .collect(),
            },
        };

        // 验证请求字段
        assert_eq!(req.protocol_version, "node.v1");
        assert_eq!(req.client_instance_id, "test-instance-001");
        assert_eq!(req.display_name, "Test Node");
        assert_eq!(req.registration_token, "test-token");
        assert_eq!(req.capabilities.runtime, "ollama");
        assert_eq!(req.capabilities.models.len(), 2);
        assert_eq!(req.capabilities.models[0].model, "deepseek-chat:latest");
        assert_eq!(req.capabilities.models[1].model, "llama3:latest");
    }

    #[test]
    /// 验证 session 持久化逻辑：注册成功后正确保存 session 信息
    fn test_session_persistence_after_register() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalStorage::new(Some(temp_dir.path().to_str().unwrap())).unwrap();

        // 模拟注册响应
        let resp = NodeRegisterResponse {
            protocol_version: "node.v1".to_string(),
            node_id: uuid::Uuid::new_v4(),
            session_id: uuid::Uuid::new_v4(),
            session_token: "new-session-token".to_string(),
            heartbeat_interval_secs: 30,
            poll_timeout_secs: 10,
        };

        // 模拟注册请求中的 capabilities
        let capabilities = NodeCapabilities {
            runtime: "ollama".to_string(),
            models: vec![NodeModelCapability {
                model: "deepseek-chat:latest".to_string(),
            }],
        };

        // 保存 session（模拟 register_node 中的持久化逻辑）
        let session = SessionData {
            node_id: resp.node_id,
            session_id: resp.session_id,
            session_token: resp.session_token.clone(),
            capabilities: capabilities.clone(),
            poll_timeout_secs: resp.poll_timeout_secs,
        };

        storage.save_session(&session).unwrap();

        // 验证 session 已正确保存
        let loaded = storage.load_session().unwrap().unwrap();
        assert_eq!(loaded.node_id, resp.node_id);
        assert_eq!(loaded.session_id, resp.session_id);
        assert_eq!(loaded.session_token, resp.session_token);
        assert_eq!(loaded.capabilities.runtime, "ollama");
        assert_eq!(loaded.capabilities.models.len(), 1);
        assert_eq!(loaded.capabilities.models[0].model, "deepseek-chat:latest");
        assert_eq!(loaded.poll_timeout_secs, 10);
    }

    #[test]
    /// 验证空模型列表时的注册请求构建
    fn test_register_request_with_empty_models() {
        let config = NodeTokenConfig {
            server_url: "http://localhost:3000".to_string(),
            registration_token: "test-token".to_string(),
            client_instance_id: "test-instance".to_string(),
            display_name: "Test Node".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            heartbeat_interval_secs: 30,
            excluded_poll_check_interval_secs: 30,
            data_dir: None,
        };

        let models: Vec<String> = vec![];

        let req = NodeRegisterRequest {
            protocol_version: "node.v1".to_string(),
            client_instance_id: config.client_instance_id.clone(),
            display_name: config.display_name.clone(),
            registration_token: config.registration_token.clone(),
            capabilities: NodeCapabilities {
                runtime: "ollama".to_string(),
                models: models
                    .into_iter()
                    .map(|m| NodeModelCapability { model: m })
                    .collect(),
            },
        };

        assert_eq!(req.capabilities.models.len(), 0);
    }
}
