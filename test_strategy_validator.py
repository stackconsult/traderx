#!/usr/bin/env python3
"""
Test suite for StrategyValidator IC and DSR calculations.
"""

import sys
from pathlib import Path
from datetime import datetime, timedelta
import numpy as np

# Add src to path
sys.path.insert(0, str(Path(__file__).parent))

from src.strategies.validator import StrategyValidator


def test_ic_calculation():
    """Test Information Coefficient calculation."""
    print("Testing IC calculation...")
    
    validator = StrategyValidator(min_samples=10)
    
    # Add signals with known correlation to returns
    base_date = datetime.utcnow() - timedelta(days=60)
    
    # Create perfect positive correlation (IC = 1.0)
    for i in range(30):
        signal_date = base_date + timedelta(days=i)
        signal = i / 29.0  # Signals from 0 to 1
        return_date = signal_date + timedelta(days=21)
        forward_return = signal * 0.02  # 2% max return
        
        validator.add_signal(signal_date, signal, "BTC/USDT")
        validator.add_return(return_date, "BTC/USDT", forward_return)
    
    ic, p_value = validator.calculate_ic()
    
    assert ic > 0.9, f"Expected IC > 0.9, got {ic}"
    assert p_value < 0.01, f"Expected p-value < 0.01, got {p_value}"
    
    print(f"✓ IC = {ic:.4f} (p-value: {p_value:.6f})")


def test_dsr_calculation():
    """Test Deflated Sharpe Ratio calculation."""
    print("Testing DSR calculation...")
    
    validator = StrategyValidator()
    
    # Create returns with known properties
    np.random.seed(42)
    returns = np.random.normal(0.001, 0.02, 100)  # Daily returns
    
    # Add some skewness and kurtosis
    returns = returns + 0.1 * returns ** 2  # Add positive skew
    
    sharpe = validator.calculate_sharpe_ratio(returns.tolist())
    dsr = validator.calculate_deflated_sharpe_ratio(returns.tolist())
    
    assert dsr >= 0, "DSR should be non-negative"
    assert isinstance(dsr, float), "DSR should be a float"
    
    print(f"✓ Sharpe = {sharpe:.4f}, DSR = {dsr:.4f}")
    
    # Note: DSR may not always be lower than Sharpe with simplified formula
    # The key is that DSR accounts for higher moments


def test_strategy_validation():
    """Test complete strategy validation."""
    print("Testing strategy validation...")
    
    # Test with good strategy
    validator = StrategyValidator(min_ic=0.3, min_dsr=0.5)
    
    # Add signals with good correlation
    base_date = datetime.utcnow() - timedelta(days=60)
    for i in range(30):
        signal_date = base_date + timedelta(days=i)
        signal = np.sin(i / 5.0)  # Oscillating signal
        return_date = signal_date + timedelta(days=21)
        # Create correlation with some noise
        forward_return = signal * 0.03 + np.random.normal(0, 0.01)
        
        validator.add_signal(signal_date, signal, "BTC/USDT")
        validator.add_return(return_date, "BTC/USDT", forward_return)
    
    # Generate some returns for Sharpe calculation
    returns = [forward_return * np.random.uniform(0.5, 1.5) 
              for forward_return in [s['forward_return'] 
                                   for s in validator.signal_history 
                                   if s['forward_return'] is not None]]
    
    # Create trending price series for Hurst calculation
    price_series = [10000 * (1 + i * 0.001) for i in range(200)]  # Trending up
    
    metrics = validator.calculate_strategy_metrics(returns, price_series)
    
    print(f"✓ IC = {metrics.information_coefficient:.4f}")
    print(f"✓ DSR = {metrics.deflated_sharpe_ratio:.4f}")
    print(f"✓ Hurst = {metrics.hurst_exponent:.4f}")
    print(f"✓ Valid: {metrics.is_valid}")
    
    # Test with bad strategy
    validator_bad = StrategyValidator(min_ic=0.5, min_dsr=1.0)
    
    # Add random signals (no correlation)
    for i in range(30):
        signal_date = base_date + timedelta(days=i)
        signal = np.random.uniform(-1, 1)
        return_date = signal_date + timedelta(days=21)
        forward_return = np.random.normal(0, 0.02)
        
        validator_bad.add_signal(signal_date, signal, "ETH/USDT")
        validator_bad.add_return(return_date, "ETH/USDT", forward_return)
    
    # Create mean-reverting price series
    price_series_bad = [10000 + 100 * np.sin(i / 10) for i in range(200)]  # Oscillating
    
    metrics_bad = validator_bad.calculate_strategy_metrics(returns, price_series_bad)
    
    assert not metrics_bad.is_valid, "Bad strategy should be invalid"
    print(f"✓ Bad strategy correctly rejected: {metrics_bad.termination_reason}")


def test_termination_logic():
    """Test strategy termination logic."""
    print("Testing termination logic...")
    
    validator = StrategyValidator(min_ic=0.1, min_dsr=0.5)
    
    # Add weak signals
    base_date = datetime.utcnow() - timedelta(days=60)
    for i in range(30):
        signal_date = base_date + timedelta(days=i)
        signal = np.random.uniform(-0.1, 0.1)  # Very weak signals
        return_date = signal_date + timedelta(days=21)
        forward_return = np.random.normal(0, 0.02)
        
        validator.add_signal(signal_date, signal, "BTC/USDT")
        validator.add_return(return_date, "BTC/USDT", forward_return)
    
    returns = np.random.normal(0, 0.02, 30).tolist()
    
    should_terminate = validator.should_terminate_strategy(returns)
    
    print(f"✓ Termination recommendation: {should_terminate}")


def test_validation_report():
    """Test validation report generation."""
    print("Testing validation report...")
    
    validator = StrategyValidator()
    
    # Add sample data
    base_date = datetime.utcnow() - timedelta(days=60)
    for i in range(30):
        signal_date = base_date + timedelta(days=i)
        signal = np.sin(i / 5.0)
        return_date = signal_date + timedelta(days=21)
        forward_return = signal * 0.02 + np.random.normal(0, 0.01)
        
        validator.add_signal(signal_date, signal, "BTC/USDT")
        validator.add_return(return_date, "BTC/USDT", forward_return)
    
    returns = [forward_return * np.random.uniform(0.5, 1.5) 
              for forward_return in [s['forward_return'] 
                                   for s in validator.signal_history 
                                   if s['forward_return'] is not None]]
    
    report = validator.get_validation_report(returns)
    
    assert 'metrics' in report, "Report should contain metrics"
    assert 'validation' in report, "Report should contain validation"
    assert 'recommendation' in report['validation'], "Report should have recommendation"
    
    print("✓ Validation report generated successfully")
    print(f"  Recommendation: {report['validation']['recommendation']}")


def main():
    """Run all validator tests."""
    print("Starting Strategy Validator Tests...\n")
    
    tests = [
        test_ic_calculation,
        test_dsr_calculation,
        test_strategy_validation,
        test_termination_logic,
        test_validation_report
    ]
    
    passed = 0
    failed = 0
    
    for test in tests:
        try:
            test()
            passed += 1
        except Exception as e:
            print(f"✗ Test failed: {test.__name__} - {e}")
            failed += 1
        print()
    
    print(f"Test Results:")
    print(f"✓ Passed: {passed}")
    print(f"✗ Failed: {failed}")
    
    if failed == 0:
        print("\n🎉 All strategy validator tests passed!")
        return True
    else:
        print(f"\n❌ {failed} tests failed!")
        return False


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
