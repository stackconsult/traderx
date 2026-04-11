/// Performance metrics computed from equity curve.

#[derive(Debug, Clone, serde::Serialize)]
pub struct BacktestMetrics {
    pub total_return:      f64,
    pub annualized_return: f64,
    pub sharpe_ratio:      f64,
    pub sortino_ratio:     f64,
    pub max_drawdown:      f64,
    pub calmar_ratio:      f64,
    pub win_rate:          f64,
    pub profit_factor:     f64,
    pub avg_trade_pnl:     f64,
    pub num_trades:        usize,
    pub turnover_annual:   f64,
}

pub fn compute(navs: &[f64], trade_pnls: &[f64], bars_per_year: f64, risk_free: f64) -> BacktestMetrics {
    assert!(navs.len() >= 2);
    let n = navs.len();
    let initial = navs[0];
    let final_nav = navs[n - 1];
    let returns: Vec<f64> = navs.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();
    let mean_ret = mean(&returns);
    let std_ret  = std_dev(&returns, mean_ret);
    let years    = n as f64 / bars_per_year;
    let total_return      = (final_nav - initial) / initial;
    let annualized_return = (1.0 + total_return).powf(1.0 / years.max(1e-10)) - 1.0;
    let sharpe_ratio = if std_ret > 0.0 {
        (mean_ret - risk_free / bars_per_year) / std_ret * bars_per_year.sqrt()
    } else { 0.0 };
    let downside_rets: Vec<f64> = returns.iter().filter(|&&r| r < 0.0).copied().collect();
    let downside_std = std_dev(&downside_rets, 0.0);
    let sortino_ratio = if downside_std > 0.0 {
        (mean_ret - risk_free / bars_per_year) / downside_std * bars_per_year.sqrt()
    } else { 0.0 };
    let max_drawdown = {
        let mut peak = navs[0]; let mut mdd = 0.0f64;
        for &nav in navs {
            if nav > peak { peak = nav; }
            let dd = (nav - peak) / peak;
            if dd < mdd { mdd = dd; }
        }
        mdd
    };
    let calmar_ratio = if max_drawdown.abs() > 0.0 { annualized_return / max_drawdown.abs() } else { 0.0 };
    let num_trades = trade_pnls.len();
    let gross_profit: f64 = trade_pnls.iter().filter(|&&p| p > 0.0).sum();
    let gross_loss: f64   = trade_pnls.iter().filter(|&&p| p < 0.0).map(|&l| l.abs()).sum();
    let win_rate = if num_trades > 0 { trade_pnls.iter().filter(|&&p| p > 0.0).count() as f64 / num_trades as f64 } else { 0.0 };
    let profit_factor = if gross_loss > 0.0 { gross_profit / gross_loss } else { f64::INFINITY };
    let avg_trade_pnl = if num_trades > 0 { trade_pnls.iter().sum::<f64>() / num_trades as f64 } else { 0.0 };
    BacktestMetrics { total_return, annualized_return, sharpe_ratio, sortino_ratio, max_drawdown, calmar_ratio, win_rate, profit_factor, avg_trade_pnl, num_trades, turnover_annual: 0.0 }
}

fn mean(v: &[f64]) -> f64 { if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 } }

fn std_dev(v: &[f64], m: f64) -> f64 {
    if v.len() < 2 { return 0.0; }
    let var = v.iter().map(|&x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64;
    var.sqrt()
}
