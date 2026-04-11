use anyhow::Result;
use research::engine::{BacktestEngine, Bar, Direction, EngineConfig, Signal, Strategy};
use research::portfolio::Portfolio;
use tracing::info;
use tracing_subscriber::EnvFilter;

/// Example strategy: simple momentum crossover (fast EMA > slow EMA → long).
struct MomentumStrategy {
    fast: usize,
    slow: usize,
    prices: Vec<f64>,
}

impl MomentumStrategy {
    fn new(fast: usize, slow: usize) -> Self {
        Self { fast, slow, prices: Vec::new() }
    }

    fn ema(&self, period: usize) -> Option<f64> {
        if self.prices.len() < period { return None; }
        let k = 2.0 / (period as f64 + 1.0);
        let mut ema = self.prices[self.prices.len() - period];
        for &p in &self.prices[self.prices.len() - period + 1..] {
            ema = p * k + ema * (1.0 - k);
        }
        Some(ema)
    }
}

impl Strategy for MomentumStrategy {
    fn name(&self) -> &str { "momentum_ema_crossover" }

    fn on_bar(&mut self, bar: &Bar, _portfolio: &Portfolio) -> Option<Signal> {
        self.prices.push(bar.close);
        let fast = self.ema(self.fast)?;
        let slow = self.ema(self.slow)?;
        Some(Signal {
            symbol: bar.symbol.clone(),
            direction: if fast > slow { Direction::Long } else { Direction::Short },
            size_frac: 0.95,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let symbol = std::env::var("SYMBOL").unwrap_or_else(|_| "BTC-USD".into());
    let qdb_host = std::env::var("QUESTDB_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let qdb_port: u16 = std::env::var("QUESTDB_HTTP_PORT").ok()
        .and_then(|v| v.parse().ok()).unwrap_or(9000);
    let limit: usize = std::env::var("BAR_LIMIT").ok()
        .and_then(|v| v.parse().ok()).unwrap_or(100_000);

    info!("Loading {} bars for {} from QuestDB...", limit, symbol);
    let bars = research::data::load_from_questdb(&qdb_host, qdb_port, &symbol, limit).await?;
    info!("Loaded {} bars.", bars.len());

    if bars.is_empty() {
        tracing::warn!("No bars loaded. Run tick-writer first.");
        return Ok(());
    }

    let adv_map = [(symbol.clone(), 1_000_000.0)];
    let mut strategy = MomentumStrategy::new(10, 50);
    let mut engine = BacktestEngine::new(EngineConfig::default());
    let result = engine.run(&bars, &mut strategy, &adv_map);
    let m = &result.metrics;

    info!("=== Backtest Results: {} ===", strategy.name());
    info!("  Total Return:      {:.2}%", m.total_return * 100.0);
    info!("  Annualized Return: {:.2}%", m.annualized_return * 100.0);
    info!("  Sharpe Ratio:      {:.3}", m.sharpe_ratio);
    info!("  Sortino Ratio:     {:.3}", m.sortino_ratio);
    info!("  Max Drawdown:      {:.2}%", m.max_drawdown * 100.0);
    info!("  Calmar Ratio:      {:.3}", m.calmar_ratio);
    info!("  Win Rate:          {:.1}%", m.win_rate * 100.0);
    info!("  Profit Factor:     {:.3}", m.profit_factor);
    info!("  Num Trades:        {}", m.num_trades);
    info!("  Nav Bars:          {}", result.nav_curve.len());

    let json = serde_json::to_string_pretty(m)?;
    println!("{}", json);
    Ok(())
}
