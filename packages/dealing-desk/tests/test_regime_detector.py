#!/usr/bin/env python3
"""
Test suite for Regime-Conditioned Execution Veto (Task 4.3)
Validates VPIN calculation, Hurst exponent detection, and sub-5μs execution freeze.
"""

import asyncio
import json
import time
from datetime import datetime, timedelta
import numpy as np
import pytest

from src.regime_detector import RegimeDetector, VolumeBucket
from src.execution_guard import ExecutionGuard, get_execution_guard


class TestRegimeDetector:
    """Test regime detection functionality."""
    
    def setup_method(self):
        """Setup test environment."""
        self.detector = RegimeDetector(
            bucket_size=1000,  # Small bucket for testing
            vpin_window=10,
            vpin_percentile=0.95,
            hurst_window=30,
            freeze_duration=5  # 5 seconds for testing
        )
    
    async def test_vpin_calculation(self):
        """Test VPIN calculation with synthetic data."""
        # Generate synthetic toxic flow
        base_time = datetime.utcnow()
        
        # Create high imbalance (toxic) flow
        for i in range(20):
            timestamp = base_time + timedelta(seconds=i)
            
            # 80% buy volume, 20% sell volume - high imbalance
            if i % 5 == 0:
                # Large buy order
                await self.detector.process_tick(timestamp, 100.0, 500.0, "BUY")
            else:
                # Small sell orders
                await self.detector.process_tick(timestamp, 100.1, 100.0, "SELL")
        
        # Check VPIN is calculated
        status = self.detector.get_regime_status()
        assert status["current_vpin"] > 0, "VPIN should be > 0 with imbalance"
        
        print(f"✓ VPIN calculated: {status['current_vpin']:.4f}")
    
    async def test_hurst_exponent_choppy(self):
        """Test Hurst exponent detection for choppy market."""
        # Generate mean-reverting price series
        base_time = datetime.utcnow()
        base_price = 100.0
        
        for i in range(100):
            timestamp = base_time + timedelta(seconds=i)
            
            # Mean-reverting price
            deviation = np.sin(i * 0.1) * 0.5
            price = base_price + deviation
            
            await self.detector.process_tick(timestamp, price, 100.0, "BUY")
        
        # Check Hurst exponent
        hurst = self.detector._calculate_hurst_exponent()
        assert hurst < 0.5, f"Hurst should be < 0.5 for choppy market, got {hurst}"
        
        print(f"✓ Hurst exponent for choppy market: {hurst:.3f}")
    
    async def test_hurst_exponent_trending(self):
        """Test Hurst exponent detection for trending market."""
        # Generate trending price series
        base_time = datetime.utcnow()
        base_price = 100.0
        
        for i in range(100):
            timestamp = base_time + timedelta(seconds=i)
            
            # Trending price
            price = base_price + i * 0.01
            
            await self.detector.process_tick(timestamp, price, 100.0, "BUY")
        
        # Check Hurst exponent
        hurst = self.detector._calculate_hurst_exponent()
        assert hurst > 0.5, f"Hurst should be > 0.5 for trending market, got {hurst}"
        
        print(f"✓ Hurst exponent for trending market: {hurst:.3f}")
    
    async def test_toxic_regime_freeze(self):
        """Test execution freeze on toxic regime detection."""
        # Generate toxic flow to trigger freeze
        base_time = datetime.utcnow()
        
        # Create very high imbalance
        for i in range(50):
            timestamp = base_time + timedelta(seconds=i)
            
            # Extreme buy pressure
            await self.detector.process_tick(timestamp, 100.0, 1000.0, "BUY")
        
        # Check if freeze is triggered
        assert not self.detector.is_execution_allowed(), "Execution should be frozen in toxic regime"
        
        # Check freeze duration
        status = self.detector.get_regime_status()
        assert status["freeze_until"] is not None, "Freeze until should be set"
        
        print(f"✓ Toxic regime freeze triggered until: {status['freeze_until']}")
    
    async def test_vpin_toxic_kill_trace(self):
        """Test that toxic kill trace is generated correctly."""
        # Generate toxic flow
        base_time = datetime.utcnow()
        
        for i in range(30):
            timestamp = base_time + timedelta(seconds=i)
            await self.detector.process_tick(timestamp, 100.0, 500.0, "BUY")
        
        # Save trace
        trace_file = "test_vpin_toxic_kill.trace"
        self.detector.save_trace(trace_file)
        
        # Verify trace file
        with open(trace_file, 'r') as f:
            trace = json.load(f)
        
        assert "vpin_history" in trace, "VPIN history should be in trace"
        assert "statistics" in trace, "Statistics should be in trace"
        assert trace["statistics"]["toxic_events"] > 0, "Should have toxic events"
        
        print(f"✓ Toxic kill trace saved to {trace_file}")


class TestExecutionGuard:
    """Test execution guard sub-5μs performance."""
    
    def setup_method(self):
        """Setup test environment."""
        self.guard = ExecutionGuard()
    
    async def test_sub_5_microsecond_check(self):
        """Test execution guard check completes in <5μs."""
        from ..handoff.src.models.handoff_package import (
            HandoffPackage, HandoffStatus, AgentType, TaskDefinition
        )
        
        # Create test handoff
        task = TaskDefinition(
            id="test-task",
            type="trading",
            priority="high",
            description="Test execution",
            context={}
        )
        
        handoff = HandoffPackage(
            id="test-handoff",
            task=task,
            status=HandoffStatus.WORKING,
            source_agent=AgentType.CLAUDE_46,
            target_agent=AgentType.GEMMA_4
        )
        
        # Measure latency
        start_time = time.time_ns()
        allowed = await self.guard.check_execution_allowed(handoff)
        latency_ns = time.time_ns() - start_time
        
        # Verify sub-5μs requirement
        assert latency_ns < 5000, f"Check took {latency_ns}ns, should be <5000ns"
        assert allowed, "Execution should be allowed in normal regime"
        
        print(f"✓ Execution guard check: {latency_ns}ns (<5μs)")
    
    async def test_execution_block_on_toxic(self):
        """Test execution is blocked during toxic regime."""
        from ..handoff.src.models.handoff_package import (
            HandoffPackage, HandoffStatus, AgentType, TaskDefinition
        )
        
        # Simulate toxic regime
        base_time = datetime.utcnow()
        for i in range(30):
            await self.guard.process_tick(base_time + timedelta(seconds=i), 100.0, 1000.0, "BUY")
        
        # Create test handoff
        task = TaskDefinition(
            id="test-task",
            type="trading",
            priority="high",
            description="Test execution",
            context={}
        )
        
        handoff = HandoffPackage(
            id="test-handoff",
            task=task,
            status=HandoffStatus.WORKING,
            source_agent=AgentType.CLAUDE_46,
            target_agent=AgentType.GEMMA_4
        )
        
        # Check execution is blocked
        allowed = await self.guard.check_execution_allowed(handoff)
        assert not allowed, "Execution should be blocked in toxic regime"
        assert handoff.status == HandoffStatus.FAILED, "Handoff should be marked as failed"
        
        print("✓ Execution blocked in toxic regime")


async def run_all_tests():
    """Run all regime detector tests."""
    print("\n=== Running Regime Detector Tests ===\n")
    
    # Test regime detector
    detector_tests = TestRegimeDetector()
    detector_tests.setup_method()
    
    await detector_tests.test_vpin_calculation()
    await detector_tests.test_hurst_exponent_choppy()
    await detector_tests.test_hurst_exponent_trending()
    await detector_tests.test_toxic_regime_freeze()
    await detector_tests.test_vpin_toxic_kill_trace()
    
    # Test execution guard
    guard_tests = TestExecutionGuard()
    guard_tests.setup_method()
    
    await guard_tests.test_sub_5_microsecond_check()
    await guard_tests.test_execution_block_on_toxic()
    
    print("\n✅ All regime detector tests passed!")


if __name__ == "__main__":
    asyncio.run(run_all_tests())
