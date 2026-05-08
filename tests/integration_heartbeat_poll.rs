//! 集成测试 - 心跳和轮询流程
//!
//! 验证节点心跳保活和任务轮询的端到端功能。
//!
//! ## 测试覆盖
//! - 心跳成功流程
//! - 轮询有任务场景
//! - 轮询无任务场景

mod common;

use common::{
    create_heartbeat_request, create_heartbeat_response_json, create_poll_empty_response_json,
    create_poll_request, create_test_config,
};
use node_token::client::KeyComputeClient;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
/// 测试心跳完整流程
///
/// 验证点：
/// 1. 客户端成功发送心跳请求
/// 2. 携带正确的 Authorization header
/// 3. 服务端返回 accepted=true 和节点状态
async fn test_heartbeat_flow() {
    let mock_server = MockServer::start().await;
    let (node_id, session_id, session_token) = create_test_config();

    Mock::given(method("POST"))
        .and(path("/node/v1/heartbeat"))
        .and(header(
            "Authorization",
            format!("Bearer {}", session_token).as_str(),
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(create_heartbeat_response_json(true, "online", 0)),
        )
        .mount(&mock_server)
        .await;

    let client = KeyComputeClient::new(mock_server.uri());
    client.set_session_token(session_token).await;

    let request = create_heartbeat_request(node_id, session_id, None);
    let response = client.heartbeat(&request).await.expect("心跳请求应该成功");

    assert!(response.accepted, "心跳应该被接受");
    assert_eq!(response.node_status, "online");
    assert_eq!(response.server_failure_count, 0);
    assert_eq!(response.failure_threshold, 3);

    // 验证 mock 被调用
    mock_server.verify().await;
}

#[tokio::test]
/// 测试轮询领取任务（有任务场景）
///
/// 验证点：
/// 1. 客户端成功发送轮询请求
/// 2. 服务端返回任务信封
/// 3. 任务数据完整（task_id, lease_id, model, payload）
async fn test_poll_with_task() {
    let mock_server = MockServer::start().await;
    let (node_id, session_id, session_token) = create_test_config();

    Mock::given(method("POST"))
        .and(path("/node/v1/tasks/poll"))
        .and(header(
            "Authorization",
            format!("Bearer {}", session_token).as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "protocol_version": "node.v1",
            "task": {
                "task_id": "00000000-0000-0000-0000-000000000010",
                "lease_id": "00000000-0000-0000-0000-000000000011",
                "model": "deepseek-chat:latest",
                "deadline_unix_ms": 9999999999999i64,
                "complete_grace_until_unix_ms": 9999999999999i64,
                "payload": {
                    "request_id": "00000000-0000-0000-0000-000000000012",
                    "chat": {
                        "model": "deepseek-chat:latest",
                        "messages": [{"role": "user", "content": "Hello"}],
                        "stream": false
                    }
                }
            },
            "retry_after_ms": null
        })))
        .mount(&mock_server)
        .await;

    let client = KeyComputeClient::new(mock_server.uri());
    client.set_session_token(session_token).await;

    let request = create_poll_request(node_id, session_id);
    let response = client.poll(&request).await.expect("轮询请求应该成功");

    assert!(response.task.is_some(), "应该返回任务");
    let task = response.task.unwrap();
    assert_eq!(task.model, "deepseek-chat:latest");
    assert_eq!(task.payload.chat.messages[0].content, "Hello");

    // 验证 mock 被调用
    mock_server.verify().await;
}

#[tokio::test]
/// 测试轮询领取任务（无任务场景）
///
/// 验证点：
/// 1. 客户端成功发送轮询请求
/// 2. 服务端返回 task=null
/// 3. 包含 retry_after_ms 建议
async fn test_poll_no_task() {
    let mock_server = MockServer::start().await;
    let (node_id, session_id, session_token) = create_test_config();

    Mock::given(method("POST"))
        .and(path("/node/v1/tasks/poll"))
        .and(header(
            "Authorization",
            format!("Bearer {}", session_token).as_str(),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(create_poll_empty_response_json(Some(5000))),
        )
        .mount(&mock_server)
        .await;

    let client = KeyComputeClient::new(mock_server.uri());
    client.set_session_token(session_token).await;

    let request = create_poll_request(node_id, session_id);
    let response = client.poll(&request).await.expect("轮询请求应该成功");

    assert!(response.task.is_none(), "不应该返回任务");
    assert_eq!(response.retry_after_ms, Some(5000));

    // 验证 mock 被调用
    mock_server.verify().await;
}
