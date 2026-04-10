#!/usr/bin/env python3
"""
Test suite for HSTR (Historical State Reconstruction) implementation.
"""

import asyncio
import sys
from pathlib import Path
from datetime import datetime, timedelta
import json

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from python.hstr_client import HSTRClient, HSTRSQLiteClient, HSTRBenchmark


async def test_sqlite_fallback():
    """Test SQLite fallback implementation."""
    print("Testing SQLite fallback...")
    
    client = HSTRSQLiteClient(":memory:")
    await client.initialize()
    
    # Add test entity
    await client.conn.execute(
        "INSERT INTO entities (id, ticker, name) VALUES (?, ?, ?)",
        (1, "TEST", "Test Entity")
    )
    await client.conn.commit()
    
    # Create initial snapshot
    initial_state = {"revenue": 100.0, "eps": 1.0, "sector": "Technology"}
    await client.conn.execute(
        "INSERT INTO snapshots (entity_id, facet, timestamp, data) VALUES (?, ?, ?, ?)",
        (1, "financials", "2024-01-01T00:00:00", json.dumps(initial_state))
    )
    await client.conn.commit()
    
    # Add deltas
    deltas = [
        {"revenue": 105.0},  # Q1 update
        {"eps": 1.1},       # Q2 update
        {"revenue": 110.0}  # Q3 update
    ]
    
    for i, delta in enumerate(deltas):
        timestamp = f"2024-{(i+1)*3:02d}-01T00:00:00"
        await client.conn.execute(
            "INSERT INTO deltas (entity_id, facet, timestamp, patch) VALUES (?, ?, ?, ?)",
            (1, "financials", timestamp, json.dumps(delta))
        )
    await client.conn.commit()
    
    # Test reconstruction at different points
    test_cases = [
        ("2024-01-15", {"revenue": 100.0, "eps": 1.0, "sector": "Technology"}),
        ("2024-04-15", {"revenue": 105.0, "eps": 1.0, "sector": "Technology"}),
        ("2024-07-15", {"revenue": 105.0, "eps": 1.1, "sector": "Technology"}),
        ("2024-10-15", {"revenue": 110.0, "eps": 1.1, "sector": "Technology"}),
    ]
    
    for timestamp_str, expected in test_cases:
        timestamp = datetime.fromisoformat(timestamp_str)
        state = await client.reconstruct_state_at(1, "financials", timestamp)
        
        assert state is not None, f"State should be reconstructed for {timestamp_str}"
        assert state["revenue"] == expected["revenue"], f"Revenue mismatch for {timestamp_str}: got {state['revenue']}, expected {expected['revenue']}"
        assert state["eps"] == expected["eps"], f"EPS mismatch for {timestamp_str}"
        
    print("✓ SQLite fallback test passed")
    
    await client.conn.close()


async def test_hstr_client_interface():
    """Test HSTR client interface (without actual database)."""
    print("Testing HSTR client interface...")
    
    # Test that client can be instantiated
    client = HSTRClient("postgresql://test:test@localhost/test")
    
    # Test methods exist
    assert hasattr(client, 'initialize')
    assert hasattr(client, 'reconstruct_state_at')
    assert hasattr(client, 'add_snapshot')
    assert hasattr(client, 'add_delta')
    assert hasattr(client, 'search_similar_vectors')
    assert hasattr(client, 'get_performance_stats')
    
    print("✓ HSTR client interface test passed")


async def test_benchmark_structure():
    """Test benchmark structure."""
    print("Testing benchmark structure...")
    
    # Mock client for testing
    class MockClient:
        async def reconstruct_state_at(self, entity_id, facet, timestamp):
            # Simulate O(1) + O(k) complexity
            await asyncio.sleep(0.001)  # 1ms base time
            k = 5  # Simulate 5 deltas
            await asyncio.sleep(0.0001 * k)  # 0.1ms per delta
            return {"data": "test", "_deltas_applied": list(range(k))}
    
    client = MockClient()
    benchmark = HSTRBenchmark(client)
    
    # Run small benchmark
    results = await benchmark.run_benchmark(num_tests=10)
    
    # Verify results structure
    assert 'query_times' in results
    assert 'delta_counts' in results
    assert 'avg_query_time' in results
    assert 'p95_query_time' in results
    assert 'complexity_verified' in results
    
    # Verify complexity
    assert results['avg_deltas'] == 5.0, "Should have 5 deltas on average"
    assert results['complexity_verified'], "Complexity should be verified"
    
    print("✓ Benchmark structure test passed")


async def main():
    """Run all HSTR tests."""
    print("Starting HSTR Tests...\n")
    
    tests = [
        test_sqlite_fallback,
        test_hstr_client_interface,
        test_benchmark_structure
    ]
    
    passed = 0
    failed = 0
    
    for test in tests:
        try:
            await test()
            passed += 1
        except Exception as e:
            print(f"✗ Test failed: {test.__name__} - {e}")
            failed += 1
        print()
    
    print(f"HSTR Test Results:")
    print(f"✓ Passed: {passed}")
    print(f"✗ Failed: {failed}")
    
    if failed == 0:
        print("\n🎉 All HSTR tests passed!")
        return True
    else:
        print(f"\n❌ {failed} tests failed!")
        return False


if __name__ == "__main__":
    success = asyncio.run(main())
    sys.exit(0 if success else 1)
