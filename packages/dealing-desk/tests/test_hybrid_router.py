#!/usr/bin/env python3
"""
Test suite for Hybrid Risk Router (Task 4.1)
Validates sub-100ns routing latency and correct B-Book/A-Book decisions.
"""

import asyncio
import json
import time
from datetime import datetime
import numpy as np
import pytest

from src.hybrid_router import HybridRiskRouter, OrderFrame, RoutingDecision, MockXDPProgram


class TestHybridRouter:
    """Test hybrid risk router functionality."""
    
    def setup_method(self):
        """Setup test environment."""
        self.router = HybridRiskRouter(
            redis_url="redis://localhost:6379",
            confidence_threshold=0.6,
            sharpe_threshold=1.5
        )
        
        # Mock handlers
        self.a_book_orders = []
        self.b_book_orders = []
        
        async def a_book_handler(order):
            self.a_book_orders.append(order)
        
        async def b_book_handler(order):
            self.b_book_orders.append(order)
        
        self.router.set_handlers(a_book_handler, b_book_handler)
    
    async def test_b_book_routing_high_confidence(self):
        """Test B-Book routing for high confidence, acceptable Sharpe."""
        # Create order with high confidence
        order = OrderFrame(
            order_id="test-001",
            tenant_id="tenant-a",
            symbol="BTCUSDT",
            side="BUY",
            quantity=1.0,
            price=50000.0,
            confidence=0.8,  # High confidence
            timestamp=datetime.utcnow()
        )
        
        # Set acceptable Sharpe ratio
        await self.router.update_sharpe_ratio("BTCUSDT", "tenant-a", 1.0)
        
        # Route order
        decision = await self.router.route_order(order)
        
        # Verify B-Book routing
        assert decision.route == "B_BOOK", "Should route to B-Book"
        assert "Adequate confidence" in decision.reason
        assert decision.latency_ns < 100, f"Latency {decision.latency_ns}ns should be <100ns"
        
        # Verify handler was called
        assert len(self.b_book_orders) == 1, "B-Book handler should be called"
        assert len(self.a_book_orders) == 0, "A-Book handler should not be called"
        
        print(f"✓ B-Book routing: {decision.latency_ns}ns (<100ns)")
    
    async def test_a_book_routing_low_confidence(self):
        """Test A-Book routing for low confidence."""
        # Create order with low confidence
        order = OrderFrame(
            order_id="test-002",
            tenant_id="tenant-a",
            symbol="BTCUSDT",
            side="BUY",
            quantity=1.0,
            price=50000.0,
            confidence=0.4,  # Low confidence
            timestamp=datetime.utcnow()
        )
        
        # Set acceptable Sharpe ratio
        await self.router.update_sharpe_ratio("BTCUSDT", "tenant-a", 1.0)
        
        # Route order
        decision = await self.router.route_order(order)
        
        # Verify A-Book routing
        assert decision.route == "A_BOOK", "Should route to A-Book"
        assert "Low confidence" in decision.reason
        assert decision.latency_ns < 100, f"Latency {decision.latency_ns}ns should be <100ns"
        
        # Verify handler was called
        assert len(self.a_book_orders) == 1, "A-Book handler should be called"
        assert len(self.b_book_orders) == 0, "B-Book handler should not be called"
        
        print(f"✓ A-Book routing (low confidence): {decision.latency_ns}ns (<100ns)")
    
    async def test_a_book_routing_high_sharpe(self):
        """Test A-Book routing for high Sharpe ratio."""
        # Create order with high confidence
        order = OrderFrame(
            order_id="test-003",
            tenant_id="tenant-a",
            symbol="BTCUSDT",
            side="BUY",
            quantity=1.0,
            price=50000.0,
            confidence=0.8,  # High confidence
            timestamp=datetime.utcnow()
        )
        
        # Set high Sharpe ratio
        await self.router.update_sharpe_ratio("BTCUSDT", "tenant-a", 2.0)
        
        # Route order
        decision = await self.router.route_order(order)
        
        # Verify A-Book routing
        assert decision.route == "A_BOOK", "Should route to A-Book"
        assert "high Sharpe" in decision.reason
        assert decision.latency_ns < 100, f"Latency {decision.latency_ns}ns should be <100ns"
        
        print(f"✓ A-Book routing (high Sharpe): {decision.latency_ns}ns (<100ns)")
    
    async def test_sub_100ns_latency_requirement(self):
        """Test that routing meets sub-100ns latency requirement."""
        # Route multiple orders to check average latency
        latencies = []
        
        for i in range(100):
            order = OrderFrame(
                order_id=f"test-{i:04d}",
                tenant_id="tenant-a",
                symbol="BTCUSDT",
                side="BUY",
                quantity=1.0,
                price=50000.0,
                confidence=0.7,
                timestamp=datetime.utcnow()
            )
            
            decision = await self.router.route_order(order)
            latencies.append(decision.latency_ns)
        
        # Check latency requirements
        avg_latency = np.mean(latencies)
        max_latency = np.max(latencies)
        p95_latency = np.percentile(latencies, 95)
        
        assert avg_latency < 100, f"Average latency {avg_latency}ns should be <100ns"
        assert max_latency < 100, f"Max latency {max_latency}ns should be <100ns"
        assert p95_latency < 100, f"P95 latency {p95_latency}ns should be <100ns"
        
        print(f"✓ Latency test: avg={avg_latency}ns, max={max_latency}ns, p95={p95_latency}ns")
    
    async def test_hstr_state_tensor(self):
        """Test HSTR state tensor integration."""
        # Update Sharpe ratios
        await self.router.update_sharpe_ratio("BTCUSDT", "tenant-a", 1.2)
        await self.router.update_sharpe_ratio("ETHUSDT", "tenant-a", 0.8)
        await self.router.update_sharpe_ratio("BTCUSDT", "tenant-b", 2.1)
        
        # Create orders
        orders = [
            OrderFrame("test-a-btc", "tenant-a", "BTCUSDT", "BUY", 1.0, 50000.0, 0.7, datetime.utcnow()),
            OrderFrame("test-a-eth", "tenant-a", "ETHUSDT", "BUY", 1.0, 3000.0, 0.7, datetime.utcnow()),
            OrderFrame("test-b-btc", "tenant-b", "BTCUSDT", "BUY", 1.0, 50000.0, 0.7, datetime.utcnow())
        ]
        
        # Route orders and verify decisions
        decisions = []
        for order in orders:
            decision = await self.router.route_order(order)
            decisions.append(decision)
        
        # Verify routing based on Sharpe ratios
        assert decisions[0].route == "B_BOOK", "tenant-a BTCUSDT (Sharpe=1.2) should route to B-Book"
        assert decisions[1].route == "B_BOOK", "tenant-a ETHUSDT (Sharpe=0.8) should route to B-Book"
        assert decisions[2].route == "A_BOOK", "tenant-b BTCUSDT (Sharpe=2.1) should route to A-Book"
        
        print("✓ HSTR state tensor integration working correctly")


class TestMockXDP:
    """Test mock XDP program simulation."""
    
    def setup_method(self):
        """Setup test environment."""
        self.router = HybridRiskRouter("redis://localhost:6379")
        self.xdp = MockXDPProgram(self.router)
    
    async def test_xdp_pass_action(self):
        """Test XDP_PASS action for A-Book routing."""
        # Create order with low confidence
        order = OrderFrame(
            order_id="test-xdp-001",
            tenant_id="tenant-a",
            symbol="BTCUSDT",
            side="BUY",
            quantity=1.0,
            price=50000.0,
            confidence=0.4,  # Low confidence
            timestamp=datetime.utcnow()
        )
        
        # Process as XDP packet
        packet_data = json.dumps(asdict(order)).encode()
        action = await self.xdp.process_packet(packet_data)
        
        assert action == "XDP_PASS", "Should return XDP_PASS for A-Book routing"
        
        print(f"✓ XDP_PASS action returned for A-Book routing")
    
    async def test_xdp_redirect_action(self):
        """Test XDP_REDIRECT action for B-Book routing."""
        # Create order with high confidence
        order = OrderFrame(
            order_id="test-xdp-002",
            tenant_id="tenant-a",
            symbol="BTCUSDT",
            side="BUY",
            quantity=1.0,
            price=50000.0,
            confidence=0.8,  # High confidence
            timestamp=datetime.utcnow()
        )
        
        # Set acceptable Sharpe ratio
        await self.router.update_sharpe_ratio("BTCUSDT", "tenant-a", 1.0)
        
        # Process as XDP packet
        packet_data = json.dumps(asdict(order)).encode()
        action = await self.xdp.process_packet(packet_data)
        
        assert action == "XDP_REDIRECT", "Should return XDP_REDIRECT for B-Book routing"
        
        print(f"✓ XDP_REDIRECT action returned for B-Book routing")


async def run_all_tests():
    """Run all hybrid router tests."""
    print("\n=== Running Hybrid Router Tests ===\n")
    
    # Test hybrid router
    router_tests = TestHybridRouter()
    router_tests.setup_method()
    
    await router_tests.test_b_book_routing_high_confidence()
    await router_tests.test_a_book_routing_low_confidence()
    await router_tests.test_a_book_routing_high_sharpe()
    await router_tests.test_sub_100ns_latency_requirement()
    await router_tests.test_hstr_state_tensor()
    
    # Test mock XDP
    xdp_tests = TestMockXDP()
    xdp_tests.setup_method()
    
    await xdp_tests.test_xdp_pass_action()
    await xdp_tests.test_xdp_redirect_action()
    
    # Save benchmark
    router_tests.router.save_benchmark("bench-xdp-routing.log")
    
    print("\n✅ All hybrid router tests passed!")


if __name__ == "__main__":
    asyncio.run(run_all_tests())
