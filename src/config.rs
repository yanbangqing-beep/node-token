//! 配置管理模块
//!
//! 支持从配置文件和环境变量加载配置，环境变量优先级更高。

use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

/// node-token 配置结构
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)] // 部分字段在后续阶段使用
pub struct NodeTokenConfig {
    /// KeyCompute 服务端 URL
    pub server_url: String,

    /// 注册 token（从 KeyCompute 配置获取）
    pub registration_token: String,

    /// 客户端实例 ID（建议固定以便重启复用）
    pub client_instance_id: String,

    /// 节点显示名称
    pub display_name: String,

    /// 本地 Ollama URL，默认 http://localhost:11434
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,

    /// 心跳间隔（秒），默认 30
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval_secs: u64,

    /// 本地数据目录，默认 ~/.local/share/node-token
    #[serde(default = "default_data_dir")]
    pub data_dir: Option<String>,
}

fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_heartbeat_interval() -> u64 {
    30
}

fn default_data_dir() -> Option<String> {
    dirs::data_local_dir().map(|d| d.join("node-token").to_string_lossy().to_string())
}

impl NodeTokenConfig {
    /// 获取数据目录路径
    #[allow(dead_code)] // 在后续阶段使用
    pub fn data_dir_path(&self) -> PathBuf {
        self.data_dir
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("node-token")
            })
    }
}

/// 加载配置
///
/// 加载优先级：
/// 1. 环境变量（前缀 NODE_TOKEN_）
/// 2. 配置文件（config.toml 或 NODE_TOKEN_CONFIG 指定的路径）
pub fn load_config() -> Result<NodeTokenConfig> {
    // 确定配置文件路径
    let config_path =
        std::env::var("NODE_TOKEN_CONFIG").unwrap_or_else(|_| "config.toml".to_string());

    let builder = Config::builder()
        // 默认值
        .set_default("server_url", "http://localhost:3000")?
        .set_default("registration_token", "")?
        .set_default("client_instance_id", "")?
        .set_default("display_name", "My PC Node")?
        .set_default("ollama_url", "http://localhost:11434")?
        .set_default("heartbeat_interval_secs", 30)?
        // 从配置文件加载
        .add_source(File::with_name(&config_path).required(false))
        // 从环境变量加载（优先级最高）
        .add_source(
            Environment::with_prefix("NODE_TOKEN")
                .separator("__")
                .try_parsing(true),
        );

    let config = builder.build()?;

    let node_config: NodeTokenConfig = config
        .try_deserialize()
        .map_err(|e: ConfigError| anyhow::anyhow!("Failed to deserialize config: {}", e))?;

    // 验证必需字段
    if node_config.server_url.is_empty() {
        anyhow::bail!("server_url is required");
    }
    if node_config.registration_token.is_empty() {
        anyhow::bail!("registration_token is required");
    }
    if node_config.client_instance_id.is_empty() {
        anyhow::bail!("client_instance_id is required");
    }
    if node_config.display_name.is_empty() {
        anyhow::bail!("display_name is required");
    }

    Ok(node_config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        // 测试默认值是否正确
        let config = NodeTokenConfig {
            server_url: "http://localhost:3000".to_string(),
            registration_token: "test-token".to_string(),
            client_instance_id: "test-instance".to_string(),
            display_name: "Test Node".to_string(),
            ollama_url: default_ollama_url(),
            heartbeat_interval_secs: default_heartbeat_interval(),
            data_dir: None,
        };

        assert_eq!(config.ollama_url, "http://localhost:11434");
        assert_eq!(config.heartbeat_interval_secs, 30);
    }

    #[test]
    fn test_data_dir_path() {
        let config = NodeTokenConfig {
            server_url: "http://localhost:3000".to_string(),
            registration_token: "test-token".to_string(),
            client_instance_id: "test-instance".to_string(),
            display_name: "Test Node".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            heartbeat_interval_secs: 30,
            data_dir: Some("/tmp/test-node-token".to_string()),
        };

        assert_eq!(
            config.data_dir_path(),
            PathBuf::from("/tmp/test-node-token")
        );
    }
}
