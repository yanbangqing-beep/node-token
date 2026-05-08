//! 集成测试 - 节点注册流程
//!
//! 这些测试验证 node-token 库的端到端功能，
//! 使用 wiremock 模拟 KeyCompute 服务端。
//!
//! ## 测试覆盖
//! - 成功注册流程
//! - 注册失败处理（服务端 500 错误）

mod common;

use common::{create_register_request, create_register_response_json, create_test_config};
use node_token::client::KeyComputeClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
/// 测试完整的节点注册流程
///
/// 验证点：
/// 1. 客户端成功发送注册请求
/// 2. 服务端返回正确的 node_id、session_id 和 session_token
/// 3. 客户端正确解析响应数据
async fn test_full_registration_flow() {
    // 启动 mock 服务器
    let mock_server = MockServer::start().await;
    let (node_id, session_id, session_token) = create_test_config();

    // 设置注册端点的 mock 响应
    Mock::given(method("POST"))
        .and(path("/node/v1/register"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(create_register_response_json(
                node_id,
                session_id,
                &session_token,
            )),
        )
        .mount(&mock_server)
        .await;

    // 创建客户端
    let client = KeyComputeClient::new(mock_server.uri());

    // 构建注册请求（使用共享工厂函数）
    let request = create_register_request(None);

    // 执行注册
    let response = client.register(&request).await.expect("注册请求应该成功");

    // 验证响应
    assert_eq!(response.node_id, node_id);
    assert_eq!(response.session_id, session_id);
    assert_eq!(response.session_token, session_token);
    assert_eq!(response.heartbeat_interval_secs, 30);
    assert_eq!(response.poll_timeout_secs, 10);

    // 验证 mock 被调用
    mock_server.verify().await;
}

#[tokio::test]
/// 测试注册失败场景（服务端返回 500 错误）
///
/// 验证点：
/// 1. 客户端正确处理服务端错误
/// 2. 返回适当的错误信息
/// 3. HTTP 请求确实发出
async fn test_registration_server_error() {
    let mock_server = MockServer::start().await;

    // Mock 500 错误
    Mock::given(method("POST"))
        .and(path("/node/v1/register"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "error": "Internal Server Error"
        })))
        .mount(&mock_server)
        .await;

    let client = KeyComputeClient::new(mock_server.uri());
    let request = create_register_request(None);

    // 执行注册（应该失败）
    let result = client.register(&request).await;
    assert!(result.is_err(), "注册请求应该失败");

    // 验证 mock 被调用（确保 500 错误请求确实发出）
    mock_server.verify().await;
}
