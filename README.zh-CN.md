<div align="center">

# node-token

<p align="center">
  <a href="./README.md">English</a> |
  <a href="./README.zh-CN.md">简体中文</a> |
  <a href="./README.zh-TW.md">繁體中文</a> |
  <a href="./README.es.md">Español</a> |
  <a href="./README.ar.md">العربية</a>
</p>

**KeyCompute 个人 PC 节点客户端 — 自带算力接入**

<p align="center">
  <a href="https://github.com/keycompute/node-token/stargazers"><img src="https://img.shields.io/github/stars/keycompute/node-token?style=social" alt="GitHub Stars" /></a>
  <a href="https://github.com/keycompute/node-token/issues"><img src="https://img.shields.io/github/issues/keycompute/node-token" alt="GitHub Issues" /></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/License-GPLv3-blue.svg" alt="GPLv3 License" /></a>
  <a href="./CONTRIBUTING.md"><img src="https://img.shields.io/badge/PRs-welcome-brightgreen" alt="PRs Welcome" /></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.92%2B-orange?logo=rust" alt="Rust Version" /></a>
</p>

<p align="center">
  <a href="#功能特性">功能特性</a> •
  <a href="#快速开始">快速开始</a> •
  <a href="#配置说明">配置说明</a> •
  <a href="#使用方法">使用方法</a>
</p>

</div>

---

## 项目简介

`node-token` 是一个轻量级 Rust 客户端，运行在个人 PC 上，将其接入 [KeyCompute](https://github.com/keycompute/keycompute) 平台成为计算节点。它主动轮询服务端任务，调用本地 Ollama 执行，并提交结果——全程无需公网 IP。

---

## 功能特性

- **拉取式轮询**：无需公网 IP，适合家庭网络和 NAT 后的个人 PC
- **本地 Ollama 执行**：在个人硬件上运行 Ollama 托管的本地模型
- **自动恢复**：本地持久化 session 状态，重启后可复用会话
- **心跳保活**：周期性心跳维持节点可用性
- **优雅退出**：退出时停止领取新任务，已领取任务尽力完成
- **排除状态处理**：镜像服务端排除状态，继续低频心跳供管理端观察

---

## 环境要求

| 组件 | 版本 |
|:---|:---|
| Rust | ≥ 1.92 |
| Ollama | 最新版 |

> 需要有已拉取至少一个模型的 Ollama 实例在运行。客户端启动时会扫描本地模型并在注册时上报。

---

## 快速开始

### 安装 Ollama

```bash
# Linux
curl -fsSL https://ollama.com/install.sh | sh

# 拉取模型
ollama pull gemma3:270m
```

### 编译并运行 node-token

```bash
# 克隆并编译
git clone https://github.com/keycompute/node-token.git
cd node-token
cp config.example.toml config.toml
# 编辑 config.toml，填入 KeyCompute 服务端 URL 和注册 token

# 编译
cargo build --release

# 运行
./target/release/node-token
```

### Docker

使用 `docker-compose.yml`（推荐，包含 Ollama 和模型预热）：

```bash
# 从模板创建 .env 文件（编辑 NODE_TOKEN__REGISTRATION_TOKEN）
cp .env.example .env

# 启动 Ollama + node-token
docker compose up -d

# 实时查看日志
docker compose logs -f
```

单独运行 node-token 容器（需要已有运行的 Ollama 实例）：

```bash
# 构建镜像
docker build -t node-token .

# 创建数据卷
docker volume create node_token_data

# 运行（使用 --network host 连接宿主机的 Ollama）
docker run -d \
  --name node-token \
  --network host \
  -v node_token_data:/data \
  -e NODE_TOKEN__SERVER_URL="http://keycompute-server:3000" \
  -e NODE_TOKEN__REGISTRATION_TOKEN="your-registration-token" \
  -e NODE_TOKEN__CLIENT_INSTANCE_ID="my-node-001" \
  -e NODE_TOKEN__DISPLAY_NAME="我的 PC 节点" \
  -e NODE_TOKEN__OLLAMA_URL="http://localhost:11434" \
  node-token
```

---

## 配置说明

配置从 `config.toml` 加载（或通过 `NODE_TOKEN_CONFIG` 环境变量指定路径）。以 `NODE_TOKEN__` 为前缀的环境变量会覆盖文件配置。

| 变量名 | 说明 | 默认值 | 必填 |
|:---|:---|:---|:---:|
| `server_url` | KeyCompute 服务端 URL | `http://localhost:3000` | ✅ |
| `registration_token` | KeyCompute 注册 token | — | ✅ |
| `client_instance_id` | 节点唯一标识（重启后沿用） | — | ✅ |
| `display_name` | 节点显示名称 | — | ✅ |
| `ollama_url` | 本地 Ollama API 地址 | `http://localhost:11434` | ⚪ |
| `heartbeat_interval_secs` | 心跳间隔（秒） | `30` | ⚪ |
| `excluded_poll_check_interval_secs` | 排除状态时 poll 检查间隔（秒） | `30` | ⚪ |
| `data_dir` | 本地数据目录 | `~/.local/share/node-token` | ⚪ |

**环境变量映射**：`NODE_TOKEN__SERVER_URL`、`NODE_TOKEN__REGISTRATION_TOKEN` 等。

> `registration_token` 和 `session_token` 绝不会以明文形式输出到日志。

---

## 使用方法

`node-token` 注册并运行后，用户可通过 KeyCompute API 使用 `node:` 模型前缀发送请求：

```bash
curl -s http://your-keycompute-server:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-xxx" \
  -d '{
    "model": "node:gemma3:270m",
    "messages": [{"role": "user", "content": "你好！"}],
    "stream": false
  }'
```

- `node:<模型名>` 将请求路由到节点池（仅支持非流式）
- `<模型名>`（无前缀）走正常的 Provider 账户路由

---

## 工作原理

```text
┌─────────────┐     轮询领取任务      ┌──────────────────┐
│  node-token │ ◄────────────────── │  KeyCompute       │
│  (你的 PC)  │ ──────────────────► │  服务端           │
│             │   心跳/提交结果      │                   │
│     │       │                     │        │          │
│     │ 调用  │                     │        │ 入队     │
│     ▼       │                     │        ▼          │
│  ┌───────┐  │                     │  ┌──────────┐    │
│  │Ollama │  │                     │  │ 用户 API │    │
│  └───────┘  │                     │  │ 请求     │    │
└─────────────┘                     └──┴──────────┴────┘
```

1. `node-token` 向 KeyCompute 服务端注册，上报可用的 Ollama 模型列表
2. 周期性发送心跳维持会话存活
3. 长轮询领取匹配其可接受模型的任务
4. 收到任务后调用本地 Ollama 实例执行并提交结果
5. 若被服务端排除（如连续失败过多），停止轮询但仍继续低频心跳

---

## 开发指南

```bash
# 编译
cargo build --release

# 运行测试
cargo test --lib
cargo test --tests

# 代码检查
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
```

---

## 项目结构

```text
node-token/
├── src/
│   ├── main.rs              # 程序入口，信号处理
│   ├── config.rs            # 配置管理
│   ├── error.rs             # 错误类型定义
│   ├── lib.rs               # 库入口
│   ├── client/              # HTTP 客户端
│   │   ├── api.rs           # KeyCompute API 客户端
│   │   └── ollama.rs        # Ollama HTTP 客户端
│   ├── protocol/            # 协议类型（复制自 keycompute-types）
│   │   ├── types.rs         # 节点协议 DTO
│   │   └── ollama.rs        # Ollama API 类型
│   ├── runtime/             # 核心运行时逻辑
│   │   ├── register.rs      # 注册逻辑
│   │   ├── heartbeat.rs     # 心跳循环
│   │   ├── poll.rs          # 轮询循环
│   │   └── executor.rs      # 任务执行器
│   └── storage/             # 本地持久化
│       └── mod.rs           # Session 存储
├── tests/                   # 集成测试
├── benches/                 # 基准测试
├── config.example.toml
├── .env.example
└── Cargo.toml
```

---

## 许可证

本项目采用 [GNU GPLv3](LICENSE) 许可证开源。

---

<div align="center">

### 💖 感谢使用 node-token

如果这个项目对你有帮助，欢迎给它一个 ⭐️ star。

**[快速开始](#快速开始)** • **[问题反馈](https://github.com/keycompute/node-token/issues)** • **[最新发布](https://github.com/keycompute/node-token/releases)**

</div>
