//! Portfolio Aggregation Engine — real-time P&L and exposure tracking.
//! Single-threaded update loop with DashMap for per-strategy state.

use crate::exposure::{ExposureBook, AssetClass, Exposure};
use crate::risk::{VarEngine, ConcentrationEngine};
use crate::persistence::WAL;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// P&L snapshot for a strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPnl {
    pub strategy_id: String,
    pub unrealized_usd: f64,
    pub realized_usd: f64,
    pub total_usd: f64,
    pub gross_exposure_usd: f64,
    pub net_exposure_usd: f64,
    pub num_trades: i64,
    pub last_update_ns: i64,
}

/// Exposure report request/response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureReport {
    pub strategy_id: String,
    pub per_symbol: Vec<(String, f64, f64)>, // (symbol, gross, net)
    pub per_asset: Vec<(AssetClass, f64, f64)>,
    pub timestamp_ns: i64,
}

/// Fill event from OMS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillEvent {
    pub strategy_id: String,
    pub symbol: String,
    pub asset_class: AssetClass,
    pub side: String, // "buy" or "sell"
    pub quantity: f64,
    pub fill_price_usd: f64,
    pub commission_usd: f64,
    pub timestamp_ns: i64,
}

/// Mark-to-market price update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    pub symbol: String,
    pub price_usd: f64,
    pub timestamp_ns: i64,
}

/// Core aggregation engine.
pub struct PortfolioAggregator {
    /// Strategy positions: (strategy, symbol) -> (quantity_fp, avg_entry_fp)
    pub positions: DashMap<(String, String), (AtomicI64, AtomicI64)>,
    
    /// Strategy realized P&L in USD (× 1e4).
    pub realized: DashMap<String, AtomicI64>,
    
    /// Trade counters per strategy.
    pub trade_counts: DashMap<String, AtomicI64>,
    
    /// Exposure tracking.
    pub exposure: Arc<ExposureBook>,
    
    /// Risk engines.
    pub var_engine: Arc<VarEngine>,
    pub concentration: Arc<ConcentrationEngine>,
    
    /// Write-ahead log for crash recovery.
    pub wal: Arc<WAL>,
    
    /// Event receiver.
    event_rx: mpsc::Receiver<AggregatorEvent>,
    
    /// Current portfolio NAV (× 1e4).
    pub nav_fp: AtomicI64,
    
    /// Peak NAV for drawdown (× 1e4).
    pub peak_nav_fp: AtomicI64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AggregatorEvent {
    Fill(FillEvent),
    Price(PriceUpdate),
    StrategyReset(String),
    GetStrategyPnl(String, mpsc::Sender<StrategyPnl>),
    GetExposure(String, mpsc::Sender<ExposureReport>),
}

impl PortfolioAggregator {
    pub fn new(event_rx: mpsc::Receiver<AggregatorEvent>, wal_path: &str) -> Arc<Self> {
        let aggregator = Arc::new(Self {
            positions: DashMap::new(),
            realized: DashMap::new(),
            trade_counts: DashMap::new(),
            exposure: Arc::new(ExposureBook::new()),
            var_engine: Arc::new(VarEngine::new()),
            concentration: Arc::new(ConcentrationEngine::new()),
            wal: Arc::new(WAL::new(wal_path)),
            event_rx,
            nav_fp: AtomicI64::new(0),
            peak_nav_fp: AtomicI64::new(0),
        });

        // Recover from WAL
        let agg_clone = Arc::clone(&aggregator);
        tokio::spawn(async move {
            if let Err(e) = agg_clone.recover_from_wal().await {
                error!("WAL recovery failed: {}", e);
            }
        });

        aggregator
    }

    /// Main event loop — single-threaded, lock-free updates.
    pub async fn run(self: Arc<Self>) {
        info!("Portfolio aggregation engine started");
        while let Some(event) = self.event_rx.recv().await {
            match event {
                AggregatorEvent::Fill(fill) => self.process_fill(fill),
                AggregatorEvent::Price(price) => self.process_price(price),
                AggregatorEvent::StrategyReset(strategy) => self.reset_strategy(&strategy),
                AggregatorEvent::GetStrategyPnl(strategy, tx) => {
                    let pnl = self.compute_strategy_pnl(&strategy);
                    let _ = tx.send(pnl);
                }
                AggregatorEvent::GetExposure(strategy, tx) => {
                    let report = self.compute_exposure_report(&strategy);
                    let _ = tx.send(report);
                }
            }
        }
    }

    fn process_fill(&self, fill: FillEvent) {
        // Log to WAL first
        if let Err(e) = self.wal.log_fill(&fill) {
            error!("WAL write failed: {}", e);
        }

        let notional = fill.quantity * fill.fill_price_usd;
        let signed_notional = if fill.side == "buy" { notional } else { -notional };
        let commission_fp = (fill.commission_usd * 1e4) as i64;

        // Update position
        let pos_key = (fill.strategy_id.clone(), fill.symbol.clone());
        let mut entry = self.positions.entry(pos_key.clone()).or_default();
        let qty_fp = entry.value().0.load(Ordering::Relaxed);
        let avg_fp = entry.value().1.load(Ordering::Relaxed);
        
        let new_qty = qty_fp as f64 * 1e-8 + if fill.side == "buy" { fill.quantity } else { -fill.quantity };
        let new_avg = if new_qty.abs() < 1e-12 {
            0.0
        } else {
            ((qty_fp as f64 * 1e-8) * (avg_fp as f64 * 1e-8) + signed_notional) / new_qty
        };
        
        entry.value().0.store((new_qty * 1e8) as i64, Ordering::Relaxed);
        entry.value().1.store((new_avg * 1e8) as i64, Ordering::Relaxed);

        // Update realized P&L
        if fill.side == "sell" {
            let pnl = signed_notional - (qty_fp as f64 * 1e-8) * (avg_fp as f64 * 1e-8) - fill.commission_usd;
            let pnl_fp = (pnl * 1e4) as i64;
            self.realized.entry(fill.strategy_id.clone())
                .or_default()
                .fetch_add(pnl_fp, Ordering::Relaxed);
        }

        // Update trade count
        self.trade_counts.entry(fill.strategy_id.clone())
            .or_default()
            .fetch_add(1, Ordering::Relaxed);

        // Update exposure
        self.exposure.apply_fill(
            &fill.strategy_id,
            &fill.symbol,
            fill.asset_class,
            signed_notional,
        );

        // Update risk metrics
        self.var_engine.update_position(&fill.strategy_id, &fill.symbol, new_qty, new_avg);
        self.concentration.update_exposure(&fill.strategy_id, fill.asset_class, signed_notional);

        debug!(
            strategy = %fill.strategy_id,
            symbol = %fill.symbol,
            notional,
            "Fill processed"
        );
    }

    fn process_price(&self, price: PriceUpdate) {
        // Update unrealized P&L for all positions in this symbol
        for entry in self.positions.iter() {
            if entry.key().1 == price.symbol {
                let (qty_fp, avg_fp) = (
                    entry.value().0.load(Ordering::Relaxed),
                    entry.value().1.load(Ordering::Relaxed),
                );
                let qty = qty_fp as f64 * 1e-8;
                let avg = avg_fp as f64 * 1e-8;
                let unrealized = qty * (price.price_usd - avg);
                
                // Note: unrealized is computed on-demand in compute_strategy_pnl
                // Here we just update risk engines
                self.var_engine.update_price(&price.symbol, price.price_usd);
            }
        }

        // Update portfolio NAV
        let total_nav = self.compute_total_nav();
        self.nav_fp.store((total_nav * 1e4) as i64, Ordering::Relaxed);
        
        // Update peak NAV
        let current = self.nav_fp.load(Ordering::Relaxed);
        let peak = self.peak_nav_fp.load(Ordering::Relaxed);
        if current > peak {
            self.peak_nav_fp.store(current, Ordering::Relaxed);
        }
    }

    fn reset_strategy(&self, strategy_id: &str) {
        // Remove all positions for this strategy
        self.positions.retain(|(s, _), _| s != strategy_id);
        self.realized.remove(strategy_id);
        self.trade_counts.remove(strategy_id);
        
        // Reset exposure
        self.exposure.per_symbol.retain(|(s, _), _| s != strategy_id);
        self.exposure.per_asset_class.retain(|(s, _), _| s != strategy_id);
        
        // Reset risk metrics
        self.var_engine.reset_strategy(strategy_id);
        self.concentration.reset_strategy(strategy_id);
        
        info!("Strategy {} reset", strategy_id);
    }

    fn compute_strategy_pnl(&self, strategy_id: &str) -> StrategyPnl {
        let mut unrealized = 0.0;
        let mut gross = 0.0;
        let mut net = 0.0;

        for entry in self.positions.iter() {
            if entry.key().0 == strategy_id {
                let qty_fp = entry.value().0.load(Ordering::Relaxed);
                let avg_fp = entry.value().1.load(Ordering::Relaxed);
                let qty = qty_fp as f64 * 1e-8;
                let avg = avg_fp as f64 * 1e-8;
                
                // Get current price from var engine
                let current_price = self.var_engine.get_price(&entry.key().1);
                let unrealized_pos = qty * (current_price - avg);
                unrealized += unrealized_pos;
                
                gross += qty.abs() * current_price;
                net += qty * current_price;
            }
        }

        let realized = self.realized.get(strategy_id)
            .map(|r| r.load(Ordering::Relaxed) as f64 * 1e-4)
            .unwrap_or(0.0);
        
        let trade_count = self.trade_counts.get(strategy_id)
            .map(|c| c.load(Ordering::Relaxed))
            .unwrap_or(0);

        StrategyPnl {
            strategy_id: strategy_id.to_owned(),
            unrealized_usd: unrealized,
            realized_usd: realized,
            total_usd: unrealized + realized,
            gross_exposure_usd: gross,
            net_exposure_usd: net,
            num_trades: trade_count,
            last_update_ns: chrono::Utc::now().timestamp_nanos(),
        }
    }

    fn compute_exposure_report(&self, strategy_id: &str) -> ExposureReport {
        let mut per_symbol = Vec::new();
        let mut per_asset = Vec::new();

        // Per-symbol exposure
        for entry in self.exposure.per_symbol.iter() {
            if entry.key().0 == strategy_id {
                per_symbol.push((
                    entry.key().1.clone(),
                    entry.value().gross(),
                    entry.value().net(),
                ));
            }
        }

        // Per-asset-class exposure
        for entry in self.exposure.per_asset_class.iter() {
            if entry.key().0 == strategy_id {
                per_asset.push((
                    *entry.key().1,
                    entry.value().gross(),
                    entry.value().net(),
                ));
            }
        }

        ExposureReport {
            strategy_id: strategy_id.to_owned(),
            per_symbol,
            per_asset,
            timestamp_ns: chrono::Utc::now().timestamp_nanos(),
        }
    }

    fn compute_total_nav(&self) -> f64 {
        let mut nav = 0.0;
        
        // Sum realized P&L
        for entry in self.realized.iter() {
            nav += entry.value().load(Ordering::Relaxed) as f64 * 1e-4;
        }
        
        // Sum unrealized P&L
        for entry in self.positions.iter() {
            let qty_fp = entry.value().0.load(Ordering::Relaxed);
            let avg_fp = entry.value().1.load(Ordering::Relaxed);
            let qty = qty_fp as f64 * 1e-8;
            let avg = avg_fp as f64 * 1e-8;
            let current_price = self.var_engine.get_price(&entry.key().1);
            nav += qty * (current_price - avg);
        }
        
        nav
    }

    async fn recover_from_wal(&self) -> anyhow::Result<()> {
        info!("Starting WAL recovery...");
        let events = self.wal.read_all().await?;
        for event in &events {
            match event {
                AggregatorEvent::Fill(fill) => self.process_fill(fill),
                AggregatorEvent::Price(price) => self.process_price(price),
                _ => {}
            }
        }
        info!("WAL recovery complete: {} events restored", events.len());
        Ok(())
    }
}
