// Live Trading System Main Binary
// Integrates BAM router, live data, order submission, and paper trading

use oms_engine::{
    bam_router::BamTradingRouter,
    live_data::{LiveMarketDataFeed, MarketDataAggregator, WebSocketFeedConfig},
    live_orders::LiveOrderConfig,
    order_conversion::{OrderConversionConfig, OrderSubmissionPipeline},
    paper_trading::{PaperTradingConfig, PaperTradingEngine},
    risk_bus::RiskBus,
    signal_router::AgentSignal,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    log::info!("Starting TraderX Live Trading System");

    // Initialize risk bus
    let risk_bus_inner = RiskBus::new(1_000_000.0, -2000);
    let risk_bus = Arc::new(risk_bus_inner);

    // Initialize BAM trading router
    let bam_router = BamTradingRouter::new(Arc::clone(&risk_bus));
    bam_router.initialize(chrono::Utc::now().timestamp()).await;

    // Initialize live market data feed
    let (aggregator, tick_sender) = MarketDataAggregator::new();
    let feed_config = WebSocketFeedConfig {
        exchange: "binance".to_string(),
        symbols: vec!["BTCUSDT".to_string()],
        reconnect_interval_ms: 5000,
    };

    let feed = LiveMarketDataFeed::new(feed_config, tick_sender);
    tokio::spawn(async move {
        let _ = feed.start().await;
    });

    // Initialize order conversion pipeline
    let order_config = OrderConversionConfig::default();
    let order_pipeline = Arc::new(OrderSubmissionPipeline::new(
        order_config,
        Arc::clone(&risk_bus),
    ));

    // Initialize paper trading engine (default mode for safety)
    let paper_config = PaperTradingConfig::default();
    let mut paper_engine = PaperTradingEngine::new(paper_config);

    // Initialize live order submitter (in paper trading mode)
    let live_order_config = LiveOrderConfig {
        paper_trading: true, // Start in paper trading mode
        ..Default::default()
    };

    // Note: This would require a real exchange adapter in production
    // For now, we'll use the paper trading engine directly

    log::info!("System initialized successfully");
    log::info!("Paper trading mode: ENABLED");

    // Simulate trading signals
    let signal = AgentSignal {
        agent_id: "bam_router".to_string(),
        symbol: "BTCUSDT".to_string(),
        direction: "long".to_string(),
        conviction: 0.7,
        max_notional_usd: 10_000.0,
        ttl_ms: 60000,
        meta: serde_json::Value::Null,
    };

    // Route signal through BAM router
    match bam_router.route_signal(signal.clone()).await {
        Ok(outcome) => {
            log::info!("Signal routed: status={:?}", outcome.status);

            if matches!(
                outcome.status,
                oms_engine::signal_router::RouteStatus::Submitted
            ) {
                // Convert to order
                match order_pipeline.process_signal(signal).await {
                    Ok(route_outcome) => {
                        if let Some(order_id) = route_outcome.order_id {
                            log::info!("Order created: {}", order_id);

                            // In paper trading, simulate the order
                            // In production, would submit via LiveOrderSubmitter
                            log::info!("Paper trading: order {} simulated", order_id);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to process signal: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Failed to route signal: {}", e);
        }
    }

    // Process market ticks
    let mut aggregator = aggregator;
    tokio::spawn(async move {
        while let Some(tick) = aggregator.next_tick().await {
            log::info!("Received tick: {} @ {}", tick.symbol, tick.price);

            // Update BAM grid with tick data
            // bam_router.process_tick(market_class, tick.price, tick.volume).await;
        }
    });

    log::info!("Live trading system running. Press Ctrl+C to stop.");

    // Keep the main task alive
    tokio::signal::ctrl_c().await?;

    log::info!("Shutting down live trading system");

    Ok(())
}
