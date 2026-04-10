#!/usr/bin/env python3
"""
Performance test suite for Hexagonal Liquidity Adapters.
Validates <10ms concurrent order submission across 5 venues.
"""

import asyncio
import sys
import time
from pathlib import Path
import json
from typing import List, Dict, Any

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from adapter_manager import AdapterManager
from adapters.mock_adapter import MockAdapter
from domain.models import UnifiedOrder, LiquidityRequest, OrderType


async def setup_mock_adapters(manager: AdapterManager, num_venues: int = 5) -> List[str]:
    """Setup mock adapters with varying latencies."""
    venues = []
    
    for i in range(num_venues):
        venue_name = f"MockVenue{i+1}"
        
        # Configure with different latencies (1-8ms)
        config = {
            "latency_ms": 1.0 + (i * 1.5),  # 1ms, 2.5ms, 4ms, 5.5ms, 7ms
            "success_rate": 0.95 + (i * 0.01),  # 95% to 99%
            "symbols": ["BTCUSDT", "ETHUSDT", "SOLUSDT"]
        }
        
        adapter = MockAdapter(venue_name, config)
        await manager.register_adapter(adapter)
        venues.append(venue_name)
        
    return venues


async def test_concurrent_order_submission():
    """Test concurrent order submission across 5 venues."""
    print("Testing concurrent order submission...")
    
    manager = AdapterManager()
    
    try:
        # Setup 5 mock venues
        venues = await setup_mock_adapters(manager, 5)
        print(f"✓ Registered {len(venues)} venues")
        
        # Get venue status
        status = await manager.get_venue_status()
        print("\nVenue Status:")
        for venue, info in status.items():
            print(f"  {venue}: connected={info['connected']}, latency={info['latency_ms']}ms")
        
        # Run performance test
        print("\nRunning performance test...")
        metrics = await manager.run_performance_test(
            num_orders=100,
            concurrent_venues=5
        )
        
        # Validate requirements
        print("\nPerformance Metrics:")
        print(f"  Total orders: {metrics['total_orders']}")
        print(f"  Successful orders: {metrics['successful_orders']}")
        print(f"  Success rate: {metrics['successful_orders']/metrics['total_orders']*100:.1f}%")
        print(f"  Average latency: {metrics['average_latency_ms']:.2f}ms")
        print(f"  Throughput: {metrics['orders_per_second']:.2f} orders/sec")
        
        # Check <10ms requirement
        if metrics['average_latency_ms'] < 10.0:
            print(f"✓ PASS: Average latency {metrics['average_latency_ms']:.2f}ms < 10ms")
        else:
            print(f"✗ FAIL: Average latency {metrics['average_latency_ms']:.2f}ms >= 10ms")
            return False
            
        # Check success rate
        success_rate = metrics['successful_orders'] / metrics['total_orders']
        if success_rate > 0.9:
            print(f"✓ PASS: Success rate {success_rate*100:.1f}% > 90%")
        else:
            print(f"✗ FAIL: Success rate {success_rate*100:.1f}% <= 90%")
            return False
            
        return True
        
    finally:
        await manager.shutdown()


async def test_best_price_routing():
    """Test best price routing strategy."""
    print("\nTesting best price routing...")
    
    manager = AdapterManager()
    
    try:
        # Setup 3 venues with different prices
        venues = []
        prices = [50000, 49950, 50050]  # Different bid/ask prices
        
        for i, price in enumerate(prices):
            venue_name = f"PriceVenue{i+1}"
            config = {
                "latency_ms": 2.0,
                "success_rate": 1.0,
                "symbols": ["BTCUSDT"]
            }
            
            adapter = MockAdapter(venue_name, config)
            await manager.register_adapter(adapter)
            
            # Set specific price
            adapter.order_book["BTCUSDT"]["ask"] = price
            venues.append(venue_name)
        
        # Submit buy order (should pick lowest ask)
        order = UnifiedOrder(
            tenant_id="test",
            symbol="BTCUSDT",
            side="BUY",
            quantity=0.01,
            order_type=OrderType.MARKET
        )
        
        request = LiquidityRequest(
            order=order,
            routing_strategy="BEST_PRICE"
        )
        
        response = await manager.execute_order(request)
        
        if response.status == "SUCCESS":
            print(f"✓ PASS: Best price routing successful")
            print(f"  Executed at: ${response.average_price:.2f}")
            print(f"  Expected: ${min(prices):.2f}")
            
            # Verify got best price
            if abs(response.average_price - min(prices)) < 1.0:
                print(f"✓ PASS: Got best price")
            else:
                print(f"✗ FAIL: Did not get best price")
                return False
        else:
            print(f"✗ FAIL: Best price routing failed: {response.error_message}")
            return False
            
        return True
        
    finally:
        await manager.shutdown()


async def test_split_routing():
    """Test split order routing."""
    print("\nTesting split order routing...")
    
    manager = AdapterManager()
    
    try:
        # Setup 3 venues
        venues = await setup_mock_adapters(manager, 3)
        
        # Submit large order for splitting
        order = UnifiedOrder(
            tenant_id="test",
            symbol="BTCUSDT",
            side="BUY",
            quantity=0.03,  # Will be split into 0.01 each
            order_type=OrderType.MARKET
        )
        
        request = LiquidityRequest(
            order=order,
            routing_strategy="SPLIT"
        )
        
        response = await manager.execute_order(request)
        
        if response.status in ["SUCCESS", "PARTIAL"]:
            print(f"✓ PASS: Split routing {response.status}")
            print(f"  Total filled: {response.total_filled}")
            print(f"  Number of executions: {len(response.executions)}")
            
            # Verify multiple executions
            if len(response.executions) > 1:
                print(f"✓ PASS: Order split across {len(response.executions)} venues")
            else:
                print(f"✗ FAIL: Order not split")
                return False
        else:
            print(f"✗ FAIL: Split routing failed: {response.error_message}")
            return False
            
        return True
        
    finally:
        await manager.shutdown()


async def test_latency_measurement():
    """Test accurate latency measurement."""
    print("\nTesting latency measurement...")
    
    manager = AdapterManager()
    
    try:
        # Setup venue with known latency
        config = {
            "latency_ms": 5.0,  # Exactly 5ms
            "success_rate": 1.0,
            "symbols": ["BTCUSDT"]
        }
        
        adapter = MockAdapter("LatencyTestVenue", config)
        await manager.register_adapter(adapter)
        
        # Submit single order
        order = UnifiedOrder(
            tenant_id="test",
            symbol="BTCUSDT",
            side="BUY",
            quantity=0.01,
            order_type=OrderType.MARKET
        )
        
        request = LiquidityRequest(
            order=order,
            routing_strategy="FASTEST"
        )
        
        # Measure actual time
        start_time = time.time()
        response = await manager.execute_order(request)
        actual_latency = (time.time() - start_time) * 1000
        
        print(f"  Configured latency: 5.0ms")
        print(f"  Measured latency: {actual_latency:.2f}ms")
        print(f"  Reported latency: {response.latency_ms:.2f}ms")
        
        # Allow some tolerance (+/-2ms)
        if 3.0 <= actual_latency <= 7.0:
            print(f"✓ PASS: Latency measurement accurate")
            return True
        else:
            print(f"✗ FAIL: Latency measurement inaccurate")
            return False
            
    finally:
        await manager.shutdown()


async def generate_performance_report():
    """Generate detailed performance report."""
    print("\nGenerating performance report...")
    
    manager = AdapterManager()
    
    try:
        # Test with different venue counts
        report = {
            "timestamp": time.time(),
            "tests": {}
        }
        
        for num_venues in [1, 3, 5, 10]:
            print(f"\nTesting with {num_venues} venues...")
            
            # Clean setup
            await manager.shutdown()
            manager = AdapterManager()
            
            # Setup venues
            await setup_mock_adapters(manager, num_venues)
            
            # Run test
            metrics = await manager.run_performance_test(
                num_orders=50,
                concurrent_venues=num_venues
            )
            
            report["tests"][f"{num_venues}_venues"] = metrics
            
            print(f"  Latency: {metrics['average_latency_ms']:.2f}ms")
            print(f"  Throughput: {metrics['orders_per_second']:.2f} ops/sec")
        
        # Save report
        report_path = Path(__file__).parent / "adapter-load-test.stats"
        with open(report_path, 'w') as f:
            json.dump(report, f, indent=2)
            
        print(f"\n✓ Report saved to {report_path}")
        
    finally:
        await manager.shutdown()


async def main():
    """Run all performance tests."""
    print("=" * 60)
    print("Hexagonal Liquidity Adapters Performance Tests")
    print("=" * 60)
    
    tests = [
        ("Concurrent Order Submission", test_concurrent_order_submission),
        ("Best Price Routing", test_best_price_routing),
        ("Split Order Routing", test_split_routing),
        ("Latency Measurement", test_latency_measurement)
    ]
    
    passed = 0
    failed = 0
    
    for test_name, test_func in tests:
        print(f"\n{test_name}:")
        print("-" * 40)
        
        try:
            if await test_func():
                passed += 1
                print(f"✅ {test_name} PASSED")
            else:
                failed += 1
                print(f"❌ {test_name} FAILED")
        except Exception as e:
            print(f"❌ {test_name} ERROR: {e}")
            failed += 1
    
    # Generate report
    await generate_performance_report()
    
    print("\n" + "=" * 60)
    print(f"Performance Test Results:")
    print(f"✅ Passed: {passed}")
    print(f"❌ Failed: {failed}")
    
    if failed == 0:
        print("\n🎉 ALL PERFORMANCE TESTS PASSED!")
        print("✅ <10ms concurrent order submission validated")
        return True
    else:
        print(f"\n❌ {failed} TESTS FAILED!")
        return False


if __name__ == "__main__":
    success = asyncio.run(main())
    sys.exit(0 if success else 1)
