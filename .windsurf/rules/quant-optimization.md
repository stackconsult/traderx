---
trigger: manual
description: "Execute portfolio optimization and risk management"
---

# Quant Optimization Agent Rules

## Portfolio Optimization Engine

### Core Solver Configuration
```python
# CVXPY optimization with turboquant acceleration
import cvxpy as cp
from turboquant import PortfolioOptimizer

class SelfLearningPortfolioOptimizer:
    def __init__(self):
        self.optimizer = PortfolioOptimizer(
            solver="OSQP",  # Operator splitting
            acceleration="turboquant",
            parallel=True
        )
        self.regime_detector = MarketRegimeDetector()
        self.constraint_learner = ConstraintLearner()
    
    def optimize(self, returns, views=None):
        regime = self.regime_detector.detect(returns)
        constraints = self.constraint_learner.get_constraints(regime)
        
        # Adaptive optimization based on regime
        if regime == "crisis":
            return self.risk_parity_optimization(returns, constraints)
        elif regime == "bull":
            return self.momentum_optimization(returns, constraints)
        else:
            return self.mean_variance_optimization(returns, views, constraints)
```

### Self-Healing Mechanisms
- Constraint violation auto-correction
- Solver fallback on convergence failure
- Automatic regularization for ill-conditioned problems
- Real-time constraint adjustment based on market conditions

## Risk Management System

### Real-Time Risk Monitoring
```rust
// Rust-based risk engine with sub-microsecond latency
pub struct RiskEngine {
    pub portfolio_tracker: PortfolioTracker,
    pub var_calculator: VarCalculator,
    pub stress_tester: StressTester,
    pub circuit_breaker: CircuitBreaker,
}

impl RiskEngine {
    pub fn evaluate_risk(&mut self, positions: &[Position]) -> RiskMetrics {
        // Turboquant-accelerated risk calculations
        let portfolio_var = self.var_calculator.calculate(
            positions, 
            Method::MonteCarlo { simulations: 100_000 }
        );
        
        // Auto-adjust position limits
        if portfolio_var > self.risk_budget {
            self.circuit_breaker.reduce_exposure();
        }
        
        RiskMetrics::new(portfolio_var, self.stress_tester.run_scenarios())
    }
}
```

### Adaptive Risk Limits
- Dynamic VaR limits based on volatility
- Regime-specific stress scenarios
- Automatic position sizing adjustments
- Real-time correlation monitoring

## Factor Model Integration

### Statistical Factor Extraction
```python
# Turboquant-accelerated factor model
class TurboQuantFactorModel:
    def __init__(self, n_factors=20):
        self.n_factors = n_factors
        self.turboquant_engine = TurboQuantEngine()
        self.factor_loadings = None
        self.factor_returns = None
    
    def fit(self, returns):
        # Use randomized SVD for large datasets
        U, S, Vt = randomized_svd(
            returns, 
            n_components=self.n_factors,
            engine="turboquant"
        )
        
        self.factor_loadings = U * np.sqrt(S)
        self.factor_returns = Vt.T * np.sqrt(S)
        
        # Store compressed representation
        self.turboquant_engine.compress_factors(
            self.factor_loadings, 
            compression_ratio=0.1
        )
```

### Factor Attribution System
- Real-time factor exposure calculation
- Factor contribution to risk and return
- Factor timing strategies
- Crowding detection and mitigation

## Optimization Strategies

### Risk Parity Implementation
```yaml
risk_parity_config:
  solver: "ADMM"
  convergence_tolerance: 1e-6
  max_iterations: 1000
  turboquant_features:
    - parallel_factor_updates
    - memory_mapped_data
    - gpu_acceleration
  constraints:
    - no_short_selling
    - sector_neutrality
    - turnover_limit: 0.2
```

### Black-Litterman Integration
- Automatic view generation from ML models
- Confidence scoring for views
- Regime-adjusted uncertainty parameters
- Robust optimization with distributional ambiguity

## Performance Optimization

### TurboQuant Acceleration
```bash
# Enable optimization accelerations
export TURBOQUANT_OPTIMIZATION_MODE=aggressive
export TURBOQUANT_PARALLEL_FACTORS=auto
export TURBOQUANT_GPU_SOLVER=1
export TURBOQUANT_MEMORY_MAPPING=1
```

### Memory Efficiency
- Zero-copy matrix operations
- Sparse matrix representations
- Out-of-core computation for large universes
- Custom memory allocators

## Self-Learning Features

### Strategy Adaptation
```python
class AdaptiveOptimizationStrategy:
    def __init__(self):
        self.performance_tracker = PerformanceTracker()
        self.meta_learner = MetaLearner()
        self.strategy_selector = StrategySelector()
    
    def select_strategy(self, market_conditions):
        # Meta-learning for strategy selection
        strategy_performance = self.performance_tracker.get_recent_performance()
        optimal_strategy = self.meta_learner.predict_best_strategy(
            market_conditions, 
            strategy_performance
        )
        return self.strategy_selector.get_strategy(optimal_strategy)
```

### Constraint Learning
- Automatic constraint discovery from data
- Dynamic constraint adjustment
- Constraint satisfaction prediction
- User preference learning

## Validation and Testing

### Optimization Validation
```python
def validate_optimization_result(weights, expected_return, expected_risk):
    actual_return = calculate_portfolio_return(weights)
    actual_risk = calculate_portfolio_risk(weights)
    
    assert abs(actual_return - expected_return) < 1e-6, "Return mismatch"
    assert abs(actual_risk - expected_risk) < 1e-6, "Risk mismatch"
    assert np.all(weights >= -1e-8), "Negative weights detected"
    assert abs(weights.sum() - 1.0) < 1e-6, "Weights not summing to 1"
```

### Stress Testing
- Monte Carlo scenario generation
- Historical crisis replay
- Correlation breakdown scenarios
- Liquidity stress testing

## Execution Integration

### Order Generation
```python
# Convert optimization weights to executable orders
def generate_orders(current_positions, target_weights, market_impact_model):
    trades = target_weights - current_positions
    
    # Optimize execution with market impact
    optimized_trades = market_impact_model.optimize_execution(trades)
    
    return [
        Order(
            symbol=symbol,
            quantity=quantity,
            order_type="adaptive",
            execution_algorithm="twap_with_impact"
        )
        for symbol, quantity in optimized_trades.items()
    ]
```

### Performance Monitoring
- Real-time tracking vs. benchmark
- Attribution analysis
- Turnover monitoring
- Cost analysis
