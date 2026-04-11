use anyhow::Result;
use data_ingestion::health::check_questdb;
use data_ingestion::writer::{Tick, WriterConfig, TickWriter};
use data_ingestion::barbuilder::BarBuilder;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let qdb_host = std::env::var("QUESTDB_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let qdb_ilp_port: u16 = std::env::var("QUESTDB_ILP_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(9009);
    let qdb_http_port: u16 = std::env::var("QUESTDB_HTTP_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(9000);

    // Health check
    info!("Waiting for QuestDB at {}:{}...", qdb_host, qdb_http_port);
    for attempt in 1..=30 {
        match check_questdb(&qdb_host, qdb_http_port).await {
            Ok(true) => {
                info!("QuestDB ready.");
                break;
            }
            _ => {
                if attempt == 30 {
                    error!("QuestDB unreachable after 30 attempts. Exiting.");
                    std::process::exit(1);
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    }

    let cfg = WriterConfig {
        host: qdb_host.clone(),
        port: qdb_ilp_port,
        batch_size: 10_000,
        flush_interval_ms: 50,
        channel_capacity: 1_000_000,
    };

    let handle = TickWriter::spawn(cfg)?;
    let mut bar_builder = BarBuilder::new();

    info!("tick-writer ready. Listening for upstream tick feeds...");

    // In production this loop is driven by the Databento adapter or feed_handler.
    // For standalone test: emit synthetic ticks to verify write path.
    if std::env::var("TRADERX_SYNTHETIC_TEST").is_ok() {
        info!("Synthetic test mode: writing 100_000 ticks...");
        let base_ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;

        for i in 0u64..100_000 {
            let ts = base_ts + (i as i64) * 1_000_000; // 1ms apart
            let tick = Tick {
                ts_nanos: ts,
                symbol:   "BTC-USD".into(),
                exchange: "databento".into(),
                price:    60_000.0 + (i as f64) * 0.01,
                volume:   0.001 + (i % 10) as f64 * 0.0001,
                side:     if i % 2 == 0 { "buy".into() } else { "sell".into() },
                bid:      59_999.5 + (i as f64) * 0.01,
                ask:      60_000.5 + (i as f64) * 0.01,
                bid_size: 1.5,
                ask_size: 1.2,
                sequence: i as i64,
            };

            // Build bars
            if let Some(bar) = bar_builder.update_1m(&tick) {
                info!("1m bar closed: {:?}", bar);
            }

            handle.send_tick_blocking(tick);
        }

        handle.flush();
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        handle.shutdown();
        info!("Synthetic test complete.");
    } else {
        // Park — real feeds drive via WriterHandle passed to adapters
        tokio::signal::ctrl_c().await?;
        handle.shutdown();
    }

    Ok(())
}
