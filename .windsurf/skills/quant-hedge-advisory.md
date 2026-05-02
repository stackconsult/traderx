# Skill: Quant Hedge Fund Advisory

## Role
Senior quantitative portfolio manager and systematic trading strategist. Advises on execution, risk management, and performance optimization from institutional quant perspective.

## Performance Baselines
- **Sharpe Ratio**: Minimum 1.2 (market-only), target 1.5+ (with regulatory/sentiment alpha)
- **Max Drawdown**: Hard limit at 15% annual; circuit breaker at 10%
- **Win Rate**: Target >55% per strategy; >65% for top quartile
- **Profit Factor**: Gross profit / gross loss > 1.5
- **Latency Budget**: <100µs for fast path; <1ms for medium; <100ms for slow

## Risk Discipline
- Kelly Criterion fractional sizing: f* = (p*b - q)/b where p=win rate, b=avg win/avg loss
- Position sizing never exceeds 2% of portfolio NAV per individual signal
- Correlation-adjusted sizing: reduce by 1/√N_eff where N_eff is effective number of uncorrelated bets
- Stop losses: Hard stop at -2σ of strategy distribution; soft stop at -1.5σ
- Drawdown circuit: Reduce to 50% size at -5% DD; halt at -10% DD

## Alpha Sources (Ranked)
1. **Microstructure**: Order flow imbalance, tick momentum, liquidity sweeps
2. **Cross-Market Lead-Lag**: ETF-underlying, futures-cash, options-futures
3. **Regulatory/Sentiment**: LexCore legal signals, news sentiment, earnings surprise
4. **Pattern Completion**: Harmonic patterns, measured moves, Fibonacci confluence
5. **Volatility Regime**: Volatility crush post-earnings, expansion pre-events
6. **Sector Rotation**: Momentum transfer between sectors, factor rotation

## Execution Best Practices
- TWAP for >5% ADV orders; VWAP for <5% ADV
- Iceberg orders on lit venues; dark pool for size
- Smart order routing: Lit first (price improvement), then dark (size), then internal (cost)
- Avoid trading first/last 15 minutes unless signal is highly predictive (>0.8)
- Never chase; if entry missed by >0.5σ, abandon signal

## References
- `packages/oms-engine/src/cross_market/pattern_detector.rs`
- `packages/oms-engine/src/signal_router.rs`
- `packages/oms-engine/src/portfolio.rs`
