<div align="center">

# node-token

<p align="center">
  <a href="./README.md">English</a> |
  <a href="./README.zh-CN.md">简体中文</a> |
  <a href="./README.zh-TW.md">繁體中文</a> |
  <a href="./README.es.md">Español</a> |
  <a href="./README.ar.md">العربية</a>
</p>

**KeyCompute 個人 PC 節點用戶端 — 自帶算力接入**

<p align="center">
  <a href="https://github.com/keycompute/node-token/stargazers"><img src="https://img.shields.io/github/stars/keycompute/node-token?style=social" alt="GitHub Stars" /></a>
  <a href="https://github.com/keycompute/node-token/issues"><img src="https://img.shields.io/github/issues/keycompute/node-token" alt="GitHub Issues" /></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/License-GPLv3-blue.svg" alt="GPLv3 License" /></a>
  <a href="./CONTRIBUTING.md"><img src="https://img.shields.io/badge/PRs-welcome-brightgreen" alt="PRs Welcome" /></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.92%2B-orange?logo=rust" alt="Rust Version" /></a>
</p>

<p align="center">
  <a href="#功能特色">功能特色</a> •
  <a href="#快速開始">快速開始</a> •
  <a href="#設定說明">設定說明</a> •
  <a href="#使用方法">使用方法</a>
</p>

</div>

---

## 專案簡介

`node-token` 是一個輕量級 Rust 用戶端，在個人 PC 上執行，將其接入 [KeyCompute](https://github.com/keycompute/keycompute) 平台成為計算節點。它主動輪詢伺服器任務，呼叫本地 Ollama 執行，並提交結果——全程無需公網 IP。

---

## 功能特色

- **拉取式輪詢**：無需公網 IP，適合家庭網路和 NAT 後的個人 PC
- **本地 Ollama 執行**：在個人硬體上執行 Ollama 託管的本地模型
- **自動恢復**：本地持久化 session 狀態，重啟後可複用工作階段
- **心跳保活**：週期性心跳維持節點可用性
- **優雅退出**：退出時停止領取新任務，已領取任務盡力完成
- **排除狀態處理**：鏡像伺服器排除狀態，繼續低頻心跳供管理端觀察

---

## 環境需求

| 元件 | 版本 |
|:---|:---|
| Rust | ≥ 1.92 |
| Ollama | 最新版 |

> 需要有已拉取至少一個模型的 Ollama 執行個體在執行。用戶端啟動時會掃描本地模型並在註冊時回報。

---

## 快速開始

### 安裝 Ollama

```bash
# Linux
curl -fsSL https://ollama.com/install.sh | sh

# 拉取模型
ollama pull gemma3:270m
```

### 編譯並執行 node-token

```bash
# 複製並編譯
git clone https://github.com/keycompute/node-token.git
cd node-token
cp config.example.toml config.toml
# 編輯 config.toml，填入 KeyCompute 伺服器 URL 和註冊 token

# 編譯
cargo build --release

# 執行
./target/release/node-token
```

### Docker

使用 `docker-compose.yml`（推薦，包含 Ollama 和模型預熱）：

```bash
# 從範本建立 .env 檔案（編輯 NODE_TOKEN__REGISTRATION_TOKEN）
cp .env.example .env

# 啟動 Ollama + node-token
docker compose up -d

# 即時查看日誌
docker compose logs -f
```

單獨執行 node-token 容器（需要已有執行的 Ollama 實例）：

```bash
# 建置映像檔
docker build -t node-token .

# 建立資料磁碟區
docker volume create node_token_data

# 執行（使用 --network host 連接宿主機的 Ollama）
docker run -d \
  --name node-token \
  --network host \
  -v node_token_data:/data \
  -e NODE_TOKEN__SERVER_URL="http://keycompute-server:3000" \
  -e NODE_TOKEN__REGISTRATION_TOKEN="your-registration-token" \
  -e NODE_TOKEN__DISPLAY_NAME="我的 PC 節點" \
  -e NODE_TOKEN__OLLAMA_URL="http://localhost:11434" \
  node-token
```

---

## 設定說明

設定從 `config.toml` 載入（或透過 `NODE_TOKEN_CONFIG` 環境變數指定路徑）。以 `NODE_TOKEN__` 為前綴的環境變數會覆蓋檔案設定。

| 變數名稱 | 說明 | 預設值 | 必填 |
|:---|:---|:---|:---:|
| `server_url` | KeyCompute 伺服器 URL | `http://localhost:3000` | ✅ |
| `registration_token` | KeyCompute 註冊 token | — | ✅ |
| `display_name` | 節點顯示名稱 | — | ✅ |
| `ollama_url` | 本地 Ollama API 位址 | `http://localhost:11434` | ⚪ |
| `heartbeat_interval_secs` | 心跳間隔（秒） | `30` | ⚪ |
| `excluded_poll_check_interval_secs` | 排除狀態時 poll 檢查間隔（秒） | `30` | ⚪ |
| `data_dir` | 本機資料目錄 | `~/.local/share/node-token` | ⚪ |

**環境變數對應**：`NODE_TOKEN__SERVER_URL`、`NODE_TOKEN__REGISTRATION_TOKEN` 等。

> `registration_token` 和 `session_token` 絕不會以明碼形式輸出到日誌。

---

## 使用方法

`node-token` 註冊並執行後，使用者可透過 KeyCompute API 使用 `node:` 模型前綴傳送請求：

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

- `node:<模型名>` 將請求路由到節點池（僅支援非串流）
- `<模型名>`（無前綴）走正常的 Provider 帳號路由

---

## 工作原理

```text
┌─────────────┐     輪詢領取任務      ┌──────────────────┐
│  node-token │ ◄────────────────── │  KeyCompute       │
│  (你的 PC)  │ ──────────────────► │  伺服器           │
│             │   心跳/提交結果      │                   │
│     │       │                     │        │          │
│     │ 呼叫  │                     │        │ 入佇列   │
│     ▼       │                     │        ▼          │
│  ┌───────┐  │                     │  ┌──────────┐    │
│  │Ollama │  │                     │  │ 使用者   │    │
│  │       │  │                     │  │ API 請求 │    │
│  └───────┘  │                     │  └──────────┘    │
└─────────────┘                     └──────────────────┘
```

1. `node-token` 向 KeyCompute 伺服器註冊，回報可用的 Ollama 模型列表
2. 週期性傳送心跳維持工作階段存活
3. 長輪詢領取匹配其可接受模型的任務
4. 收到任務後呼叫本地 Ollama 執行個體執行並提交結果
5. 若被伺服器排除（如連續失敗過多），停止輪詢但仍繼續低頻心跳

---

## 開發指南

```bash
# 編譯
cargo build --release

# 執行測試
cargo test --lib
cargo test --tests

# 程式碼檢查
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
```

---

## 專案結構

```text
node-token/
├── src/
│   ├── main.rs              # 程式進入點，訊號處理
│   ├── config.rs            # 設定管理
│   ├── error.rs             # 錯誤類型定義
│   ├── lib.rs               # 函式庫進入點
│   ├── client/              # HTTP 用戶端
│   │   ├── api.rs           # KeyCompute API 用戶端
│   │   └── ollama.rs        # Ollama HTTP 用戶端
│   ├── protocol/            # 協定型別（複製自 keycompute-types）
│   │   ├── types.rs         # 節點協定 DTO
│   │   └── ollama.rs        # Ollama API 型別
│   ├── runtime/             # 核心執行時邏輯
│   │   ├── register.rs      # 註冊邏輯
│   │   ├── heartbeat.rs     # 心跳迴圈
│   │   ├── poll.rs          # 輪詢迴圈
│   │   └── executor.rs      # 任務執行器
│   └── storage/             # 本機持久化
│       └── mod.rs           # Session 儲存
├── tests/                   # 整合測試
├── benches/                 # 基準測試
├── config.example.toml
├── .env.example
└── Cargo.toml
```

---

## 授權條款

本專案採用 [GNU GPLv3](LICENSE) 授權條款開源。

---

<div align="center">

### 💖 感謝使用 node-token

如果這個專案對你有幫助，歡迎給它一個 ⭐️ star。

**[快速開始](#快速開始)** • **[問題回報](https://github.com/keycompute/node-token/issues)** • **[最新發佈](https://github.com/keycompute/node-token/releases)**

</div>
