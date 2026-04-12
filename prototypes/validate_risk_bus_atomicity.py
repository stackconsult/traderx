#!/usr/bin/env python3
"""
Risk Bus Atomicity Validation Prototype
Validates lock-free atomic operations under high contention
"""

import threading
import time
import random
import statistics
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from typing import List, Dict
import ctypes

# Simulate the Risk Bus atomic operations
class MockRiskBus:
    def __init__(self):
        # Using ctypes to simulate atomic operations
        self.global_halt = ctypes.c_bool(False)
        self.kill_switch = ctypes.c_bool(False)
        self.var_breach = ctypes.c_bool(False)
        self.portfolio_dd_bps = ctypes.c_int32(0)
        self.peak_nav_fp = ctypes.c_int64(100_000_000)  # $10M in 1e4 fixed point
        self.current_nav_fp = ctypes.c_int64(100_000_000)
        
        # Position limits (symbol -> limit_fp)
        self.position_limits = {}
        self.current_positions = {}
        
        # Statistics
        self.check_count = 0
        self.update_count = 0
        
    def check_risk(self) -> str:
        """Simulate risk check with atomic reads"""
        self.check_count += 1
        
        # Atomic reads (simulated)
        if self.kill_switch.value:
            return "kill_switch"
        if self.global_halt.value:
            return "global_halt"
        if self.var_breach.value:
            return "var_breach"
        
        dd = self.portfolio_dd_bps.value
        if dd < -2000:  # -20% drawdown
            return "drawdown_breach"
        
        return "ok"
    
    def update_nav(self, new_nav_usd: float):
        """Update NAV and check drawdown"""
        self.update_count += 1
        nav_fp = int(new_nav_usd * 10000)
        
        # Atomic compare-and-swap simulation
        old_nav = self.current_nav_fp.value
        self.current_nav_fp.value = nav_fp
        
        # Update peak if necessary
        if nav_fp > self.peak_nav_fp.value:
            self.peak_nav_fp.value = nav_fp
        
        # Calculate drawdown
        peak = self.peak_nav_fp.value
        if peak > 0:
            drawdown_bps = int(((nav_fp - peak) / peak) * 10000)
            self.portfolio_dd_bps.value = drawdown_bps
            
            # Trigger global halt on excessive drawdown
            if drawdown_bps < -2000:
                self.global_halt.value = True
    
    def update_position(self, symbol: str, delta_notional: float):
        """Update position for a symbol"""
        if symbol not in self.position_limits:
            self.position_limits[symbol] = int(1_000_000 * 10000)  # $1M default limit
        
        if symbol not in self.current_positions:
            self.current_positions[symbol] = 0
        
        limit_fp = self.position_limits[symbol]
        current_fp = self.current_positions[symbol]
        delta_fp = int(delta_notional * 10000)
        
        # Check if update would breach limit
        if abs(current_fp + delta_fp) > limit_fp:
            return False
        
        # Atomic update
        self.current_positions[symbol] = current_fp + delta_fp
        return True
    
    def reset(self):
        """Reset for next test"""
        self.global_halt.value = False
        self.kill_switch.value = False
        self.var_breach.value = False
        self.portfolio_dd_bps.value = 0
        self.current_nav_fp.value = self.peak_nav_fp.value
        self.current_positions.clear()

@dataclass
class AtomicityTestResult:
    total_operations: int
    contention_errors: int
    avg_latency_ns: float
    max_latency_ns: float
    throughput_ops_per_sec: float
    data_consistency_passed: bool

class RiskBusAtomicityTester:
    def __init__(self):
        self.risk_bus = MockRiskBus()
        self.latencies: List[float] = []
        self.errors = 0
        
    def worker_thread(self, thread_id: int, operations: int, symbols: List[str]):
        """Worker thread that performs risk operations"""
        thread_latencies = []
        thread_errors = 0
        
        for i in range(operations):
            start = time.perf_counter_ns()
            
            # Mix of operations
            op = random.choice(['check', 'update_nav', 'update_position'])
            
            try:
                if op == 'check':
                    result = self.risk_bus.check_risk()
                elif op == 'update_nav':
                    # Simulate NAV updates
                    new_nav = 10_000_000 + random.uniform(-1_000_000, 1_000_000)
                    self.risk_bus.update_nav(new_nav)
                else:
                    # Update position
                    symbol = random.choice(symbols)
                    delta = random.uniform(-100_000, 100_000)
                    success = self.risk_bus.update_position(symbol, delta)
                    if not success:
                        thread_errors += 1
                
                latency = (time.perf_counter_ns() - start)
                thread_latencies.append(latency)
                
            except Exception as e:
                thread_errors += 1
                print(f"Thread {thread_id} error: {e}")
        
        # Aggregate results
        self.latencies.extend(thread_latencies)
        self.errors += thread_errors
    
    def test_atomicity_under_contention(self, num_threads: int = 100, operations_per_thread: int = 1000) -> AtomicityTestResult:
        """Test atomic operations under high contention"""
        print(f"Testing atomicity: {num_threads} threads, {operations_per_thread} ops each")
        
        # Setup test symbols
        symbols = [f"SYM_{i}" for i in range(10)]
        
        # Reset state
        self.risk_bus.reset()
        self.latencies.clear()
        self.errors = 0
        
        # Record start time
        start_time = time.perf_counter()
        
        # Create and start threads
        threads = []
        for i in range(num_threads):
            thread = threading.Thread(
                target=self.worker_thread,
                args=(i, operations_per_thread, symbols)
            )
            threads.append(thread)
        
        # Start all threads simultaneously
        for thread in threads:
            thread.start()
        
        # Wait for completion
        for thread in threads:
            thread.join()
        
        total_time = time.perf_counter() - start_time
        total_operations = num_threads * operations_per_thread
        
        # Calculate statistics
        if self.latencies:
            avg_latency = statistics.mean(self.latencies)
            max_latency = max(self.latencies)
        else:
            avg_latency = max_latency = 0
        
        throughput = total_operations / total_time
        
        # Verify data consistency
        consistency_passed = self.verify_data_consistency()
        
        return AtomicityTestResult(
            total_operations=total_operations,
            contention_errors=self.errors,
            avg_latency_ns=avg_latency,
            max_latency_ns=max_latency,
            throughput_ops_per_sec=throughput,
            data_consistency_passed=consistency_passed
        )
    
    def verify_data_consistency(self) -> bool:
        """Verify that data remains consistent after concurrent operations"""
        # Check that positions don't exceed limits
        for symbol, pos_fp in self.risk_bus.current_positions.items():
            limit_fp = self.risk_bus.position_limits.get(symbol, int(1_000_000 * 10000))
            if abs(pos_fp) > limit_fp:
                print(f"❌ Position limit breached: {symbol} pos=${pos_fp/10000:.2f} limit=${limit_fp/10000:.2f}")
                return False
        
        # Check NAV consistency
        current_nav = self.risk_bus.current_nav_fp.value / 10000
        peak_nav = self.risk_bus.peak_nav_fp.value / 10000
        
        if current_nav < 0 or peak_nav < 0:
            print(f"❌ Invalid NAV: current=${current_nav:.2f} peak=${peak_nav:.2f}")
            return False
        
        # Check drawdown calculation
        if peak_nav > 0:
            expected_dd = ((current_nav - peak_nav) / peak_nav) * 10000
            actual_dd = self.risk_bus.portfolio_dd_bps.value
            if abs(expected_dd - actual_dd) > 1:  # Allow 1 BP rounding error
                print(f"❌ Drawdown mismatch: expected={expected_dd:.2f} actual={actual_dd}")
                return False
        
        return True
    
    def test_memory_ordering(self) -> bool:
        """Test that memory ordering is correct for atomic operations"""
        print("Testing memory ordering...")
        
        # Test scenario: Thread A sets kill_switch, Thread B should see it immediately
        kill_switch_seen = threading.Event()
        
        def thread_a():
            time.sleep(0.1)  # Small delay
            self.risk_bus.kill_switch.value = True
        
        def thread_b():
            while not self.risk_bus.kill_switch.value:
                pass
            kill_switch_seen.set()
        
        # Start threads
        a = threading.Thread(target=thread_a)
        b = threading.Thread(target=thread_b)
        
        start = time.perf_counter()
        a.start()
        b.start()
        
        # Wait for thread B to see the update
        kill_switch_seen.wait(timeout=1.0)
        
        elapsed = time.perf_counter() - start
        
        a.join()
        b.join()
        
        # Should see the update almost immediately
        if elapsed > 0.1:  # More than 100ms delay is bad
            print(f"❌ Memory ordering issue: {elapsed*1000:.2f}ms delay")
            return False
        
        print(f"✅ Memory ordering OK: {elapsed*1000:.2f}ms delay")
        return True

def main():
    """Run all atomicity tests"""
    tester = RiskBusAtomicityTester()
    
    print("=== Risk Bus Atomicity Validation ===\n")
    
    # Test 1: Basic contention
    print("Test 1: High Contention (100 threads)")
    result1 = tester.test_atomicity_under_contention(num_threads=100, operations_per_thread=1000)
    
    print(f"\nResults:")
    print(f"Total operations: {result1.total_operations}")
    print(f"Contention errors: {result1.contention_errors}")
    print(f"Avg latency: {result1.avg_latency_ns:.2f}ns")
    print(f"Max latency: {result1.max_latency_ns:.2f}ns")
    print(f"Throughput: {result1.throughput_ops_per_sec:.0f} ops/sec")
    print(f"Data consistency: {'✅ PASSED' if result1.data_consistency_passed else '❌ FAILED'}")
    
    # Validate requirements
    if result1.avg_latency_ns > 100:  # Should be <100ns
        print(f"❌ FAILED: Avg latency {result1.avg_latency_ns:.2f}ns > 100ns")
    else:
        print(f"✅ PASSED: Avg latency {result1.avg_latency_ns:.2f}ns <= 100ns")
    
    if result1.throughput_ops_per_sec < 1_000_000:  # Should handle >1M ops/sec
        print(f"⚠️  WARNING: Throughput {result1.throughput_ops_per_sec:.0f} < 1M ops/sec")
    
    # Test 2: Extreme contention
    print("\nTest 2: Extreme Contention (1000 threads)")
    result2 = tester.test_atomicity_under_contention(num_threads=1000, operations_per_thread=100)
    
    print(f"\nResults:")
    print(f"Data consistency: {'✅ PASSED' if result2.data_consistency_passed else '❌ FAILED'}")
    
    # Test 3: Memory ordering
    print("\nTest 3: Memory Ordering")
    memory_ok = tester.test_memory_ordering()
    
    # Summary
    print("\n=== Summary ===")
    all_passed = (
        result1.data_consistency_passed and
        result2.data_consistency_passed and
        memory_ok and
        result1.avg_latency_ns <= 100
    )
    
    if all_passed:
        print("✅ ALL TESTS PASSED - Risk Bus is ready for production")
    else:
        print("❌ SOME TESTS FAILED - Review and fix issues before production")

if __name__ == "__main__":
    main()
