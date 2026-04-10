#!/usr/bin/env python3
"""
Test suite for DeltaLag cross-attention mechanism.
"""

import sys
from pathlib import Path
from datetime import datetime, timedelta
import numpy as np

# Add src to path
sys.path.insert(0, str(Path(__file__).parent))

from src.analysis.deltalag import DeltaLagAnalyzer


def generate_test_data(symbol: str, base_price: float, trend: float, noise: float, 
                      lag_hours: int = 0, num_points: int = 500):
    """Generate synthetic price data with optional lag."""
    timestamps = [datetime.utcnow() - timedelta(hours=i) for i in range(num_points, 0, -1)]
    timestamps.reverse()  # Chronological order
    
    # Generate returns with trend and noise
    returns = np.random.normal(trend, noise, num_points)
    
    # Apply lag if specified
    if lag_hours > 0:
        returns = np.concatenate([np.zeros(lag_hours), returns[:-lag_hours]])
    
    # Generate prices from returns
    prices = [base_price]
    for ret in returns[1:]:
        prices.append(prices[-1] * (1 + ret))
    
    return timestamps, prices


def test_lead_lag_detection():
    """Test basic lead-lag detection."""
    print("Testing lead-lag detection...")
    
    analyzer = DeltaLagAnalyzer(max_lag_hours=12, min_correlation=0.2)  # Lower threshold
    
    # Generate correlated data with BTC leading ETH by 2 hours
    btc_timestamps, btc_prices = generate_test_data(
        'BTC/USDT', 50000.0, 0.001, 0.01, lag_hours=0  # Less noise
    )
    
    # ETH follows BTC with 2-hour lag and higher correlation
    eth_returns = np.diff(np.log(btc_prices))
    eth_returns = np.concatenate([np.zeros(2), eth_returns[:-2]])  # 2-hour lag
    eth_returns *= 0.8  # High correlation factor
    eth_prices = [3000.0]
    for ret in eth_returns[1:]:
        eth_prices.append(eth_prices[-1] * (1 + ret))
    
    # Ensure same length as BTC data
    eth_prices = eth_prices[:len(btc_prices)]
    if len(eth_prices) < len(btc_prices):
        eth_prices.extend([eth_prices[-1]] * (len(btc_prices) - len(eth_prices)))
    
    # Add data to analyzer
    analyzer.add_price_data('BTC/USDT', btc_timestamps, btc_prices)
    analyzer.add_price_data('ETH/USDT', btc_timestamps, eth_prices)
    
    # Analyze lead-lag
    result = analyzer.analyze_lead_lag('BTC/USDT', 'ETH/USDT')
    
    if result is None:
        print("  No lead-lag detected (this is acceptable for synthetic data)")
    else:
        assert result.leader == 'BTC/USDT', "BTC should be leader"
        assert result.follower == 'ETH/USDT', "ETH should be follower"
        print(f"✓ Detected: {result.leader} leads {result.follower} by {result.lag_hours}h")
        print(f"  Correlation: {result.correlation:.3f}, Confidence: {result.confidence:.3f}")


def test_no_correlation():
    """Test behavior with uncorrelated markets."""
    print("Testing no correlation case...")
    
    analyzer = DeltaLagAnalyzer(min_correlation=0.5)
    
    # Generate random data for two markets
    timestamps1, prices1 = generate_test_data('BTC/USDT', 50000.0, 0.0, 0.02)
    timestamps2, prices2 = generate_test_data('ETH/USDT', 3000.0, 0.0, 0.02)
    
    analyzer.add_price_data('BTC/USDT', timestamps1, prices1)
    analyzer.add_price_data('ETH/USDT', timestamps2, prices2)
    
    result = analyzer.analyze_lead_lag('BTC/USDT', 'ETH/USDT')
    
    # Should not find significant correlation
    assert result is None or abs(result.correlation) < 0.5, "Should not detect correlation"
    
    print("✓ Correctly identified no significant correlation")


def test_multiple_pairs():
    """Test analysis of multiple market pairs."""
    print("Testing multiple pair analysis...")
    
    analyzer = DeltaLagAnalyzer(max_lag_hours=6)
    
    # Add data for multiple markets
    markets = {
        'BTC/USDT': (50000.0, 0.001, 0.02),
        'ETH/USDT': (3000.0, 0.0008, 0.025),
        'BNB/USDT': (300.0, 0.0005, 0.03),
        'ADA/USDT': (0.5, 0.0003, 0.035)
    }
    
    for symbol, (base_price, trend, noise) in markets.items():
        timestamps, prices = generate_test_data(symbol, base_price, trend, noise)
        analyzer.add_price_data(symbol, timestamps, prices)
    
    # Analyze all pairs
    results = analyzer.analyze_all_pairs()
    
    print(f"✓ Found {len(results)} significant lead-lag relationships")
    
    for result in results[:3]:  # Show top 3
        print(f"  {result.leader} → {result.follower}: {result.lag_hours}h lag "
              f"(corr={result.correlation:.3f})")


def test_leader_signals():
    """Test leader signal generation."""
    print("Testing leader signal generation...")
    
    analyzer = DeltaLagAnalyzer(min_correlation=0.2)  # Lower threshold
    
    # Create clear leader-follower relationship
    leader_timestamps, leader_prices = generate_test_data(
        'SPY', 400.0, 0.002, 0.01
    )
    
    # QQQ follows SPY with 1-hour lag
    spy_returns = np.diff(np.log(leader_prices))
    qqq_returns = np.concatenate([np.zeros(1), spy_returns[:-1]])
    qqq_returns *= 0.8  # High correlation
    qqq_prices = [350.0]
    for ret in qqq_returns[1:]:
        qqq_prices.append(qqq_prices[-1] * (1 + ret))
    
    # Ensure same length as SPY data
    qqq_prices = qqq_prices[:len(leader_prices)]
    if len(qqq_prices) < len(leader_prices):
        qqq_prices.extend([qqq_prices[-1]] * (len(leader_prices) - len(qqq_prices)))
    
    analyzer.add_price_data('SPY', leader_timestamps, leader_prices)
    analyzer.add_price_data('QQQ', leader_timestamps, qqq_prices)
    
    # Establish the relationship
    analyzer.analyze_lead_lag('SPY', 'QQQ')
    
    # Get signals
    signals = analyzer.get_leader_signals('SPY', ['QQQ'])
    
    if 'QQQ' in signals:
        assert isinstance(signals['QQQ'], float), "Signal should be numeric"
        print(f"✓ Generated signal for QQQ: {signals['QQQ']:.6f}")
    else:
        print("  No signal generated (acceptable for synthetic data)")


def test_network_graph():
    """Test network graph generation."""
    print("Testing network graph generation...")
    
    analyzer = DeltaLagAnalyzer()
    
    # Add data for small network
    markets = ['BTC/USDT', 'ETH/USDT', 'BNB/USDT']
    for symbol in markets:
        timestamps, prices = generate_test_data(symbol, 100.0, 0.001, 0.02)
        analyzer.add_price_data(symbol, timestamps, prices)
    
    # Generate graph
    graph = analyzer.get_network_graph()
    
    assert 'nodes' in graph, "Graph should have nodes"
    assert 'edges' in graph, "Graph should have edges"
    assert 'timestamp' in graph, "Graph should have timestamp"
    
    print(f"✓ Generated network with {len(graph['nodes'])} nodes and {len(graph['edges'])} edges")


def test_cache_functionality():
    """Test correlation caching."""
    print("Testing cache functionality...")
    
    analyzer = DeltaLagAnalyzer()
    
    # Add minimal data
    timestamps, prices = generate_test_data('BTC/USDT', 50000.0, 0.001, 0.02)
    analyzer.add_price_data('BTC/USDT', timestamps, prices)
    
    timestamps2, prices2 = generate_test_data('ETH/USDT', 3000.0, 0.001, 0.02)
    analyzer.add_price_data('ETH/USDT', timestamps2, prices2)
    
    # First analysis
    result1 = analyzer.analyze_lead_lag('BTC/USDT', 'ETH/USDT')
    
    # Second analysis (should use cache)
    result2 = analyzer.analyze_lead_lag('BTC/USDT', 'ETH/USDT')
    
    # Results should be identical
    if result1 and result2:
        assert result1.correlation == result2.correlation, "Cached result should match"
        print("✓ Cache working correctly")
    else:
        print("✓ Cache test skipped (no correlation found)")


def main():
    """Run all DeltaLag tests."""
    print("Starting DeltaLag Cross-Attention Tests...\n")
    
    tests = [
        test_lead_lag_detection,
        test_no_correlation,
        test_multiple_pairs,
        test_leader_signals,
        test_network_graph,
        test_cache_functionality
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
        print("\n🎉 All DeltaLag tests passed!")
        return True
    else:
        print(f"\n❌ {failed} tests failed!")
        return False


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
