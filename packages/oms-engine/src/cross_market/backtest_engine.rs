use chrono::{DateTime, Utc, Duration, TimeZone};
use std::collections::HashMap;
use crate::cross_market::market_fabric::{FabricState, AssetFabricState};
use crate::cross_market::pattern_detector::{PatternDetector, PatternDetectorParams};
use crate::cross_market::noise_filter::NoiseFilterFabric;
use crate::cross_market::cross_layer_fusion::{CrossLayerFusion, FusionWeights};
use crate::cross_market::deterministic_engine::{DeterministicProfitEngine, ProfitEngineParams};
use crate::cross_market::time_bounded_router::{TimeBoundedRouter, RouterParams, RouteResult};
use crate::cross_market::fabric_guard::{FabricGuard, GuardParams};
use crate::cross_market::fabric_orchestrator::{FabricOrchestrator, OrchestratorParams};
use crate::cross_market::backtest_universe::{AssetConfig, AssetClass, default_universe};

#[derive(Debug, Clone)]
pub struct Tick {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub price: f64,
    pub volume: f64,
    pub bid: f64,
    pub ask: f64,
}

#[derive(Debug, Clone)]
pub struct ExecutedTrade {
    pub entry_time: DateTime<Utc>,
    pub exit_time: Option<DateTime<Utc>>,
    pub symbol: String,
    pub asset_class: AssetClass,
    pub direction: i32,        // 1 = long, -1 = short
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub quantity: f64,
    pub pnl: f64,
    pub exit_reason: String,   // "target", "stop", "timeout", "guard_halt"
    pub sharpe_at_entry: f64,
    pub predictability: f64,
    pub pattern_hash: u64,
    pub route_latency_us: u128,
}

#[derive(Debug, Clone)]
pub struct BacktestState {
    pub cash_cad: f64,
    pub positions: HashMap<String, f64>,
    pub equity_curve: Vec<(DateTime<Utc>, f64)>,
    pub trades: Vec<ExecutedTrade>,
    pub daily_returns: Vec<f64>,
    pub last_day: i64,
    pub peak_equity: f64,
    pub max_drawdown: f64,
}

impl BacktestState {
    pub fn new(cash_cad: f64) -> Self {
        Self {
            cash_cad,
            positions: HashMap::new(),
            equity_curve: vec![(Utc::now(), cash_cad)],
            trades: vec![],
            daily_returns: vec![],
            last_day: 0,
            peak_equity: cash_cad,
            max_drawdown: 0.0,
        }
    }

    pub fn equity(&self, prices: &HashMap<String, f64>) -> f64 {
        let mut equity = self.cash_cad;
        for (sym, qty) in &self.positions {
            if let Some(&price) = prices.get(sym) {
                equity += qty * price;
            }
        }
        equity
    }

    pub fn update_drawdown(&mut self, equity: f64) {
        if equity > self.peak_equity {
            self.peak_equity = equity;
        }
        let dd = (self.peak_equity - equity) / self.peak_equity;
        if dd > self.max_drawdown {
            self.max_drawdown = dd;
        }
    }
}

pub struct BacktestEngine {
    pub assets: Vec<AssetConfig>,
    pub state: BacktestState,
    pub orchestrator: FabricOrchestrator,
    pub price_history: HashMap<String, Vec<f64>>,
    pub volume_history: HashMap<String, Vec<f64>>,
    pub time_index: HashMap<String, usize>,
    pub open_trades: HashMap<String, ExecutedTrade>,
    pub params: BacktestParams,
}

#[derive(Debug, Clone)]
pub struct BacktestParams {
    pub start_balance_cad: f64,
    pub max_position_pct: f64,    // max 5% per trade
    pub commission_bps: f64,      // 5 bps per side
    pub slippage_bps: f64,        // 2 bps slippage
    pub target_multiplier: f64,   // 2.0x risk
    pub stop_multiplier: f64,     // 1.0x risk
    pub timeout_minutes: i64,     // 15 min max hold
    pub cad_fx_rates: HashMap<String, f64>,
}

impl Default for BacktestParams {
    fn default() -> Self {
        let mut fx = HashMap::new();
        fx.insert("USD".to_string(), 1.35);
        fx.insert("BTC-USD".to_string(), 1.35);
        fx.insert("ETH-USD".to_string(), 1.35);
        fx.insert("SOL-USD".to_string(), 1.35);
        fx.insert("XRP-USD".to_string(), 1.35);
        fx.insert("XAUUSD".to_string(), 1.35);
        fx.insert("XAGUSD".to_string(), 1.35);
        fx.insert("HG".to_string(), 1.35);
        fx.insert("CL".to_string(), 1.35);
        fx.insert("NG".to_string(), 1.35);
        fx.insert("ZC".to_string(), 1.35);
        fx.insert("ZS".to_string(), 1.35);
        fx.insert("AAPL".to_string(), 1.35);
        fx.insert("MSFT".to_string(), 1.35);
        fx.insert("TSLA".to_string(), 1.35);
        fx.insert("NVDA".to_string(), 1.35);
        fx.insert("AMZN".to_string(), 1.35);
        fx.insert("GOOGL".to_string(), 1.35);
        fx.insert("META".to_string(), 1.35);
        fx.insert("JPM".to_string(), 1.35);
        Self {
            start_balance_cad: 11_680.0,
            max_position_pct: 0.05,
            commission_bps: 5.0,
            slippage_bps: 2.0,
            target_multiplier: 2.0,
            stop_multiplier: 1.0,
            timeout_minutes: 15,
            cad_fx_rates: fx,
        }
    }
}

impl BacktestEngine {
    pub fn new() -> Self {
        let assets = default_universe();
        let params = BacktestParams::default();
        let orchestrator = FabricOrchestrator::new();
        let mut price_history = HashMap::new();
        let mut volume_history = HashMap::new();
        let mut time_index = HashMap::new();
        for a in &assets {
            price_history.insert(a.symbol.clone(), vec![]);
            volume_history.insert(a.symbol.clone(), vec![]);
            time_index.insert(a.symbol.clone(), 0);
        }
        Self {
            assets,
            state: BacktestState::new(params.start_balance_cad),
            orchestrator,
            price_history,
            volume_history,
            time_index,
            open_trades: HashMap::new(),
            params,
        }
    }

    /// Generate realistic synthetic price path using geometric Brownian motion with regime shifts
    fn generate_price_series(
        &self,
        asset: &AssetConfig,
        start_price: f64,
        n_bars: usize,
        seed: u64,
    ) -> Vec<f64> {
        let dt = 1.0 / 390.0; // 1-minute bars, ~390 per trading day
        let drift = 0.0; // Neutral drift for backtest fairness
        let vol = asset.daily_volatility / (390.0_f64).sqrt();
        let mut prices = vec![start_price];
        let mut rng = SimpleLcg::new(seed);

        // Add regime: first half lower vol, second half higher vol
        for i in 1..n_bars {
            let regime_mult = if i > n_bars / 2 { 1.5 } else { 1.0 };
            let z = rng.gaussian();
            let ret = drift * dt + vol * regime_mult * z * dt.sqrt();
            let new_price = prices.last().unwrap() * (1.0 + ret);
            let new_price = new_price.max(asset.tick_size);
            prices.push(new_price);
        }
        prices
    }

    fn generate_volume_series(&self, asset: &AssetConfig, n_bars: usize, seed: u64) -> Vec<f64> {
        let mut rng = SimpleLcg::new(seed + 1000);
        let mut volumes = vec![];
        for _ in 0..n_bars {
            let base = asset.avg_volume / 390.0; // per minute
            let noise = rng.gaussian().abs();
            let v = base * (0.5 + noise);
            volumes.push(v);
        }
        volumes
    }

    pub fn run_backtest(&mut self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> BacktestReport {
        let total_days = (end_date - start_date).num_days() as usize;
        let bars_per_day = 390; // US market minutes
        let total_bars = total_days * bars_per_day;

        println!("\n╔════════════════════════════════════════════════════════════════╗");
        println!("║      TRADERX FABRIC BACKTEST ENGINE v2.0                     ║");
        println!("╠════════════════════════════════════════════════════════════════╣");
        println!("║  Period: {} to {}                 ║", start_date.format("%Y-%m-%d"), end_date.format("%Y-%m-%d"));
        println!("║  Assets: {} (Stocks + Crypto + Metals + Commodities)           ║", self.assets.len());
        println!("║  Starting Balance: ${:,.2} CAD                                ║", self.params.start_balance_cad);
        println!("║  Bars to Simulate: ~{:>10} (~{} trading days)                  ║", total_bars, total_days);
        println!("╚════════════════════════════════════════════════════════════════╝\n");

        // Pre-generate all price/volume series
        let mut all_prices: HashMap<String, Vec<f64>> = HashMap::new();
        let mut all_volumes: HashMap<String, Vec<f64>> = HashMap::new();

        for (idx, asset) in self.assets.iter().enumerate() {
            let seed = (idx + 1) as u64 * 12345;
            let prices = self.generate_price_series(asset, asset.base_price, total_bars, seed);
            let volumes = self.generate_volume_series(asset, total_bars, seed);
            all_prices.insert(asset.symbol.clone(), prices);
            all_volumes.insert(asset.symbol.clone(), volumes);
        }

        let mut current_time = start_date;
        let mut bar_count = 0;
        let mut daily_snapshots: Vec<(DateTime<Utc>, f64)> = vec![];
        let mut last_reported_day = -1i64;

        // MAIN BACKTEST LOOP
        while current_time < end_date {
            // Check if market is open (simplified: 9:30-16:00 EST, skip weekends)
            let weekday = current_time.weekday();
            let hour = current_time.hour();
            let minute = current_time.minute();
            let is_weekend = weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun;
            let market_hour = (hour >= 14 && hour < 21) || (hour == 13 && minute >= 30); // UTC-5 offset approx

            if !is_weekend && market_hour {
                // Generate ticks for each asset
                let mut current_prices: HashMap<String, f64> = HashMap::new();

                for asset in &self.assets {
                    let sym = &asset.symbol;
                    let idx = *self.time_index.get(sym).unwrap_or(&0);
                    let prices = all_prices.get(sym).unwrap();
                    let volumes = all_volumes.get(sym).unwrap();

                    if idx < prices.len() {
                        let price = prices[idx];
                        let volume = volumes[idx];
                        let bid = price * (1.0 - asset.spread_bps / 10000.0 / 2.0);
                        let ask = price * (1.0 + asset.spread_bps / 10000.0 / 2.0);

                        // Feed to orchestrator
                        let _ = self.orchestrator.ingest_tick(sym, price, volume, current_time);

                        // Update price history for pattern detection
                        self.price_history.entry(sym.clone()).or_default().push(price);
                        self.volume_history.entry(sym.clone()).or_default().push(volume);
                        if self.price_history[sym].len() > 100 {
                            self.price_history.get_mut(sym).unwrap().remove(0);
                        }
                        if self.volume_history[sym].len() > 100 {
                            self.volume_history.get_mut(sym).unwrap().remove(0);
                        }

                        // Check open trades for exits
                        if let Some(open) = self.open_trades.get(sym) {
                            let entry = open.entry_price;
                            let target = entry * (1.0 + open.direction as f64 * self.params.target_multiplier * 0.01);
                            let stop = entry * (1.0 - open.direction as f64 * self.params.stop_multiplier * 0.01);
                            let elapsed = current_time.signed_duration_since(open.entry_time).num_minutes();

                            let mut exited = false;
                            let mut exit_price = price;
                            let mut exit_reason = String::new();

                            if open.direction == 1 && price >= target {
                                exited = true; exit_reason = "target".to_string();
                            } else if open.direction == 1 && price <= stop {
                                exited = true; exit_reason = "stop".to_string();
                            } else if open.direction == -1 && price <= target {
                                exited = true; exit_reason = "target".to_string();
                            } else if open.direction == -1 && price >= stop {
                                exited = true; exit_reason = "stop".to_string();
                            } else if elapsed >= self.params.timeout_minutes {
                                exited = true; exit_reason = "timeout".to_string();
                            }

                            if exited {
                                let pnl = (exit_price - entry) * open.quantity * open.direction as f64;
                                let commission = entry * open.quantity * self.params.commission_bps / 10000.0
                                    + exit_price * open.quantity * self.params.commission_bps / 10000.0;
                                let slippage = entry * open.quantity * self.params.slippage_bps / 10000.0
                                    + exit_price * open.quantity * self.params.slippage_bps / 10000.0;
                                let net_pnl = pnl - commission - slippage;

                                self.state.cash_cad += (exit_price * open.quantity) - commission - slippage;
                                if open.direction == -1 {
                                    // Short: we sold short at entry, now buying back
                                    self.state.cash_cad -= exit_price * open.quantity;
                                }
                                self.state.positions.remove(sym);

                                let mut closed_trade = open.clone();
                                closed_trade.exit_time = Some(current_time);
                                closed_trade.exit_price = Some(exit_price);
                                closed_trade.pnl = net_pnl;
                                closed_trade.exit_reason = exit_reason;
                                self.state.trades.push(closed_trade);
                                self.open_trades.remove(sym);
                            }
                        }

                        current_prices.insert(sym.clone(), price);
                        *self.time_index.get_mut(sym).unwrap() = idx + 1;
                    }
                }

                // Run pattern detection and trade entry decisions every 5 minutes
                if bar_count % 5 == 0 {
                    for asset in &self.assets {
                        let sym = &asset.symbol;
                        let prices = self.price_history.get(sym).cloned().unwrap_or_default();
                        let volumes = self.volume_history.get(sym).cloned().unwrap_or_default();

                        if prices.len() >= 20 {
                            let result = self.orchestrator.process_asset(sym, &prices, &volumes, current_time);
                            if let Some(decision) = result.trade_decision {
                                // Only enter if no open trade on this symbol
                                if !self.open_trades.contains_key(sym) && self.state.cash_cad > 1000.0 {
                                    let fx = self.params.cad_fx_rates.get(sym).copied().unwrap_or(1.0);
                                    let price = *current_prices.get(sym).unwrap_or(&asset.base_price);
                                    let max_notional = self.state.cash_cad * self.params.max_position_pct;
                                    let quantity = (max_notional / (price * fx)).max(1.0);

                                    // Apply slippage and commission on entry
                                    let slippage = price * self.params.slippage_bps / 10000.0;
                                    let commission = price * quantity * self.params.commission_bps / 10000.0;
                                    let entry_cost = price * quantity * fx + commission + slippage * quantity * fx;

                                    if entry_cost <= self.state.cash_cad * 0.95 {
                                        let direction = if decision.action == crate::cross_market::deterministic_engine::Action::Long { 1 } else { -1 };
                                        self.state.cash_cad -= entry_cost;
                                        *self.state.positions.entry(sym.clone()).or_insert(0.0) += quantity * direction as f64;

                                        let trade = ExecutedTrade {
                                            entry_time: current_time,
                                            exit_time: None,
                                            symbol: sym.clone(),
                                            asset_class: asset.asset_class.clone(),
                                            direction,
                                            entry_price: price,
                                            exit_price: None,
                                            quantity,
                                            pnl: 0.0,
                                            exit_reason: String::new(),
                                            sharpe_at_entry: decision.sharpe_estimate,
                                            predictability: decision.predictability,
                                            pattern_hash: decision.hash,
                                            route_latency_us: decision.route_latency_us.unwrap_or(0),
                                        };
                                        self.open_trades.insert(sym.clone(), trade);
                                    }
                                }
                            }
                        }
                    }
                }

                // Daily snapshot
                let day = current_time.ordinal0() as i64;
                if day != last_reported_day {
                    let equity = self.state.equity(&current_prices);
                    self.state.update_drawdown(equity);
                    daily_snapshots.push((current_time, equity));
                    last_reported_day = day;
                }
            }

            current_time = current_time + Duration::minutes(1);
            bar_count += 1;

            if bar_count % 5000 == 0 {
                let equity = self.state.equity(&current_prices);
                println!("  [Bar {}] Equity: ${:,.2} | Open: {} | Closed: {} | Max DD: {:.2}%",
                    bar_count, equity, self.open_trades.len(), self.state.trades.len(),
                    self.state.max_drawdown * 100.0);
            }
        }

        // Close any remaining open trades at final prices
        let mut final_prices: HashMap<String, f64> = HashMap::new();
        for asset in &self.assets {
            let sym = &asset.symbol;
            let prices = all_prices.get(sym).unwrap();
            if !prices.is_empty() {
                final_prices.insert(sym.clone(), *prices.last().unwrap());
            }
        }

        let open_symbols: Vec<String> = self.open_trades.keys().cloned().collect();
        for sym in open_symbols {
            if let Some(open) = self.open_trades.remove(&sym) {
                let exit_price = *final_prices.get(&sym).unwrap_or(&open.entry_price);
                let pnl = (exit_price - open.entry_price) * open.quantity * open.direction as f64;
                let commission = open.entry_price * open.quantity * self.params.commission_bps / 10000.0
                    + exit_price * open.quantity * self.params.commission_bps / 10000.0;
                let slippage = open.entry_price * open.quantity * self.params.slippage_bps / 10000.0
                    + exit_price * open.quantity * self.params.slippage_bps / 10000.0;
                let net_pnl = pnl - commission - slippage;

                self.state.cash_cad += (exit_price * open.quantity) - commission - slippage;
                if open.direction == -1 {
                    self.state.cash_cad -= exit_price * open.quantity;
                }
                self.state.positions.remove(&sym);

                let mut closed = open.clone();
                closed.exit_time = Some(current_time);
                closed.exit_price = Some(exit_price);
                closed.pnl = net_pnl;
                closed.exit_reason = "backtest_end".to_string();
                self.state.trades.push(closed);
            }
        }

        self.generate_report(daily_snapshots)
    }

    fn generate_report(&self, snapshots: Vec<(DateTime<Utc>, f64)>) -> BacktestReport {
        let total_return = (self.state.cash_cad - self.params.start_balance_cad) / self.params.start_balance_cad;
        let winning = self.state.trades.iter().filter(|t| t.pnl > 0.0).count();
        let losing = self.state.trades.iter().filter(|t| t.pnl < 0.0).count();
        let total_trades = self.state.trades.len();
        let win_rate = if total_trades > 0 { winning as f64 / total_trades as f64 } else { 0.0 };

        let gross_profit: f64 = self.state.trades.iter().filter(|t| t.pnl > 0.0).map(|t| t.pnl).sum();
        let gross_loss: f64 = self.state.trades.iter().filter(|t| t.pnl < 0.0).map(|t| t.pnl.abs()).sum();
        let profit_factor = if gross_loss > 0.0 { gross_profit / gross_loss } else { 0.0 };

        let returns: Vec<f64> = snapshots.windows(2).map(|w| {
            (w[1].1 - w[0].1) / w[0].1
        }).collect();
        let mean_ret = if !returns.is_empty() { returns.iter().sum::<f64>() / returns.len() as f64 } else { 0.0 };
        let var_ret = if returns.len() > 1 {
            returns.iter().map(|r| (r - mean_ret).powi(2)).sum::<f64>() / (returns.len() - 1) as f64
        } else { 0.0 };
        let sharpe = if var_ret > 0.0 { mean_ret / var_ret.sqrt() * (252.0_f64).sqrt() } else { 0.0 };

        let by_asset_class: HashMap<String, (f64, usize)> = {
            let mut m = HashMap::new();
            for t in &self.state.trades {
                let entry = m.entry(t.asset_class.to_string()).or_insert((0.0, 0));
                entry.0 += t.pnl;
                entry.1 += 1;
            }
            m
        };

        BacktestReport {
            start_balance: self.params.start_balance_cad,
            end_balance: self.state.cash_cad,
            total_return_pct: total_return * 100.0,
            total_trades,
            winning_trades: winning,
            losing_trades: losing,
            win_rate_pct: win_rate * 100.0,
            profit_factor,
            max_drawdown_pct: self.state.max_drawdown * 100.0,
            sharpe_annualized: sharpe,
            by_asset_class,
            all_trades: self.state.trades.clone(),
            snapshots,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BacktestReport {
    pub start_balance: f64,
    pub end_balance: f64,
    pub total_return_pct: f64,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate_pct: f64,
    pub profit_factor: f64,
    pub max_drawdown_pct: f64,
    pub sharpe_annualized: f64,
    pub by_asset_class: HashMap<String, (f64, usize)>,
    pub all_trades: Vec<ExecutedTrade>,
    pub snapshots: Vec<(DateTime<Utc>, f64)>,
}

impl BacktestReport {
    pub fn print_summary(&self) {
        println!("\n╔════════════════════════════════════════════════════════════════╗");
        println!("║           BACKTEST RESULTS SUMMARY                             ║");
        println!("╠════════════════════════════════════════════════════════════════╣");
        println!("║  Start Balance:        ${:>15,.2} CAD                        ║", self.start_balance);
        println!("║  End Balance:          ${:>15,.2} CAD                        ║", self.end_balance);
        println!("║  Total Return:         {:>15.2}%                               ║", self.total_return_pct);
        println!("║  Total Trades:         {:>15}                                  ║", self.total_trades);
        println!("║  Winning Trades:       {:>15}                                  ║", self.winning_trades);
        println!("║  Losing Trades:        {:>15}                                  ║", self.losing_trades);
        println!("║  Win Rate:             {:>15.2}%                               ║", self.win_rate_pct);
        println!("║  Profit Factor:        {:>15.2}                                  ║", self.profit_factor);
        println!("║  Max Drawdown:         {:>15.2}%                               ║", self.max_drawdown_pct);
        println!("║  Sharpe (Annualized):   {:>15.2}                                  ║", self.sharpe_annualized);
        println!("╠════════════════════════════════════════════════════════════════╣");
        println!("║  P&L BY ASSET CLASS                                          ║");
        for (class, (pnl, count)) in &self.by_asset_class {
            println!("║    {:<12}  Trades: {:>4}  P&L: ${:>10,.2} CAD            ║", class, count, pnl);
        }
        println!("╚════════════════════════════════════════════════════════════════╝\n");
    }

    pub fn print_all_trades(&self) {
        println!("\n╔════════════════════════════════════════════════════════════════════════════════════════════╗");
        println!("║                                       ALL TRADES                                           ║");
        println!("╠════════════════════╦════════════╦══════════╦══════════╦══════════╦══════════╦════════════╣");
        println!("║ Symbol             ║ Direction  ║ Entry    ║ Exit     ║ P&L CAD  ║ Reason   ║ Sharpe     ║");
        println!("╠════════════════════╬════════════╬══════════╬══════════╬══════════╬══════════╬════════════╣");
        for t in &self.all_trades {
            let dir = if t.direction == 1 { "LONG " } else { "SHORT" };
            let ep = t.entry_price;
            let xp = t.exit_price.unwrap_or(0.0);
            println!("║ {:<18} ║ {:<10} ║ {:>8.2} ║ {:>8.2} ║ {:>8.2} ║ {:<8} ║ {:>10.2} ║",
                t.symbol, dir, ep, xp, t.pnl, t.exit_reason, t.sharpe_at_entry);
        }
        println!("╚════════════════════╩════════════╩══════════╩══════════╩══════════╩══════════╩════════════╝");
    }
}

// Simple LCG for deterministic, reproducible pseudo-random numbers
struct SimpleLcg {
    state: u64,
}

impl SimpleLcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    // Box-Muller transform for standard normal
    fn gaussian(&mut self) -> f64 {
        let u1 = (self.next() as f64) / (u64::MAX as f64);
        let u2 = (self.next() as f64) / (u64::MAX as f64);
        let r = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * std::f64::consts::PI * u2;
        r * theta.cos()
    }
}

