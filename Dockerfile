# =============================================================================
# Dockerfile — node-token 节点客户端构建
# =============================================================================

# ─────────────────────────────────────────────────────────────────────────────
# 阶段 1：builder — 编译应用
# ─────────────────────────────────────────────────────────────────────────────
FROM rust:1.92.0-bookworm AS builder

# 安装构建依赖（node-token 不需要 libpq/libssl，只需 ca-certificates）
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 先复制依赖文件以利用缓存
COPY Cargo.toml Cargo.lock ./

# 创建空的 src 文件（用于缓存依赖）
RUN mkdir -p src && \
    echo 'fn main() {}' > src/main.rs && \
    echo '' > src/lib.rs && \
    mkdir -p benches && \
    touch benches/http_client.rs benches/protocol_serialization.rs benches/storage_operations.rs

# 构建空的依赖层以缓存依赖
RUN cargo build --release 2>/dev/null || true

# 复制真实源代码
COPY src/ src/
COPY benches/ benches/

# 重新构建实际应用（touch 所有源文件触发重新编译）
RUN find src -type f -exec touch {} + && cargo build --release

# ─────────────────────────────────────────────────────────────────────────────
# 阶段 2：runtime — 最小化运行时镜像
# ─────────────────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

# 安装运行时依赖（仅需要 ca-certificates 用于 HTTPS 连接）
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 运行用户
RUN groupadd -r nodetoken && useradd -r -g nodetoken nodetoken

# 创建数据目录用于 session 持久化
RUN mkdir -p /data && chown nodetoken:nodetoken /data

WORKDIR /app

# 复制编译产物
COPY --from=builder /app/target/release/node-token /usr/local/bin/node-token
RUN chmod +x /usr/local/bin/node-token

USER nodetoken

# 在容器中默认使用 /data 作为数据目录
ENV NODE_TOKEN__DATA_DIR=/data

ENTRYPOINT ["/usr/local/bin/node-token"]
