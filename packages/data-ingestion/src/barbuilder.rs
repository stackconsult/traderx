use crate::writer::{OhlcvBar, Tick};
use std::collections::HashMap;

/// Builds OHLCV bars from raw ticks in real-time.
/// One instance per symbol+exchange. Stateless across bar boundaries.
#[derive(Default)]
pub struct BarBuilder {
    state: HashMap<String, BarState>,
}

struct BarState {
    bar_ts_nanos: i64,   // timestamp of bar open
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    vwap_num: f64,       // Σ(price × volume)
    trade_count: i64,
    bar_width_nanos: i64,
}

impl BarState {
    fn new(tick: &Tick, bar_ts: i64, width_nanos: i64) -> Self {
        Self {
            bar_ts_nanos: bar_ts,
            open: tick.price,
            high: tick.price,
            low: tick.price,
            close: tick.price,
            volume: tick.volume,
            vwap_num: tick.price * tick.volume,
            trade_count: 1,
            bar_width_nanos: width_nanos,
        }
    }

    fn update(&mut self, tick: &Tick) {
        if tick.price > self.high { self.high = tick.price; }
        if tick.price < self.low  { self.low  = tick.price; }
        self.close = tick.price;
        self.volume    += tick.volume;
        self.vwap_num  += tick.price * tick.volume;
        self.trade_count += 1;
    }

    fn to_bar(&self, exchange: &str, symbol: &str) -> OhlcvBar {
        let vwap = if self.volume > 0.0 { self.vwap_num / self.volume } else { self.close };
        OhlcvBar {
            ts_nanos: self.bar_ts_nanos,
            symbol:   symbol.to_owned(),
            exchange: exchange.to_owned(),
            open:     self.open,
            high:     self.high,
            low:      self.low,
            close:    self.close,
            volume:   self.volume,
            vwap,
            trade_count: self.trade_count,
        }
    }
}

impl BarBuilder {
    pub fn new() -> Self { Self::default() }

    /// Feed a tick. Returns a completed bar if the bar window closed.
    pub fn update_1m(&mut self, tick: &Tick) -> Option<OhlcvBar> {
        self.update_bar(tick, 60_000_000_000)
    }

    pub fn update_1h(&mut self, tick: &Tick) -> Option<OhlcvBar> {
        self.update_bar(tick, 3_600_000_000_000)
    }

    fn update_bar(&mut self, tick: &Tick, width_nanos: i64) -> Option<OhlcvBar> {
        let key = format!("{}:{}", tick.exchange, tick.symbol);
        let bar_ts = (tick.ts_nanos / width_nanos) * width_nanos;

        let entry = self.state.get_mut(&key);
        match entry {
            None => {
                self.state.insert(
                    key,
                    BarState::new(tick, bar_ts, width_nanos),
                );
                None
            }
            Some(state) if bar_ts > state.bar_ts_nanos => {
                // Bar rolled — emit completed bar, start fresh
                let completed = state.to_bar(&tick.exchange, &tick.symbol);
                *state = BarState::new(tick, bar_ts, width_nanos);
                Some(completed)
            }
            Some(state) => {
                state.update(tick);
                None
            }
        }
    }
}
