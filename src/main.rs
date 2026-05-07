//! node-token - KeyCompute 个人 PC 节点客户端
//!
//! 运行在个人 PC 上，负责连接本机 Ollama、主动轮询任务并提交结果。

mod client;
mod config;
mod error;
mod protocol;
mod storage;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("node_token=info".parse()?),
        )
        .init();

    info!("node-token starting");

    // 加载配置
    let config = config::load_config()?;
    info!("Configuration loaded successfully");
    info!("Server URL: {}", config.server_url);
    info!("Client instance ID: {}", config.client_instance_id);
    info!("Display name: {}", config.display_name);
    info!("Ollama URL: {}", config.ollama_url);

    // TODO: 后续阶段实现
    // - 初始化 HTTP 客户端
    // - 加载或注册 session
    // - 启动心跳循环
    // - 启动轮询循环
    // - 等待退出信号

    info!("node-token initialized successfully");

    Ok(())
}
