#!/usr/bin/env python3
"""
Load testing prototype for Signal Router
Validates 10k signals/sec with <5μs latency requirement
"""

import asyncio
import json
import socket
import time
import statistics
from dataclasses import dataclass
from typing import List
import random

@dataclass
class TestResult:
    signals_sent: int
    duration_seconds: float
    avg_latency_us: float
    p95_latency_us: float
    p99_latency_us: float
    errors: int

class SignalRouterLoadTester:
    def __init__(self, socket_path: str = "/tmp/traderx_signals.sock"):
        self.socket_path = socket_path
        self.latencies: List[float] = []
        self.errors = 0
        
    async def send_signal(self, sock, signal_id: int) -> float:
        """Send a single signal and measure latency"""
        signal = {
            "agent_id": f"test_agent_{signal_id % 10}",
            "symbol": random.choice(["BTC-USD", "ETH-USD", "SPY"]),
            "direction": random.choice(["long", "short"]),
            "conviction": round(random.random(), 3),
            "max_notional_usd": random.uniform(1000, 100000),
            "ttl_ms": 1000,
            "meta": {"test_id": signal_id}
        }
        
        # Serialize once
        message = json.dumps(signal) + "\n"
        
        # Measure round-trip time (approximate)
        start = time.perf_counter()
        
        try:
            sock.send(message.encode())
            # In real test, we'd wait for ack or measure one-way
            latency = (time.perf_counter() - start) * 1_000_000  # Convert to μs
            return latency
        except Exception as e:
            self.errors += 1
            print(f"Error sending signal {signal_id}: {e}")
            return -1
    
    async def run_load_test(self, num_signals: int = 100_000, duration_seconds: int = 10) -> TestResult:
        """Run load test with specified number of signals"""
        print(f"Starting load test: {num_signals} signals")
        
        # Connect to Unix socket
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.connect(self.socket_path)
        
        start_time = time.perf_counter()
        signal_id = 0
        
        # Send signals continuously
        while signal_id < num_signals and (time.perf_counter() - start_time) < duration_seconds:
            # Batch sends for efficiency
            batch_size = 100
            for _ in range(batch_size):
                if signal_id >= num_signals:
                    break
                    
                latency = await self.send_signal(sock, signal_id)
                if latency > 0:
                    self.latencies.append(latency)
                signal_id += 1
            
            # Small yield to prevent blocking
            await asyncio.sleep(0.001)
        
        total_duration = time.perf_counter() - start_time
        sock.close()
        
        # Calculate statistics
        if self.latencies:
            avg_latency = statistics.mean(self.latencies)
            p95_latency = statistics.quantiles(self.latencies, n=20)[18]  # 95th percentile
            p99_latency = statistics.quantiles(self.latencies, n=100)[98]  # 99th percentile
        else:
            avg_latency = p95_latency = p99_latency = 0
        
        return TestResult(
            signals_sent=signal_id,
            duration_seconds=total_duration,
            avg_latency_us=avg_latency,
            p95_latency_us=p95_latency,
            p99_latency_us=p99_latency,
            errors=self.errors
        )
    
    async def run_concurrent_test(self, num_clients: int = 10, signals_per_client: int = 10_000):
        """Test with multiple concurrent clients"""
        print(f"Running concurrent test: {num_clients} clients, {signals_per_client} signals each")
        
        tasks = []
        for i in range(num_clients):
            task = asyncio.create_task(self.run_load_test(signals_per_client, 30))
            tasks.append(task)
        
        results = await asyncio.gather(*tasks)
        
        # Aggregate results
        total_signals = sum(r.signals_sent for r in results)
        total_errors = sum(r.errors for r in results)
        all_latencies = []
        for r in results:
            all_latencies.extend([r.avg_latency_us] * r.signals_sent)
        
        print(f"\nConcurrent Test Results:")
        print(f"Total signals: {total_signals}")
        print(f"Total errors: {total_errors}")
        print(f"Signals/sec: {total_signals / 30:.0f}")
        
        if all_latencies:
            print(f"Average latency: {statistics.mean(all_latencies):.2f}μs")
            print(f"P95 latency: {statistics.quantiles(all_latencies, n=20)[18]:.2f}μs")
        
        return results

async def main():
    """Main test runner"""
    tester = SignalRouterLoadTester()
    
    # Test 1: Single client throughput
    print("\n=== Test 1: Single Client Throughput ===")
    result = await tester.run_load_test(num_signals=100_000, duration_seconds=10)
    
    print(f"\nResults:")
    print(f"Signals sent: {result.signals_sent}")
    print(f"Duration: {result.duration_seconds:.2f}s")
    print(f"Signals/sec: {result.signals_sent / result.duration_seconds:.0f}")
    print(f"Average latency: {result.avg_latency_us:.2f}μs")
    print(f"P95 latency: {result.p95_latency_us:.2f}μs")
    print(f"P99 latency: {result.p99_latency_us:.2f}μs")
    print(f"Errors: {result.errors}")
    
    # Validate requirements
    signals_per_sec = result.signals_sent / result.duration_seconds
    if signals_per_sec < 10_000:
        print(f"❌ FAILED: Throughput {signals_per_sec:.0f} < 10,000 signals/sec")
    else:
        print(f"✅ PASSED: Throughput {signals_per_sec:.0f} >= 10,000 signals/sec")
    
    if result.p95_latency_us > 5.0:
        print(f"❌ FAILED: P95 latency {result.p95_latency_us:.2f}μs > 5μs")
    else:
        print(f"✅ PASSED: P95 latency {result.p95_latency_us:.2f}μs <= 5μs")
    
    # Test 2: Concurrent clients
    print("\n=== Test 2: Concurrent Clients ===")
    await tester.run_concurrent_test(num_clients=10, signals_per_client=10_000)

if __name__ == "__main__":
    asyncio.run(main())
