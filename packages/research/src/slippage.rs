/// Market impact and slippage models for realistic backtesting.

#[derive(Debug, Clone)]
pub enum SlippageModel {
    /// Fixed basis points each way.
    Fixed { bps: f64 },
    /// Square-root market impact: impact = eta * sigma * sqrt(qty / adv)
    SquareRoot { eta: f64, sigma: f64 },
    /// Almgren-Chriss linear temporary impact.
    AlmgrenChriss { eta: f64, gamma: f64 },
}

impl SlippageModel {
    /// Returns the fill price including slippage for a buy order.
    /// `is_buy`: true = buy, false = sell.
    pub fn apply(&self, mid: f64, qty: f64, adv: f64, is_buy: bool) -> f64 {
        let impact = match self {
            SlippageModel::Fixed { bps } => mid * bps / 10_000.0,
            SlippageModel::SquareRoot { eta, sigma } => {
                let participation = (qty / adv).max(0.0);
                eta * sigma * mid * participation.sqrt()
            }
            SlippageModel::AlmgrenChriss { eta, gamma } => {
                let _ = gamma;
                eta * qty / adv * mid
            }
        };
        if is_buy { mid + impact } else { mid - impact }
    }
}

impl Default for SlippageModel {
    fn default() -> Self {
        SlippageModel::Fixed { bps: 1.0 }
    }
}

/// Commission model.
#[derive(Debug, Clone)]
pub struct CommissionModel {
    /// Per-trade fixed cost in USD.
    pub fixed: f64,
    /// Proportional rate (e.g. 0.0001 = 1 bps).
    pub rate: f64,
}

impl CommissionModel {
    pub fn calculate(&self, qty: f64, price: f64) -> f64 {
        self.fixed + self.rate * qty.abs() * price
    }
}

impl Default for CommissionModel {
    fn default() -> Self {
        Self { fixed: 0.0, rate: 0.0001 } // 1 bps
    }
}
