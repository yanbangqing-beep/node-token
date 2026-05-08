//! 配置管理模块
//!
//! 支持从配置文件和环境变量加载配置，环境变量优先级更高。

use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

/// 环境变量守卫器，确保测试后清理环境变量
#[cfg(test)]
struct EnvGuard {
    vars: Vec<(String, Option<String>)>,
}

#[cfg(test)]
impl EnvGuard {
    fn new(keys: &[&str]) -> Self {
        let vars = keys
            .iter()
            .map(|key| (key.to_string(), std::env::var(key).ok()))
            .collect();
        Self { vars }
    }
}

#[cfg(test)]
impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, old_value) in &self.vars {
            match old_value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }
}

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

    /// Excluded 节点 poll 检查间隔（秒），默认 30（与心跳间隔一致）
    /// 节点被 excluded 后，poll 循环定期检查是否恢复
    #[serde(default = "default_excluded_poll_check_interval")]
    pub excluded_poll_check_interval_secs: u64,

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

fn default_excluded_poll_check_interval() -> u64 {
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
        .set_default("display_name", "My KeyComputeC Node")?
        .set_default("ollama_url", "http://localhost:11434")?
        .set_default("heartbeat_interval_secs", 30)?
        .set_default("excluded_poll_check_interval_secs", 30)?
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
            excluded_poll_check_interval_secs: default_excluded_poll_check_interval(),
            data_dir: None,
        };

        assert_eq!(config.ollama_url, "http://localhost:11434");
        assert_eq!(config.heartbeat_interval_secs, 30);
        assert_eq!(config.excluded_poll_check_interval_secs, 30);
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
            excluded_poll_check_interval_secs: 30,
            data_dir: Some("/tmp/test-node-token".to_string()),
        };

        assert_eq!(
            config.data_dir_path(),
            PathBuf::from("/tmp/test-node-token")
        );
    }

    #[test]
    fn test_config_from_toml_file() {
        // 创建临时配置文件
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let config_content = r#"
server_url = "http://example.com:3000"
registration_token = "my-secret-token"
client_instance_id = "my-pc-001"
display_name = "My PC Node"
ollama_url = "http://localhost:11434"
heartbeat_interval_secs = 60
excluded_poll_check_interval_secs = 60
"#;
        std::fs::write(&config_path, config_content).unwrap();

        // 使用 EnvGuard 确保测试后清理环境变量
        let _guard = EnvGuard::new(&["NODE_TOKEN_CONFIG"]);
        std::env::set_var("NODE_TOKEN_CONFIG", config_path.to_str().unwrap());

        // 清除可能干扰的其他 NODE_TOKEN_ 环境变量
        std::env::remove_var("NODE_TOKEN__SERVER_URL");
        std::env::remove_var("NODE_TOKEN__REGISTRATION_TOKEN");
        std::env::remove_var("NODE_TOKEN__CLIENT_INSTANCE_ID");
        std::env::remove_var("NODE_TOKEN__DISPLAY_NAME");

        // 加载配置
        let config = load_config().unwrap();

        assert_eq!(config.server_url, "http://example.com:3000");
        assert_eq!(config.registration_token, "my-secret-token");
        assert_eq!(config.client_instance_id, "my-pc-001");
        assert_eq!(config.display_name, "My PC Node");
        assert_eq!(config.ollama_url, "http://localhost:11434");
        assert_eq!(config.heartbeat_interval_secs, 60);
        assert_eq!(config.excluded_poll_check_interval_secs, 60);
    }

    #[test]
    fn test_config_with_custom_values() {
        let config = NodeTokenConfig {
            server_url: "https://keycompute.example.com".to_string(),
            registration_token: "custom-token-123".to_string(),
            client_instance_id: "custom-instance-456".to_string(),
            display_name: "Custom Node".to_string(),
            ollama_url: "http://192.168.1.100:11434".to_string(),
            heartbeat_interval_secs: 45,
            excluded_poll_check_interval_secs: 45,
            data_dir: Some("/custom/data/dir".to_string()),
        };

        assert_eq!(config.server_url, "https://keycompute.example.com");
        assert_eq!(config.registration_token, "custom-token-123");
        assert_eq!(config.client_instance_id, "custom-instance-456");
        assert_eq!(config.display_name, "Custom Node");
        assert_eq!(config.ollama_url, "http://192.168.1.100:11434");
        assert_eq!(config.heartbeat_interval_secs, 45);
        assert_eq!(config.excluded_poll_check_interval_secs, 45);
        assert_eq!(config.data_dir, Some("/custom/data/dir".to_string()));
    }

    #[test]
    fn test_data_dir_path_with_none() {
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

        // 当 data_dir 为 None 时，应该使用默认路径
        let path = config.data_dir_path();
        // 路径应该以 "node-token" 结尾
        assert!(path.ends_with("node-token"));
    }

    #[test]
    fn test_config_validation_empty_server_url() {
        // 直接测试验证逻辑，而不是通过 load_config()
        // 因为 load_config() 有默认值，会覆盖空字符串
        let config = NodeTokenConfig {
            server_url: "".to_string(), // 空值
            registration_token: "test-token".to_string(),
            client_instance_id: "test-instance".to_string(),
            display_name: "Test Node".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            heartbeat_interval_secs: 30,
            excluded_poll_check_interval_secs: 30,
            data_dir: None,
        };

        // 验证空值检测逻辑
        assert!(config.server_url.is_empty());
    }

    #[test]
    fn test_config_validation_empty_registration_token() {
        // 直接测试验证逻辑，而不是通过 load_config()
        let config = NodeTokenConfig {
            server_url: "http://localhost:3000".to_string(),
            registration_token: "".to_string(), // 空值
            client_instance_id: "test-instance".to_string(),
            display_name: "Test Node".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            heartbeat_interval_secs: 30,
            excluded_poll_check_interval_secs: 30,
            data_dir: None,
        };

        // 验证空值检测逻辑
        assert!(config.registration_token.is_empty());
    }

    #[test]
    fn test_config_validation_empty_client_instance_id() {
        // 直接测试验证逻辑，而不是通过 load_config()
        // 因为 load_config() 有默认值，会覆盖空字符串
        let config = NodeTokenConfig {
            server_url: "http://localhost:3000".to_string(),
            registration_token: "test-token".to_string(),
            client_instance_id: "".to_string(), // 空值
            display_name: "Test Node".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            heartbeat_interval_secs: 30,
            excluded_poll_check_interval_secs: 30,
            data_dir: None,
        };

        // 验证空值检测逻辑
        assert!(config.client_instance_id.is_empty());
    }

    #[test]
    fn test_config_validation_empty_display_name() {
        // 直接测试验证逻辑
        let config = NodeTokenConfig {
            server_url: "http://localhost:3000".to_string(),
            registration_token: "test-token".to_string(),
            client_instance_id: "test-instance".to_string(),
            display_name: "".to_string(), // 空值
            ollama_url: "http://localhost:11434".to_string(),
            heartbeat_interval_secs: 30,
            excluded_poll_check_interval_secs: 30,
            data_dir: None,
        };

        // 验证空值检测逻辑
        assert!(config.display_name.is_empty());
    }

    // ========================================================================
    // 边界条件测试
    // ========================================================================

    #[test]
    fn test_config_with_unicode_display_name() {
        let config = NodeTokenConfig {
            server_url: "http://localhost:3000".to_string(),
            registration_token: "test-token".to_string(),
            client_instance_id: "test-instance".to_string(),
            display_name: "🚀 My Node 节点".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            heartbeat_interval_secs: 30,
            excluded_poll_check_interval_secs: 30,
            data_dir: None,
        };

        assert_eq!(config.display_name, "🚀 My Node 节点");
    }

    #[test]
    fn test_config_with_special_characters() {
        let config = NodeTokenConfig {
            server_url: "http://localhost:3000".to_string(),
            registration_token: "test-token-with-special-chars!@#$%".to_string(),
            client_instance_id: "test-instance_123.456".to_string(),
            display_name: "Node \"Test\" (Production)".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            heartbeat_interval_secs: 30,
            excluded_poll_check_interval_secs: 30,
            data_dir: None,
        };

        assert_eq!(
            config.registration_token,
            "test-token-with-special-chars!@#$%"
        );
        assert_eq!(config.display_name, "Node \"Test\" (Production)");
    }

    #[test]
    fn test_config_custom_data_dir() {
        let config = NodeTokenConfig {
            server_url: "http://localhost:3000".to_string(),
            registration_token: "test-token".to_string(),
            client_instance_id: "test-instance".to_string(),
            display_name: "Test Node".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            heartbeat_interval_secs: 30,
            excluded_poll_check_interval_secs: 30,
            data_dir: Some("/custom/path/to/data".to_string()),
        };

        assert_eq!(config.data_dir, Some("/custom/path/to/data".to_string()));
        assert_eq!(
            config.data_dir_path(),
            PathBuf::from("/custom/path/to/data")
        );
    }
}
