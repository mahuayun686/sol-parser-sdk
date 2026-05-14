//! ShredStream 超低延迟订阅示例
//!
//! 展示如何使用 ShredStream 客户端订阅 Solana 交易事件
//!
//! ShredStream 是 Jito 提供的超低延迟数据流服务，
//! 相比 gRPC 订阅具有约 50-100ms 的延迟优势。
//!
//! ## 限制说明
//! - 仅 `static_account_keys()`，使用 ALT 的交易会有错误账户
//! - 无 Inner Instructions，无法解析 CPI 调用
//! - 无 block_time，恒为 0
//! - tx_index 是 entry 内索引而非 slot 内索引

use sol_parser_sdk::core::now_micros;
use sol_parser_sdk::shredstream::{ShredStreamClient, ShredStreamConfig};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// 辅助函数：更新最小值和最大值
fn update_min_max(min: &Arc<AtomicU64>, max: &Arc<AtomicU64>, value: u64) {
    // 更新最小值
    let mut current_min = min.load(Ordering::Relaxed);
    while value < current_min {
        match min.compare_exchange(current_min, value, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(x) => current_min = x,
        }
    }

    // 更新最大值
    let mut current_max = max.load(Ordering::Relaxed);
    while value > current_max {
        match max.compare_exchange(current_max, value, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(x) => current_max = x,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 ShredStream Low-Latency Test");
    println!("================================\n");

    run_example().await
}

async fn run_example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 配置
    let config = ShredStreamConfig {
        connection_timeout_ms: 5000,
        request_timeout_ms: 30000,
        max_decoding_message_size: 1024 * 1024 * 1024, // 1GB
        reconnect_delay_ms: 1000,
        max_reconnect_attempts: 0, // 0 = 无限重连
    };

    println!("📋 Configuration:");
    println!("   Endpoint: http://127.0.0.1:10800");
    println!("   Reconnect: infinite");
    println!();

    // 创建客户端
    let client = ShredStreamClient::new_with_config("http://127.0.0.1:10800", config).await?;

    println!("✅ ShredStream client connected");
    println!("🎧 Starting subscription...\n");

    // 订阅并获取事件队列
    let queue = client.subscribe().await?;

    // 性能统计
    let event_count = Arc::new(AtomicU64::new(0));
    let total_latency = Arc::new(AtomicU64::new(0));
    let min_latency = Arc::new(AtomicU64::new(u64::MAX));
    let max_latency = Arc::new(AtomicU64::new(0));

    // 克隆用于统计报告
    let stats_count = event_count.clone();
    let stats_total = total_latency.clone();
    let stats_min = min_latency.clone();
    let stats_max = max_latency.clone();
    let queue_for_stats = queue.clone();

    // 统计报告线程（10秒间隔）
    tokio::spawn(async move {
        let mut last_count = 0u64;
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;

            let count = stats_count.load(Ordering::Relaxed);
            let total = stats_total.load(Ordering::Relaxed);
            let min = stats_min.load(Ordering::Relaxed);
            let max = stats_max.load(Ordering::Relaxed);
            let queue_len = queue_for_stats.len();

            if count > 0 {
                let avg = total / count;
                let events_per_sec = (count - last_count) as f64 / 10.0;

                println!("\n╔════════════════════════════════════════════════════╗");
                println!("║          性能统计 (10秒间隔)                       ║");
                println!("╠════════════════════════════════════════════════════╣");
                println!("║  事件总数: {:>10}                              ║", count);
                println!("║  事件速率: {:>10.1} events/sec                  ║", events_per_sec);
                println!("║  队列长度: {:>10}                              ║", queue_len);
                println!("║  平均延迟: {:>10} μs                           ║", avg);
                println!(
                    "║  最小延迟: {:>10} μs                           ║",
                    if min == u64::MAX { 0 } else { min }
                );
                println!("║  最大延迟: {:>10} μs                           ║", max);
                println!("╚════════════════════════════════════════════════════╝\n");

                if queue_len > 1000 {
                    println!("⚠️  警告: 队列堆积 ({}), 消费速度 < 生产速度", queue_len);
                }
            }

            last_count = count;
        }
    });

    // 克隆用于消费者线程
    let consumer_event_count = event_count.clone();
    let consumer_total_latency = total_latency.clone();
    let consumer_min_latency = min_latency.clone();
    let consumer_max_latency = max_latency.clone();

    // 高性能消费事件
    tokio::spawn(async move {
        let mut spin_count = 0u32;

        loop {
            if let Some(event) = queue.pop() {
                spin_count = 0;

                // 使用高性能时钟源
                let queue_recv_us = now_micros();

                let grpc_recv_us = event.metadata().grpc_recv_us;
                let latency_us = (queue_recv_us - grpc_recv_us) as u64;

                // 更新统计
                consumer_event_count.fetch_add(1, Ordering::Relaxed);
                consumer_total_latency.fetch_add(latency_us, Ordering::Relaxed);
                update_min_max(&consumer_min_latency, &consumer_max_latency, latency_us);

                // 打印完整的时间指标和事件数据
                println!("\n================================================");
                println!("ShredStream接收时间: {} μs", grpc_recv_us);
                println!("事件接收时间:       {} μs", queue_recv_us);
                println!("延迟时间:           {} μs", latency_us);
                println!("队列长度:           {}", queue.len());
                println!("================================================");
                println!("{:?}", event);
                println!();
            } else {
                spin_count += 1;
                if spin_count < 1000 {
                    std::hint::spin_loop();
                } else {
                    tokio::task::yield_now().await;
                    spin_count = 0;
                }
            }
        }
    });

    // 自动停止（用于测试）
    let client_clone = client.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(600)).await;
        println!("⏰ Auto-stopping after 10 minutes...");
        client_clone.stop().await;
    });

    println!("🛑 Press Ctrl+C to stop...\n");
    tokio::signal::ctrl_c().await?;
    println!("\n👋 Shutting down gracefully...");

    client.stop().await;
    Ok(())
}
