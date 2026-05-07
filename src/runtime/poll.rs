/// 轮询循环逻辑
///
/// 定期从服务端领取任务，提交到执行器执行。
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tracing::{debug, error, info};

use crate::client::KeyComputeClient;
use crate::protocol::types::NodePollRequest;
use crate::runtime::executor::TaskExecutor;
use crate::storage::SessionData;

/// 轮询循环
///
/// # 参数
/// - `client`: KeyCompute HTTP 客户端
/// - `session`: 当前 session 信息
/// - `executor`: 任务执行器
/// - `is_excluded`: 节点排除标志（由 heartbeat 循环更新）
/// - `stop_signal`: 退出信号
///
/// # 行为
/// - 定期调用 poll API 领取任务
/// - 如果节点被 excluded，停止 poll
/// - 服务端返回 retry_after_ms 时等待指定时间
/// - 网络错误指数退避（AGENTS.md 第 724 行）
#[allow(dead_code)] // 在阶段五使用
pub async fn poll_loop(
    client: &KeyComputeClient,
    session: &SessionData,
    executor: Arc<TaskExecutor>,
    is_excluded: Arc<AtomicBool>,
    stop_signal: Arc<AtomicBool>,
) {
    info!("Starting poll loop");

    // 连续失败计数，用于指数退避
    let mut consecutive_failures: u32 = 0;
    let max_backoff = Duration::from_secs(16);

    while !stop_signal.load(Ordering::Relaxed) {
        // 如果节点被 excluded，停止 poll
        if is_excluded.load(Ordering::Relaxed) {
            info!("Node excluded, stopping poll (will continue heartbeat only)");
            // 每 60 秒检查一次是否恢复
            tokio::time::sleep(Duration::from_secs(60)).await;
            continue;
        }

        let req = NodePollRequest {
            protocol_version: "node.v1".to_string(),
            node_id: session.node_id,
            session_id: session.session_id,
        };

        match client.poll(&req).await {
            Ok(resp) => {
                // 成功后重置失败计数
                consecutive_failures = 0;

                if let Some(task) = resp.task {
                    info!(
                        "Received task: task_id={}, model={}, deadline_unix_ms={}",
                        task.task_id, task.model, task.deadline_unix_ms
                    );

                    // 收到任务，提交到执行器
                    // 使用 Arc 克隆 executor，让 executor 在后台执行
                    let executor_clone = executor.clone();
                    tokio::spawn(async move {
                        executor_clone.execute(task).await;
                    });
                } else if let Some(retry_ms) = resp.retry_after_ms {
                    debug!("No task available, retry_after={}ms", retry_ms);
                    tokio::time::sleep(Duration::from_millis(retry_ms)).await;
                } else {
                    // 没有任务也没有 retry_after，短暂等待后继续
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
            Err(e) => {
                error!("Poll failed: {}", e);
                // 网络错误指数退避（AGENTS.md 第 724 行）
                consecutive_failures += 1;
                let backoff = std::cmp::min(
                    Duration::from_secs(2_u64.pow(consecutive_failures.min(4))),
                    max_backoff,
                );
                info!(
                    "Poll retrying after {}s (consecutive_failures={})",
                    backoff.as_secs(),
                    consecutive_failures
                );
                tokio::time::sleep(backoff).await;
            }
        }
    }

    info!("Poll loop stopped");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_excluded_flag_check() {
        let is_excluded = Arc::new(AtomicBool::new(false));

        // 非 excluded 时可以 poll
        assert!(!is_excluded.load(Ordering::Relaxed));

        // excluded 时停止 poll
        is_excluded.store(true, Ordering::Relaxed);
        assert!(is_excluded.load(Ordering::Relaxed));
    }

    #[test]
    fn test_stop_signal_check() {
        let stop_signal = Arc::new(AtomicBool::new(false));

        // 未停止
        assert!(!stop_signal.load(Ordering::Relaxed));

        // 发送停止信号
        stop_signal.store(true, Ordering::Relaxed);
        assert!(stop_signal.load(Ordering::Relaxed));
    }
}
