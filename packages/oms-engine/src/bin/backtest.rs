use chrono::{DateTime, Utc, Duration, TimeZone};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use oms_engine::cross_market::{
    FabricOrchestrator, OrchestratorParams,
    FabricState, AssetFabricState,
    TradeDecision, GuardedRoute, PathType,
};

#[derive(Debug)]
enum BacktestError {
    InvalidDateTime(String),
    MissingPriceSeries(String),
    IoError(String),
}

impl fmt::Display for BacktestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BacktestError::InvalidDateTime(msg) => write!(f, "Invalid datetime: {}", msg),
            BacktestError::MissingPriceSeries(symbol) => write!(f, "Missing price series for {}", symbol),
            BacktestError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl Error for BacktestError {}

impl From<std::io::Error> for BacktestError {
    fn from(e: std::io::Error) -> Self {
        BacktestError::IoError(e.to_string())
    }
}

#[derive(Debug, Clone)]
struct Asset {
    symbol: String, class: String,
    base_price: f64, vol: f64, spread_bps: f64,
}

#[derive(Debug, Clone)]
struct OpenTrade {
    symbol: String, direction: i8,
    entry: f64, quantity: f64,
    target: f64, stop: f64,
    entry_time: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct ClosedTrade {
    symbol: String, class: String,
    direction: i8, entry: f64, exit: f64,
    quantity: f64, pnl_cad: f64, fee_cad: f64,
    reason: String,
    entry_time: DateTime<Utc>, exit_time: DateTime<Utc>,
    path: String,
    sharpe: f64,
}

fn lcg(seed: u64) -> u64 {
    seed.wrapping_mul(1103515245).wrapping_add(12345) & 0x7fffffff
}

fn gen_prices(base: f64, vol: f64, n: usize, seed: u64) -> Vec<f64> {
    let dt = 1.0 / (78.0 * 252.0);
    let mut prices = vec![base];
    let mut s = seed;
    for _ in 1..n {
        s = lcg(s);
        let z = (s as f64 / 0x7fffffff as f64) * 2.0 - 1.0;
        let drift = 0.0;
        let ret = drift * dt + vol * z * dt.sqrt();
        let last_price = *prices.last().unwrap_or(&base);
        prices.push(last_price * (1.0 + ret).max(0.1));
    }
    prices
}

fn gen_volumes(n: usize, seed: u64) -> Vec<f64> {
    let mut vols = vec![];
    let mut s = seed;
    for _ in 0..n {
        s = lcg(s);
        vols.push(1000.0 + (s as f64 / 0x7fffffff as f64) * 9000.0);
    }
    vols
}

fn build_universe() -> Vec<Asset> {
    let mut assets = vec![
        Asset{symbol:"AAPL".into(),class:"Stock".into(),base_price:195.0,vol:0.22,spread_bps:0.5},
        Asset{symbol:"MSFT".into(),class:"Stock".into(),base_price:415.0,vol:0.20,spread_bps:0.5},
        Asset{symbol:"TSLA".into(),class:"Stock".into(),base_price:175.0,vol:0.55,spread_bps:1.0},
        Asset{symbol:"NVDA".into(),class:"Stock".into(),base_price:138.0,vol:0.45,spread_bps:0.8},
        Asset{symbol:"AMZN".into(),class:"Stock".into(),base_price:200.0,vol:0.28,spread_bps:0.6},
        Asset{symbol:"GOOGL".into(),class:"Stock".into(),base_price:178.0,vol:0.25,spread_bps:0.6},
        Asset{symbol:"META".into(),class:"Stock".into(),base_price:595.0,vol:0.32,spread_bps:0.7},
        Asset{symbol:"JPM".into(),class:"Stock".into(),base_price:245.0,vol:0.18,spread_bps:0.4},
        Asset{symbol:"V".into(),class:"Stock".into(),base_price:325.0,vol:0.20,spread_bps:0.5},
        Asset{symbol:"JNJ".into(),class:"Stock".into(),base_price:155.0,vol:0.15,spread_bps:0.4},
        Asset{symbol:"WMT".into(),class:"Stock".into(),base_price:95.0,vol:0.16,spread_bps:0.4},
        Asset{symbol:"PG".into(),class:"Stock".into(),base_price:170.0,vol:0.14,spread_bps:0.3},
        Asset{symbol:"UNH".into(),class:"Stock".into(),base_price:520.0,vol:0.22,spread_bps:0.6},
        Asset{symbol:"HD".into(),class:"Stock".into(),base_price:380.0,vol:0.18,spread_bps:0.5},
        Asset{symbol:"BAC".into(),class:"Stock".into(),base_price:42.0,vol:0.28,spread_bps:0.6},
        Asset{symbol:"MA".into(),class:"Stock".into(),base_price:485.0,vol:0.22,spread_bps:0.5},
        Asset{symbol:"ABBV".into(),class:"Stock".into(),base_price:210.0,vol:0.18,spread_bps:0.4},
        Asset{symbol:"PFE".into(),class:"Stock".into(),base_price:28.0,vol:0.20,spread_bps:0.5},
        Asset{symbol:"KO".into(),class:"Stock".into(),base_price:68.0,vol:0.14,spread_bps:0.3},
        Asset{symbol:"PEP".into(),class:"Stock".into(),base_price:155.0,vol:0.15,spread_bps:0.4},
        Asset{symbol:"COST".into(),class:"Stock".into(),base_price:920.0,vol:0.18,spread_bps:0.5},
        Asset{symbol:"DIS".into(),class:"Stock".into(),base_price:105.0,vol:0.25,spread_bps:0.6},
        Asset{symbol:"NFLX".into(),class:"Stock".into(),base_price:880.0,vol:0.32,spread_bps:0.8},
        Asset{symbol:"AMD".into(),class:"Stock".into(),base_price:105.0,vol:0.48,spread_bps:0.8},
        Asset{symbol:"INTC".into(),class:"Stock".into(),base_price:22.0,vol:0.35,spread_bps:0.7},
        Asset{symbol:"CRM".into(),class:"Stock".into(),base_price:285.0,vol:0.28,spread_bps:0.6},
        Asset{symbol:"ADBE".into(),class:"Stock".into(),base_price:420.0,vol:0.30,spread_bps:0.6},
        Asset{symbol:"NKE".into(),class:"Stock".into(),base_price:78.0,vol:0.25,spread_bps:0.6},
    ];
    let cryptos = vec![
        Asset{symbol:"BTC-USD".into(),class:"Crypto".into(),base_price:88000.0,vol:0.65,spread_bps:2.0},
        Asset{symbol:"ETH-USD".into(),class:"Crypto".into(),base_price:2150.0,vol:0.78,spread_bps:3.0},
        Asset{symbol:"SOL-USD".into(),class:"Crypto".into(),base_price:142.0,vol:0.95,spread_bps:5.0},
        Asset{symbol:"XRP-USD".into(),class:"Crypto".into(),base_price:2.35,vol:1.10,spread_bps:8.0},
        Asset{symbol:"ADA-USD".into(),class:"Crypto".into(),base_price:0.72,vol:1.15,spread_bps:10.0},
        Asset{symbol:"DOGE-USD".into(),class:"Crypto".into(),base_price:0.18,vol:1.20,spread_bps:12.0},
        Asset{symbol:"DOT-USD".into(),class:"Crypto".into(),base_price:4.50,vol:0.95,spread_bps:8.0},
        Asset{symbol:"AVAX-USD".into(),class:"Crypto".into(),base_price:22.0,vol:1.05,spread_bps:9.0},
        Asset{symbol:"MATIC-USD".into(),class:"Crypto".into(),base_price:0.38,vol:1.10,spread_bps:10.0},
        Asset{symbol:"LTC-USD".into(),class:"Crypto".into(),base_price:85.0,vol:0.80,spread_bps:5.0},
        Asset{symbol:"BCH-USD".into(),class:"Crypto".into(),base_price:350.0,vol:0.85,spread_bps:6.0},
        Asset{symbol:"LINK-USD".into(),class:"Crypto".into(),base_price:14.0,vol:0.90,spread_bps:7.0},
        Asset{symbol:"UNI-USD".into(),class:"Crypto".into(),base_price:6.50,vol:0.95,spread_bps:8.0},
        Asset{symbol:"ATOM-USD".into(),class:"Crypto".into(),base_price:4.20,vol:1.00,spread_bps:9.0},
        Asset{symbol:"ETC-USD".into(),class:"Crypto".into(),base_price:18.0,vol:0.88,spread_bps:7.0},
        Asset{symbol:"XLM-USD".into(),class:"Crypto".into(),base_price:0.28,vol:1.05,spread_bps:10.0},
        Asset{symbol:"VET-USD".into(),class:"Crypto".into(),base_price:0.022,vol:1.20,spread_bps:15.0},
        Asset{symbol:"FIL-USD".into(),class:"Crypto".into(),base_price:3.20,vol:1.00,spread_bps:10.0},
        Asset{symbol:"TRX-USD".into(),class:"Crypto".into(),base_price:0.25,vol:0.85,spread_bps:8.0},
        Asset{symbol:"EOS-USD".into(),class:"Crypto".into(),base_price:0.65,vol:0.95,spread_bps:9.0},
        Asset{symbol:"AAVE-USD".into(),class:"Crypto".into(),base_price:155.0,vol:0.90,spread_bps:7.0},
        Asset{symbol:"ALGO-USD".into(),class:"Crypto".into(),base_price:0.18,vol:1.05,spread_bps:11.0},
        Asset{symbol:"ICP-USD".into(),class:"Crypto".into(),base_price:6.80,vol:0.92,spread_bps:8.0},
        Asset{symbol:"MANA-USD".into(),class:"Crypto".into(),base_price:0.28,vol:1.10,spread_bps:12.0},
        Asset{symbol:"SAND-USD".into(),class:"Crypto".into(),base_price:0.25,vol:1.12,spread_bps:12.0},
        Asset{symbol:"AXS-USD".into(),class:"Crypto".into(),base_price:4.50,vol:1.00,spread_bps:9.0},
        Asset{symbol:"FTM-USD".into(),class:"Crypto".into(),base_price:0.55,vol:1.08,spread_bps:11.0},
        Asset{symbol:"GRT-USD".into(),class:"Crypto".into(),base_price:0.13,vol:1.15,spread_bps:13.0},
        Asset{symbol:"NEAR-USD".into(),class:"Crypto".into(),base_price:2.35,vol:0.95,spread_bps:8.0},
    ];
    let metals = vec![
        Asset{symbol:"XAUUSD".into(),class:"Metal".into(),base_price:2850.0,vol:0.14,spread_bps:1.5},
        Asset{symbol:"XAGUSD".into(),class:"Metal".into(),base_price:32.0,vol:0.22,spread_bps:2.5},
        Asset{symbol:"XPTUSD".into(),class:"Metal".into(),base_price:980.0,vol:0.18,spread_bps:2.0},
        Asset{symbol:"XPDUSD".into(),class:"Metal".into(),base_price:950.0,vol:0.20,spread_bps:2.5},
        Asset{symbol:"HG".into(),class:"Metal".into(),base_price:4.35,vol:0.18,spread_bps:3.0},
    ];
    let commodities = vec![
        Asset{symbol:"CL".into(),class:"Commodity".into(),base_price:68.0,vol:0.32,spread_bps:2.0},
        Asset{symbol:"NG".into(),class:"Commodity".into(),base_price:3.40,vol:0.45,spread_bps:4.0},
        Asset{symbol:"ZC".into(),class:"Commodity".into(),base_price:4.80,vol:0.20,spread_bps:2.5},
        Asset{symbol:"ZS".into(),class:"Commodity".into(),base_price:10.20,vol:0.22,spread_bps:3.0},
        Asset{symbol:"ZW".into(),class:"Commodity".into(),base_price:5.50,vol:0.20,spread_bps:2.5},
        Asset{symbol:"KC".into(),class:"Commodity".into(),base_price:2.10,vol:0.25,spread_bps:3.0},
        Asset{symbol:"CC".into(),class:"Commodity".into(),base_price:3.80,vol:0.22,spread_bps:3.0},
        Asset{symbol:"CT".into(),class:"Commodity".into(),base_price:72.0,vol:0.18,spread_bps:2.5},
    ];
    let forex = vec![
        Asset{symbol:"EURUSD".into(),class:"FX".into(),base_price:1.0850,vol:0.08,spread_bps:0.3},
        Asset{symbol:"GBPUSD".into(),class:"FX".into(),base_price:1.2650,vol:0.10,spread_bps:0.4},
        Asset{symbol:"USDJPY".into(),class:"FX".into(),base_price:151.20,vol:0.09,spread_bps:0.3},
        Asset{symbol:"USDCHF".into(),class:"FX".into(),base_price:0.9050,vol:0.09,spread_bps:0.4},
        Asset{symbol:"AUDUSD".into(),class:"FX".into(),base_price:0.6320,vol:0.11,spread_bps:0.5},
        Asset{symbol:"USDCAD".into(),class:"FX".into(),base_price:1.4320,vol:0.09,spread_bps:0.4},
        Asset{symbol:"NZDUSD".into(),class:"FX".into(),base_price:0.5850,vol:0.12,spread_bps:0.5},
        Asset{symbol:"USDCNH".into(),class:"FX".into(),base_price:7.2450,vol:0.10,spread_bps:0.6},
    ];
    assets.extend(cryptos);
    assets.extend(metals);
    assets.extend(commodities);
    assets.extend(forex);
    assets
}

fn main() -> Result<(), Box<dyn Error>> {
    let start = Utc.with_ymd_and_hms(2026, 2, 20, 14, 30, 0)
        .ok_or_else(|| BacktestError::InvalidDateTime("Invalid start datetime".to_string()))?;
    let end = Utc.with_ymd_and_hms(2026, 5, 2, 21, 0, 0)
        .ok_or_else(|| BacktestError::InvalidDateTime("Invalid end datetime".to_string()))?;
    let balance_cad: f64 = 2_700.0;

    println!("\nTRADERX FABRIC LIVE SIMULATION v3.0");
    println!("Period: 2026-02-20 to 2026-05-02 | Balance: ${:.2} CAD\n", balance_cad);

    let params = OrchestratorParams::default();
    let mut orchestrator = FabricOrchestrator::new(params);
    let assets = build_universe();

    let days = (end - start).num_days() as usize;
    let bars = days * 78;
    println!("Pre-generating {} bars across {} assets...", bars, assets.len());

    let mut price_series: HashMap<String, Vec<f64>> = HashMap::new();
    let mut volume_series: HashMap<String, Vec<f64>> = HashMap::new();
    for (idx, asset) in assets.iter().enumerate() {
        let seed = ((idx + 1) as u64).wrapping_mul(12345);
        price_series.insert(asset.symbol.clone(), gen_prices(asset.base_price, asset.vol, bars, seed));
        volume_series.insert(asset.symbol.clone(), gen_volumes(bars, seed + 999));
    }

    let mut cash = balance_cad;
    let mut open_trades: Vec<OpenTrade> = vec![];
    let mut closed_trades: Vec<ClosedTrade> = vec![];
    let mut equity_snapshots: Vec<(DateTime<Utc>, f64)> = vec![];
    let mut peak = balance_cad;
    let mut max_dd = 0.0_f64;
    let mut bar_idx: usize = 0;
    let mut current_time = start;

    let mut total_patterns: usize = 0;
    let mut total_fused: usize = 0;
    let mut total_decisions: usize = 0;
    let mut total_allowed: usize = 0;
    let mut total_rejected: usize = 0;

    while current_time < end {
        let wd = current_time.weekday();
        let is_weekend = wd == chrono::Weekday::Sat || wd == chrono::Weekday::Sun;
        let h = current_time.hour();
        let m = current_time.minute();
        let market_open = (h == 14 && m >= 30) || (h > 14 && h < 21);
        if is_weekend || !market_open {
            current_time += Duration::minutes(5);
            continue;
        }

        let mut prices_now: HashMap<String, f64> = HashMap::new();
        for asset in &assets {
            let sym = &asset.symbol;
            let price_vec = price_series.get(sym)
                .ok_or_else(|| BacktestError::MissingPriceSeries(sym.clone()))?;
            let vol_vec = volume_series.get(sym)
                .ok_or_else(|| BacktestError::MissingPriceSeries(format!("volume for {}", sym)))?;
            let idx = bar_idx.min(price_vec.len() - 1);
            let p = price_vec[idx];
            let v = vol_vec[idx];
            let high = p * (1.0 + asset.spread_bps / 20000.0);
            let low = p * (1.0 - asset.spread_bps / 20000.0);
            orchestrator.ingest_ohlcv(sym, high, low, p, v);
            prices_now.insert(sym.clone(), p);
        }

        let mut still_open: Vec<OpenTrade> = vec![];
        for ot in open_trades {
            let p = *prices_now.get(&ot.symbol).unwrap_or(&ot.entry);
            let elapsed = current_time.signed_duration_since(ot.entry_time).num_minutes();
            let mut exited = false;
            let mut reason = String::new();
            let mut exit_price = p;
            if ot.direction == 1 && p >= ot.target { exited = true; reason = "target".into(); }
            else if ot.direction == 1 && p <= ot.stop { exited = true; reason = "stop".into(); exit_price = ot.stop; }
            else if ot.direction == -1 && p <= ot.target { exited = true; reason = "target".into(); }
            else if ot.direction == -1 && p >= ot.stop { exited = true; reason = "stop".into(); exit_price = ot.stop; }
            else if elapsed >= 15 { exited = true; reason = "timeout".into(); }
            if exited {
                let notional = ot.quantity * exit_price;
                let pnl = (exit_price - ot.entry) * ot.quantity * (ot.direction as f64);
                let fee = notional * 0.0005;
                let slip = notional * 0.0002;
                let net_pnl = pnl - fee - slip;
                cash += net_pnl;
                let class = assets.iter().find(|a| a.symbol == ot.symbol).map(|a| a.class.clone()).unwrap_or_default();
                closed_trades.push(ClosedTrade {
                    symbol: ot.symbol.clone(), class,
                    direction: ot.direction, entry: ot.entry, exit: exit_price,
                    quantity: ot.quantity, pnl_cad: net_pnl, fee_cad: fee + slip,
                    reason: reason.clone(),
                    entry_time: ot.entry_time, exit_time: current_time,
                    path: "Fast".into(),
                    sharpe: 0.0,
                });
                orchestrator.record_execution_pnl(net_pnl);
            } else { still_open.push(ot); }
        }
        open_trades = still_open;

        if bar_idx > 20 {
            let mut fabric_assets = HashMap::new();
            for asset in &assets {
                let p = *prices_now.get(&asset.symbol).unwrap_or(&asset.base_price);
                let high = p * (1.0 + asset.spread_bps / 20000.0);
                let low = p * (1.0 - asset.spread_bps / 20000.0);
                fabric_assets.insert(asset.symbol.clone(), AssetFabricState {
                    symbol: asset.symbol.clone(),
                    last_price: p, bid: low, ask: high,
                    volume_24h: volume_series[&asset.symbol][bar_idx.min(volume_series[&asset.symbol].len() - 1)] * 1000.0,
                    volatility_1h: asset.vol * p,
                    spread_bps: asset.spread_bps,
                    timestamp: current_time,
                    health_score: 1.0,
                    deterministic_hash: format!("hash_{}", asset.symbol),
                });
            }
            let fabric = FabricState { assets: fabric_assets, portfolio_value: cash };
            let result = orchestrator.tick(&fabric);
            total_patterns += result.patterns_detected;
            total_fused += result.signals_fused;
            total_decisions += result.decisions_made;
            total_allowed += result.allowed.len();
            total_rejected += result.rejected.len();

            for gr in result.allowed {
                let d = &gr.route.decision;
                let sym = d.symbol.clone();
                if open_trades.iter().any(|t| t.symbol == sym) { continue; }
                let price = *prices_now.get(&sym).unwrap_or(&d.entry_price);
                let max_notional = cash * 0.05;
                let qty = (max_notional / price).max(1.0);
                let cost = qty * price * 1.0007;
                if cost < cash * 0.95 {
                    cash -= cost;
                    open_trades.push(OpenTrade {
                        symbol: sym, direction: d.direction,
                        entry: price, quantity: qty,
                        target: d.target_price, stop: d.stop_loss,
                        entry_time: current_time,
                    });
                }
            }
        }

        let open_value: f64 = open_trades.iter().map(|t| {
            let p = *prices_now.get(&t.symbol).unwrap_or(&t.entry);
            t.quantity * p
        }).sum();
        let equity = cash + open_value;
        if equity > peak { peak = equity; }
        let dd = (peak - equity) / peak;
        if dd > max_dd { max_dd = dd; }
        if bar_idx % 78 == 0 { equity_snapshots.push((current_time, equity)); }

        bar_idx += 1;
        current_time += Duration::minutes(5);

        if bar_idx % 5000 == 0 {
            println!("  [Bar {:>6}] Equity: ${:>12,.2} | Open: {:>2} | Closed: {:>3} | MaxDD: {:>5.2}%",
                bar_idx, equity, open_trades.len(), closed_trades.len(), max_dd * 100.0);
        }
    }

    for ot in open_trades {
        let p = *price_series[&ot.symbol].last().unwrap_or(&ot.entry);
        let notional = ot.quantity * p;
        let pnl = (p - ot.entry) * ot.quantity * (ot.direction as f64);
        let fee = notional * 0.0005;
        let slip = notional * 0.0002;
        let net_pnl = pnl - fee - slip;
        cash += net_pnl;
        let class = assets.iter().find(|a| a.symbol == ot.symbol).map(|a| a.class.clone()).unwrap_or_default();
        closed_trades.push(ClosedTrade {
            symbol: ot.symbol.clone(), class,
            direction: ot.direction, entry: ot.entry, exit: p,
            quantity: ot.quantity, pnl_cad: net_pnl, fee_cad: fee + slip,
            reason: "end_of_session".into(),
            entry_time: ot.entry_time, exit_time: end,
            path: "Fast".into(),
            sharpe: 0.0,
        });
    }

    let final_equity = cash;
    let total_return = final_equity - balance_cad;
    let total_return_pct = (total_return / balance_cad) * 100.0;
    let win_trades: Vec<&ClosedTrade> = closed_trades.iter().filter(|t| t.pnl_cad > 0.0).collect();
    let lose_trades: Vec<&ClosedTrade> = closed_trades.iter().filter(|t| t.pnl_cad <= 0.0).collect();
    let win_rate = if !closed_trades.is_empty() { win_trades.len() as f64 / closed_trades.len() as f64 * 100.0 } else { 0.0 };
    let gross_profit: f64 = win_trades.iter().map(|t| t.pnl_cad).sum();
    let gross_loss: f64 = lose_trades.iter().map(|t| t.pnl_cad).sum();
    let profit_factor = if gross_loss.abs() > 0.01 { gross_profit / gross_loss.abs() } else { f64::INFINITY };

    println!("\n══════════════════════════════════════════════════════════════════");
    println!("                    LIVE TRADING RESULTS                        ");
    println!("══════════════════════════════════════════════════════════════════");
    println!("  Starting Balance:        ${:>14,.2} CAD", balance_cad);
    println!("  Final Equity:            ${:>14,.2} CAD", final_equity);
    println!("  Total Return:            ${:>14,.2} CAD  ({:>+7.2}%)", total_return, total_return_pct);
    println!("  Max Drawdown:            {:>14.2}%", max_dd * 100.0);
    println!("  ────────────────────────────────────────────────────────────────");
    println!("  Total Trades:            {:>14}", closed_trades.len());
    println!("  Win Rate:                {:>14.1}%", win_rate);
    println!("  Profit Factor:           {:>14.2}", profit_factor);
    println!("  ────────────────────────────────────────────────────────────────");
    println!("  Patterns: {:>6} | Fused: {:>6} | Decisions: {:>6} | Allowed: {:>6} | Rejected: {:>6}",
        total_patterns, total_fused, total_decisions, total_allowed, total_rejected);
    println!("══════════════════════════════════════════════════════════════════\n");

    // Write markdown report
    let mut md = String::new();
    md.push_str("# Live Trading Results\n\n");
    md.push_str("| # | Symbol | Class | Dir | Entry | Exit | Qty | P&L (CAD) | Fee | Reason | Date |\n");
    md.push_str("|---|--------|-------|-----|-------|------|-----|-----------|-----|--------|------|\n");
    for (i, t) in closed_trades.iter().enumerate() {
        let dir = if t.direction == 1 { "Long" } else { "Short" };
        md.push_str(&format!("| {} | {} | {} | {} | {:.2} | {:.2} | {:.2} | {:+.2} | {:.2} | {} |\n",
            i + 1, t.symbol, t.class, dir, t.entry, t.exit, t.quantity,
            t.pnl_cad, t.fee_cad, t.exit_time.format("%Y-%m-%d %H:%M")));
    }
    md.push_str(&format!("\n**Total Trades:** {} | **Final Balance:** ${:.2} CAD | **Net Return:** ${:+.2} CAD ({:+.2}%)\n",
        closed_trades.len(), final_equity, total_return, total_return_pct));

    let report_path = "LIVE_TRADING_RESULTS.md";
    std::fs::write(report_path, md)?;
    println!("Report saved to: {}", report_path);
    Ok(())
}
