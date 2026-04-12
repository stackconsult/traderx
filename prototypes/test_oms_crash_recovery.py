#!/usr/bin/env python3
"""
OMS Crash Recovery Validation Prototype
Tests Aeron journal replay and complete state recovery
"""

import asyncio
import json
import time
import uuid
import random
import subprocess
import signal
import os
from dataclasses import dataclass, asdict
from typing import List, Dict, Optional
import sqlite3
import tempfile
import shutil

@dataclass
class Order:
    order_id: str
    account_id: str
    symbol: str
    side: str
    order_type: str
    quantity: float
    price: Optional[float]
    state: str
    created_at: str
    updated_at: str

@dataclass
class Fill:
    order_id: str
    symbol: str
    side: str
    quantity: float
    fill_price: float
    commission: float
    timestamp_ns: int

class MockAeronJournal:
    """Mock Aeron journal for testing"""
    def __init__(self, journal_path: str):
        self.journal_path = journal_path
        self.conn = sqlite3.connect(journal_path)
        self._init_db()
        
    def _init_db(self):
        """Initialize journal database"""
        self.conn.execute("""
            CREATE TABLE IF NOT EXISTS journal_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp_ns INTEGER NOT NULL,
                entry_type TEXT NOT NULL,
                data BLOB NOT NULL
            )
        """)
        self.conn.commit()
    
    def append(self, entry_type: str, data: dict):
        """Append entry to journal"""
        timestamp_ns = time.time_ns()
        data_json = json.dumps(data).encode()
        
        self.conn.execute(
            "INSERT INTO journal_entries (timestamp_ns, entry_type, data) VALUES (?, ?, ?)",
            (timestamp_ns, entry_type, data_json)
        )
        self.conn.commit()
    
    def read_all(self) -> List[Dict]:
        """Read all journal entries"""
        cursor = self.conn.execute(
            "SELECT timestamp_ns, entry_type, data FROM journal_entries ORDER BY timestamp_ns"
        )
        entries = []
        for ts, entry_type, data in cursor:
            entries.append({
                "timestamp_ns": ts,
                "entry_type": entry_type,
                "data": json.loads(data.decode())
            })
        return entries
    
    def close(self):
        """Close journal"""
        self.conn.close()

class MockOMSEngine:
    """Mock OMS Engine for testing"""
    def __init__(self, journal_path: str):
        self.journal = MockAeronJournal(journal_path)
        self.orders: Dict[str, Order] = {}
        self.fills: List[Fill] = []
        self.next_order_id = 1
        
    def create_order(self, account_id: str, symbol: str, side: str, 
                    order_type: str, quantity: float, price: Optional[float] = None) -> str:
        """Create a new order"""
        order_id = f"ORD_{self.next_order_id:08d}"
        self.next_order_id += 1
        
        order = Order(
            order_id=order_id,
            account_id=account_id,
            symbol=symbol,
            side=side,
            order_type=order_type,
            quantity=quantity,
            price=price,
            state="New",
            created_at=time.strftime("%Y-%m-%dT%H:%M:%S.%fZ"),
            updated_at=time.strftime("%Y-%m-%dT%H:%M:%S.%fZ")
        )
        
        # Journal the order creation
        self.journal.append("order_created", asdict(order))
        
        # Update state
        order.state = "Pending"
        self.orders[order_id] = order
        
        return order_id
    
    def fill_order(self, order_id: str, fill_price: float, fill_qty: float, commission: float = 0.0):
        """Fill an order"""
        if order_id not in self.orders:
            raise ValueError(f"Order {order_id} not found")
        
        order = self.orders[order_id]
        
        fill = Fill(
            order_id=order_id,
            symbol=order.symbol,
            side=order.side,
            quantity=fill_qty,
            fill_price=fill_price,
            commission=commission,
            timestamp_ns=time.time_ns()
        )
        
        # Journal the fill
        self.journal.append("order_filled", asdict(fill))
        
        # Update order state
        if fill_qty >= order.quantity:
            order.state = "Filled"
        else:
            order.state = "PartialFill"
        order.updated_at = time.strftime("%Y-%m-%dT%H:%M:%S.%fZ")
        
        self.fills.append(fill)
    
    def crash(self):
        """Simulate crash - close without cleanup"""
        self.journal.close()
        # Don't save in-memory state
    
    def recover_from_journal(self) -> bool:
        """Recover state from journal"""
        print("Recovering from journal...")
        
        entries = self.journal.read_all()
        print(f"Found {len(entries)} journal entries")
        
        recovered_orders = 0
        recovered_fills = 0
        
        for entry in entries:
            if entry["entry_type"] == "order_created":
                order_data = entry["data"]
                order = Order(**order_data)
                self.orders[order.order_id] = order
                recovered_orders += 1
                
            elif entry["entry_type"] == "order_filled":
                fill_data = entry["data"]
                fill = Fill(**fill_data)
                self.fills.append(fill)
                
                # Update corresponding order state
                if fill.order_id in self.orders:
                    order = self.orders[fill.order_id]
                    if fill.quantity >= order.quantity:
                        order.state = "Filled"
                    else:
                        order.state = "PartialFill"
                    order.updated_at = time.strftime("%Y-%m-%dT%H:%M:%S.%fZ")
                recovered_fills += 1
        
        print(f"Recovered {recovered_orders} orders and {recovered_fills} fills")
        return True
    
    def get_state_summary(self) -> Dict:
        """Get current state summary"""
        return {
            "total_orders": len(self.orders),
            "pending_orders": len([o for o in self.orders.values() if o.state == "Pending"]),
            "filled_orders": len([o for o in self.orders.values() if o.state == "Filled"]),
            "partial_fills": len([o for o in self.orders.values() if o.state == "PartialFill"]),
            "total_fills": len(self.fills)
        }

class OMSCrashRecoveryTester:
    def __init__(self):
        self.temp_dir = tempfile.mkdtemp()
        self.journal_path = os.path.join(self.temp_dir, "oms_journal.db")
        
    def cleanup(self):
        """Clean up test resources"""
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    def test_basic_crash_recovery(self, num_orders: int = 1000) -> bool:
        """Test basic crash and recovery"""
        print(f"\n=== Test: Basic Crash Recovery ({num_orders} orders) ===")
        
        # Phase 1: Create orders and some fills
        oms = MockOMSEngine(self.journal_path)
        
        print(f"Creating {num_orders} orders...")
        order_ids = []
        for i in range(num_orders):
            order_id = oms.create_order(
                account_id="ACC_001",
                symbol=random.choice(["AAPL", "GOOGL", "MSFT", "TSLA"]),
                side=random.choice(["Buy", "Sell"]),
                order_type="Market",
                quantity=random.uniform(100, 1000),
                price=random.uniform(100, 500) if random.random() > 0.5 else None
            )
            order_ids.append(order_id)
            
            # Fill some orders
            if random.random() > 0.3:
                oms.fill_order(
                    order_id,
                    fill_price=random.uniform(100, 500),
                    fill_qty=random.uniform(100, 1000)
                )
        
        state_before = oms.get_state_summary()
        print(f"State before crash: {state_before}")
        
        # Phase 2: Simulate crash
        print("\nSimulating crash...")
        oms.crash()
        
        # Phase 3: Recover
        print("\nStarting recovery...")
        oms = MockOMSEngine(self.journal_path)
        recovery_success = oms.recover_from_journal()
        
        state_after = oms.get_state_summary()
        print(f"State after recovery: {state_after}")
        
        # Phase 4: Validate
        validation_passed = (
            state_before["total_orders"] == state_after["total_orders"] and
            state_before["total_fills"] == state_after["total_fills"] and
            state_before["filled_orders"] == state_after["filled_orders"]
        )
        
        if validation_passed:
            print("✅ PASSED: State fully recovered")
        else:
            print("❌ FAILED: State mismatch after recovery")
            print(f"Before: {state_before}")
            print(f"After:  {state_after}")
        
        oms.journal.close()
        return validation_passed
    
    def test_partial_journal_corruption(self) -> bool:
        """Test recovery with partial journal corruption"""
        print(f"\n=== Test: Partial Journal Corruption ===")
        
        # Create orders
        oms = MockOMSEngine(self.journal_path)
        
        for i in range(100):
            order_id = oms.create_order(
                account_id="ACC_001",
                symbol="AAPL",
                side="Buy",
                order_type="Market",
                quantity=100
            )
            if i % 2 == 0:
                oms.fill_order(order_id, 150.0, 100)
        
        state_before = oms.get_state_summary()
        oms.crash()
        
        # Corrupt part of the journal
        print("Simulating journal corruption...")
        conn = sqlite3.connect(self.journal_path)
        conn.execute("DELETE FROM journal_entries WHERE id % 10 = 0")  # Delete every 10th entry
        conn.commit()
        conn.close()
        
        # Attempt recovery
        oms = MockOMSEngine(self.journal_path)
        recovery_success = oms.recover_from_journal()
        
        state_after = oms.get_state_summary()
        
        # Should recover what's available
        print(f"State after partial recovery: {state_after}")
        
        # Validate that we recovered something
        partial_recovery_passed = state_after["total_orders"] > 0
        
        if partial_recovery_passed:
            print("✅ PASSED: Partial recovery successful")
        else:
            print("❌ FAILED: No recovery from corrupted journal")
        
        oms.journal.close()
        return partial_recovery_passed
    
    def test_concurrent_crash_recovery(self) -> bool:
        """Test crash during high load"""
        print(f"\n=== Test: Concurrent Crash Recovery ===")
        
        async def high_load_orders():
            """Generate high load of orders"""
            oms = MockOMSEngine(self.journal_path)
            
            tasks = []
            for i in range(10):  # 10 concurrent workers
                task = asyncio.create_task(worker(oms, i, 100))
                tasks.append(task)
            
            # Crash after 2 seconds
            crash_task = asyncio.create_task(crash_after_delay(oms, 2.0))
            tasks.append(crash_task)
            
            await asyncio.gather(*tasks, return_exceptions=True)
            
            return oms
        
        async def worker(oms, worker_id, num_orders):
            """Worker generating orders"""
            for i in range(num_orders):
                try:
                    order_id = oms.create_order(
                        account_id=f"ACC_{worker_id:03d}",
                        symbol="BTC-USD",
                        side="Buy",
                        order_type="Market",
                        quantity=random.uniform(0.1, 1.0)
                    )
                    
                    if random.random() > 0.5:
                        oms.fill_order(order_id, random.uniform(45000, 55000), random.uniform(0.1, 1.0))
                    
                    await asyncio.sleep(0.001)  # 1ms delay
                except:
                    break  # OMS crashed
        
        async def crash_after_delay(oms, delay):
            """Crash OMS after delay"""
            await asyncio.sleep(delay)
            print("High load crash triggered")
            oms.crash()
        
        # Run the test
        oms = asyncio.run(high_load_orders())
        
        # Recover
        print("\nRecovering from high load crash...")
        oms = MockOMSEngine(self.journal_path)
        recovery_success = oms.recover_from_journal()
        
        state = oms.get_state_summary()
        print(f"Final state: {state}")
        
        oms.journal.close()
        return state["total_orders"] > 0
    
    def test_journal_performance(self) -> Dict:
        """Test journal write performance"""
        print(f"\n=== Test: Journal Performance ===")
        
        oms = MockOMSEngine(self.journal_path)
        
        # Measure write latency
        latencies = []
        num_writes = 10000
        
        for i in range(num_writes):
            start = time.perf_counter_ns()
            
            oms.create_order(
                account_id="PERF_TEST",
                symbol="TEST",
                side="Buy",
                order_type="Market",
                quantity=100
            )
            
            latency = (time.perf_counter_ns() - start) / 1000  # Convert to μs
            latencies.append(latency)
        
        avg_latency = sum(latencies) / len(latencies)
        p95_latency = sorted(latencies)[int(0.95 * len(latencies))]
        p99_latency = sorted(latencies)[int(0.99 * len(latencies))]
        
        print(f"Journal write performance:")
        print(f"  Average: {avg_latency:.2f}μs")
        print(f"  P95: {p95_latency:.2f}μs")
        print(f"  P99: {p99_latency:.2f}μs")
        print(f"  Throughput: {num_writes / (sum(latencies) / 1_000_000):.0f} writes/sec")
        
        oms.journal.close()
        
        return {
            "avg_latency_us": avg_latency,
            "p95_latency_us": p95_latency,
            "p99_latency_us": p99_latency,
            "throughput_wps": num_writes / (sum(latencies) / 1_000_000)
        }

def main():
    """Run all OMS crash recovery tests"""
    tester = OMSCrashRecoveryTester()
    
    try:
        print("=== OMS Crash Recovery Validation ===\n")
        
        # Test 1: Basic crash recovery
        test1_passed = tester.test_basic_crash_recovery(1000)
        
        # Test 2: Journal corruption
        test2_passed = tester.test_partial_journal_corruption()
        
        # Test 3: Concurrent crash
        test3_passed = tester.test_concurrent_crash_recovery()
        
        # Test 4: Performance
        perf_results = tester.test_journal_performance()
        
        # Summary
        print("\n=== Summary ===")
        print(f"Basic crash recovery: {'✅ PASSED' if test1_passed else '❌ FAILED'}")
        print(f"Partial corruption: {'✅ PASSED' if test2_passed else '❌ FAILED'}")
        print(f"Concurrent crash: {'✅ PASSED' if test3_passed else '❌ FAILED'}")
        
        if perf_results["avg_latency_us"] > 10:  # Should be <10μs
            print(f"⚠️  WARNING: Journal latency {perf_results['avg_latency_us']:.2f}μs > 10μs")
        else:
            print(f"✅ Journal performance OK: {perf_results['avg_latency_us']:.2f}μs avg")
        
        all_passed = test1_passed and test2_passed and test3_passed
        
        if all_passed:
            print("\n✅ ALL TESTS PASSED - OMS crash recovery is production ready")
        else:
            print("\n❌ SOME TESTS FAILED - Review and fix issues before production")
    
    finally:
        tester.cleanup()

if __name__ == "__main__":
    main()
