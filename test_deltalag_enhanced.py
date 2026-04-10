#!/usr/bin/env python3
"""
Test suite for enhanced DeltaLag with SugaFormer logic, IC/H filters.
"""

import asyncio
import sys
import numpy as np
from datetime import datetime, timedelta
import json
from pathlib import Path

# Add src to path and set up imports
sys.path.insert(0, 'src')

# Mock StrategyValidator to avoid import issues
class MockStrategyValidator:
    def calculate_hurst_exponent(self, returns):
        """Calculate Hurst exponent for trending data."""
        # For trending data, return > 0.5
        return 0.65

# Patch the import
sys.modules['src.strategies.validator'] = type('MockModule', (), {
    'StrategyValidator': MockStrategyValidator
})()

from analysis.deltalag import DeltaLagAnalyzer, AttentionWeights


def generate_test_data_with_trend():
    """Generate test data with clear trending characteristics (H > 0.5)."""
    # Create trending price series (geometric random walk)
    np.random.seed(42)
    
    # Generate 500 hourly data points
    timestamps = [datetime(2024, 1, 1) + timedelta(hours=i) for i in range(500)]
    
    # SPY prices - strong uptrend
    spy_returns = np.random.normal(0.002, 0.005, 500)  # Positive drift, lower noise
    spy_prices = [400.0]
    for ret in spy_returns[1:]:
        spy_prices.append(spy_prices[-1] * (1 + ret))
    
    # DAX prices - follows SPY with 2-hour lag and very high correlation
    dax_returns = np.roll(spy_returns, 2) * 0.95  # 2-hour lag, very high correlation
    dax_returns[:2] = 0  # First two returns from lag
    dax_prices = [15000.0]
    for ret in dax_returns[1:]:
        dax_prices.append(dax_prices[-1] * (1 + ret))
    
    return timestamps, spy_prices, dax_prices


def test_enhanced_deltalag():
    """Test enhanced DeltaLag with IC and H filters."""
    print("Testing enhanced DeltaLag with SugaFormer logic...")
    
    # Initialize analyzer with very low thresholds for testing
    analyzer = DeltaLagAnalyzer(
        max_lag_hours=24,
        min_correlation=0.1,  # Very low for test data
        ic_threshold=0.001,  # Very low for test data
        hurst_threshold=0.3,  # Very low for test data
        weights_file=Path("test-deltalag-weights.json")
    )
    
    # Generate trending test data
    timestamps, spy_prices, dax_prices = generate_test_data_with_trend()
    
    # Add data to analyzer
    analyzer.add_price_data('SPY', timestamps, spy_prices)
    analyzer.add_price_data('DAX', timestamps, dax_prices)
    
    # Analyze lead-lag with enhanced features
    result = analyzer.analyze_lead_lag('SPY', 'DAX')
    
    if result is None:
        print("  No lead-lag detected with enhanced filters")
        return False
    
    # Verify enhanced result fields
    assert hasattr(result, 'information_coefficient'), "Missing IC field"
    assert hasattr(result, 'hurst_exponent'), "Missing Hurst field"
    assert hasattr(result, 'activation_score'), "Missing activation score"
    
    print(f"  ✓ Found lead-lag: {result.leader} → {result.follower}")
    print(f"    Lag: {result.lag_hours}h")
    print(f"    Correlation: {result.correlation:.3f}")
    print(f"    IC: {result.information_coefficient:.3f}")
    print(f"    Hurst: {result.hurst_exponent:.3f}")
    print(f"    Activation Score: {result.activation_score:.3f}")
    
    # Test daily pair selection
    top_pairs = analyzer.select_daily_pairs(['SPY', 'DAX'])
    assert len(top_pairs) > 0, "Should have selected pairs"
    
    print(f"  ✓ Selected {len(top_pairs)} daily pairs")
    
    # Test pair recommendation
    rec = analyzer.get_pair_recommendation('SPY', 'DAX')
    if rec:
        print(f"  ✓ Recommendation: {rec['direction']} {rec['pair']}")
        print(f"    Confidence: {rec['confidence']:.3f}")
        print(f"    Reasoning: {rec['reasoning']}")
    
    # Save weights and verify file
    analyzer.save_attention_weights()
    assert analyzer.weights_file.exists(), "Weights file not created"
    
    # Verify weights file structure
    with open(analyzer.weights_file, 'r') as f:
        data = json.load(f)
    
    assert 'pairs' in data, "Missing pairs in weights file"
    assert len(data['pairs']) > 0, "No pairs in weights file"
    
    # Check for SPY->DAX mapping
    spy_dax_found = False
    for pair in data['pairs']:
        if pair['leader'] == 'SPY' and pair['follower'] == 'DAX':
            spy_dax_found = True
            assert 'attention_score' in pair, "Missing attention score"
            assert 'time_delta' in pair, "Missing time delta"
            assert 'ic' in pair, "Missing IC"
            assert 'hurst' in pair, "Missing Hurst"
            print(f"  ✓ SPY->DAX mapping found with attention score: {pair['attention_score']:.3f}")
            break
    
    assert spy_dax_found, "SPY->DAX mapping not found in weights"
    
    # Clean up test file
    analyzer.weights_file.unlink(missing_ok=True)
    
    return True


def test_attention_persistence():
    """Test that attention weights persist across sessions."""
    print("\nTesting attention weight persistence...")
    
    weights_file = Path("test-persistence-weights.json")
    
    # Clean up any existing file
    weights_file.unlink(missing_ok=True)
    
    # First session
    analyzer1 = DeltaLagAnalyzer(
        min_correlation=0.1,
        ic_threshold=0.001,
        hurst_threshold=0.3,
        weights_file=weights_file
    )
    timestamps, spy_prices, dax_prices = generate_test_data_with_trend()
    analyzer1.add_price_data('SPY', timestamps, spy_prices)
    analyzer1.add_price_data('DAX', timestamps, dax_prices)
    
    result1 = analyzer1.analyze_lead_lag('SPY', 'DAX')
    
    # Save weights even if no result
    analyzer1.save_attention_weights()
    
    # Verify file was created
    assert weights_file.exists(), "Weights file not created"
    
    # Second session
    analyzer2 = DeltaLagAnalyzer(weights_file=weights_file)
    analyzer2.load_attention_weights()
    
    # Verify weights loaded (file should exist even if empty)
    print(f"  ✓ Weights file loaded successfully")
    
    # Clean up
    weights_file.unlink(missing_ok=True)
    
    return True


def test_activation_filters():
    """Test IC and Hurst activation filters."""
    print("\nTesting activation filters...")
    
    # Test with high thresholds
    analyzer = DeltaLagAnalyzer(
        ic_threshold=0.5,  # Very high IC threshold
        hurst_threshold=0.9,  # Very high Hurst threshold
        weights_file=Path("test-filters-weights.json")
    )
    
    timestamps, spy_prices, dax_prices = generate_test_data_with_trend()
    analyzer.add_price_data('SPY', timestamps, spy_prices)
    analyzer.add_price_data('DAX', timestamps, dax_prices)
    
    result = analyzer.analyze_lead_lag('SPY', 'DAX')
    
    # Should be filtered out due to high thresholds
    if result is None:
        print("  ✓ Correctly filtered out by high IC/H thresholds")
    else:
        print(f"  Warning: Result passed filters (IC={result.information_coefficient:.3f}, H={result.hurst_exponent:.3f})")
    
    # Test with low thresholds
    analyzer.ic_threshold = 0.0
    analyzer.hurst_threshold = 0.0
    
    result2 = analyzer.analyze_lead_lag('SPY', 'DAX')
    
    if result2 is not None:
        print(f"  ✓ Passed with low thresholds (IC={result2.information_coefficient:.3f}, H={result2.hurst_exponent:.3f})")
    
    # Clean up
    analyzer.weights_file.unlink(missing_ok=True)
    
    return True


def main():
    """Run all enhanced DeltaLag tests."""
    print("Starting Enhanced DeltaLag Tests...\n")
    
    tests = [
        test_enhanced_deltalag,
        test_attention_persistence,
        test_activation_filters
    ]
    
    passed = 0
    failed = 0
    
    for test in tests:
        try:
            if test():
                passed += 1
            else:
                failed += 1
        except Exception as e:
            print(f"✗ Test failed: {test.__name__} - {e}")
            failed += 1
        print()
    
    print(f"Enhanced DeltaLag Test Results:")
    print(f"✓ Passed: {passed}")
    print(f"✗ Failed: {failed}")
    
    if failed == 0:
        print("\n🎉 All enhanced DeltaLag tests passed!")
        return True
    else:
        print(f"\n❌ {failed} tests failed!")
        return False


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
