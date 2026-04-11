---
trigger: manual
description: "Execute quant research and backtesting workflows"
---

# Quant Research Agent Rules

## Backtesting Engine Deployment

### Core Architecture
```rust
// Rust-based backtesting core with turboquant acceleration
pub struct BacktestEngine {
    pub vectorized_processor: VectorizedProcessor,
    pub turboquant_accelerator: TurboQuantAccelerator,
    pub memory_pool: MemoryPool,
    pub jit_compiler: JITCompiler,
}
```

### Self-Learning Features
- Automatic strategy pattern recognition
- Adaptive parameter tuning based on market regimes
- Neural architecture search for strategy optimization
- Meta-learning for rapid strategy adaptation

### Execution Protocol
1. Initialize vectorized data structures
2. Enable turboquant compression for historical data
3. Configure JIT compilation for strategy code
4. Set up distributed processing for large datasets

## Feature Engineering Automation

### Adaptive Feature Selection
```python
# Self-learning feature importance tracking
class AdaptiveFeatureSelector:
    def __init__(self):
        self.feature_performance = {}
        self.market_regime_detector = MarketRegimeDetector()
        self.neural_importance_estimator = NeuralImportanceEstimator()
    
    def update_importance(self, features, returns, regime):
        # Update feature importance based on current regime
        importance = self.neural_importance_estimator.predict(
            features, returns, regime
        )
        self.feature_performance[regime] = importance
```

### Feature Store Integration
- Real-time feature computation with streaming
- Batch feature generation for backtesting
- Automatic feature versioning and lineage
- Feature drift detection and alerting

## Strategy Development Framework

### Template-Based Strategy Generation
```yaml
strategy_template:
  name: "momentum_reversal"
  features:
    - returns_5d
    - volatility_20d
    - rsi_14
    - volume_ratio
  parameters:
    - momentum_threshold: [0.01, 0.05, 0.1]
    - reversal_threshold: [-0.05, -0.02, -0.01]
  optimization:
    method: "bayesian"
    objective: "sharpe_ratio"
    constraints:
      - max_drawdown < 0.2
      - win_rate > 0.4
```

### Auto-ML Integration
- Automated hyperparameter optimization
- Neural architecture search for deep strategies
- Ensemble strategy construction
- Performance attribution analysis

## Performance Optimization

### TurboQuant Acceleration
```bash
# Enable turboquant optimizations
export TURBOQUANT_JIT_ENABLE=1
export TURBOQUANT_VECTOR_WIDTH=512
export TURBOQUANT_MEMORY_POOL_SIZE="16GB"
export TURBOQUANT_PARALLEL_STRATEGIES=auto
```

### Memory Management
- Zero-copy operations between components
- Memory-mapped file access for large datasets
- Custom allocators for frequent allocations
- GPU memory pooling for matrix operations

## Self-Healing Mechanisms

### Error Recovery
- Automatic strategy rollback on performance degradation
- Failed job retry with exponential backoff
- Circuit breaker for failing data sources
- Graceful degradation under resource constraints

### Data Quality Assurance
- Real-time data validation
- Outlier detection and handling
- Missing data imputation strategies
- Cross-source data verification

## Validation and Testing

### Automated Testing Suite
```python
# Comprehensive backtesting validation
def validate_backtest_results(results):
    assert results.sharpe_ratio > 1.0, "Sharpe ratio too low"
    assert results.max_drawdown < 0.2, "Drawdown exceeds limit"
    assert results.turnover < 5.0, "Excessive trading"
    assert results.coverage > 0.95, "Insufficient market coverage"
```

### Performance Benchmarks
- Strategy backtesting: 10M bars/second
- Feature computation: 100K features/second
- Parameter optimization: 1000 evaluations/second
- Real-time inference: <1ms latency

## Research Collaboration

### Experiment Tracking
- Automatic experiment logging with MLflow
- Version control for strategies and features
- Collaborative research notebooks
- Reproducible research templates

### Knowledge Management
- Strategy performance database
- Feature effectiveness tracking
- Market regime classification
- Alpha decay monitoring
