#!/usr/bin/env python3
"""
Comprehensive test suite for Risk Manager validation.
Tests all risk limits and circuit breaker functionality.
"""

import asyncio
import logging
import sys
from pathlib import Path
from datetime import datetime, timedelta
from typing import Dict, List

# Add src to path
sys.path.insert(0, str(Path(__file__).parent))

from src.risk.manager import RiskManager, RiskLevel
from src.core.models import Order, OrderSide, OrderType, OrderStatus, Position

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


async def test_position_size_limit():
    """Test that position size limits are enforced."""
    logger.info("Testing position size limit enforcement...")
    
    # Initialize risk manager with $1000 max position size
    risk_manager = RiskManager(max_position_size=1000.0)
    
    # Create an order that exceeds the limit
    large_order = Order(
        id="test_1",
        symbol="BTC/USDT",
        side=OrderSide.BUY,
        type=OrderType.MARKET,
        quantity=1.0,  # Will exceed $1000 at $50000 price
        price=50000.0
    )
    
    # Should reject the order
    positions = {"BTC/USDT": Position("BTC/USDT", 0, 0, 0)}
    is_valid = await risk_manager.validate_order(large_order, positions)
    
    assert not is_valid, "Order exceeding position size limit should be rejected"
    logger.info("✓ Position size limit enforced")
    
    # Test order within limit
    small_order = Order(
        id="test_2",
        symbol="BTC/USDT",
        side=OrderSide.BUY,
        type=OrderType.MARKET,
        quantity=0.01,  # Within $1000 limit
        price=50000.0
    )
    
    is_valid = await risk_manager.validate_order(small_order, positions)
    assert is_valid, "Order within position size limit should be accepted"
    logger.info("✓ Valid order accepted")


async def test_daily_loss_limit():
    """Test that daily loss limits trigger circuit breaker."""
    logger.info("Testing daily loss limit...")
    
    # Initialize risk manager with $100 daily loss limit
    risk_manager = RiskManager(max_daily_loss=100.0)
    
    # Set daily PnL directly to exceed limit
    risk_manager.daily_pnl = -150.0  # $150 loss exceeds $100 limit
    
    # Create a new order
    order = Order(
        id="test_3",
        symbol="BTC/USDT",
        side=OrderSide.BUY,
        type=OrderType.MARKET,
        quantity=0.01,
        price=50000.0
    )
    
    positions = {}
    is_valid = await risk_manager.validate_order(order, positions)
    
    assert not is_valid, "Orders should be rejected when daily loss limit exceeded"
    assert risk_manager.circuit_breaker_active, "Circuit breaker should be activated"
    logger.info("✓ Daily loss limit enforced, circuit breaker activated")


async def test_drawdown_limit():
    """Test that maximum drawdown limits trigger circuit breaker."""
    logger.info("Testing maximum drawdown limit...")
    
    # Initialize risk manager with 20% max drawdown
    risk_manager = RiskManager(max_drawdown=0.20)
    
    # Set up drawdown scenario
    risk_manager.daily_start_balance = 10000.0
    risk_manager.peak_balance = 12000.0
    risk_manager.current_balance = 9000.0  # 25% drawdown from peak
    
    # Update metrics (should trigger circuit breaker)
    positions = {}
    await risk_manager.update_metrics(positions, 9000.0)
    
    assert risk_manager.circuit_breaker_active, "Circuit breaker should activate on max drawdown"
    assert risk_manager.limits['drawdown'].current > 0.20, "Drawdown should be calculated correctly"
    logger.info("✓ Maximum drawdown limit enforced")


async def test_leverage_limit():
    """Test that leverage limits are enforced."""
    logger.info("Testing leverage limit...")
    
    # Initialize risk manager with 3x max leverage
    risk_manager = RiskManager(max_leverage=3.0)
    
    # Create positions with high exposure
    positions = {
        "BTC/USDT": Position("BTC/USDT", 0.5, 50000.0, 50000.0),  # $25000 exposure
        "ETH/USDT": Position("ETH/USDT", 10.0, 3000.0, 3000.0)   # $30000 exposure
    }
    
    # Set balance to make leverage exceed limit
    risk_manager.current_balance = 15000.0  # Total exposure $55000, leverage 3.67x
    
    # Create new order
    order = Order(
        id="test_4",
        symbol="BTC/USDT",
        side=OrderSide.BUY,
        type=OrderType.MARKET,
        quantity=0.01,
        price=50000.0
    )
    
    is_valid = await risk_manager.validate_order(order, positions)
    
    assert not is_valid, "Orders should be rejected when leverage limit exceeded"
    logger.info("✓ Leverage limit enforced")


async def test_order_frequency_limit():
    """Test that order frequency limits are enforced."""
    logger.info("Testing order frequency limit...")
    
    risk_manager = RiskManager()
    
    # Create multiple recent orders
    now = datetime.utcnow()
    for i in range(15):  # Exceed 10 orders per minute limit
        order = Order(
            id=f"freq_test_{i}",
            symbol="BTC/USDT",
            side=OrderSide.BUY,
            type=OrderType.MARKET,
            quantity=0.01,
            price=50000.0,
            timestamp=now - timedelta(seconds=i*2)
        )
        risk_manager.open_orders.append(order)
    
    # Try to add another order
    new_order = Order(
        id="freq_test_new",
        symbol="BTC/USDT",
        side=OrderSide.BUY,
        type=OrderType.MARKET,
        quantity=0.01,
        price=50000.0,
        timestamp=now
    )
    
    positions = {}
    is_valid = await risk_manager.validate_order(new_order, positions)
    
    assert not is_valid, "Orders should be rejected when frequency limit exceeded"
    logger.info("✓ Order frequency limit enforced")


async def test_risk_metrics_calculation():
    """Test risk metrics calculation accuracy."""
    logger.info("Testing risk metrics calculation...")
    
    risk_manager = RiskManager()
    
    # Add some trade history with clear win/loss pattern
    risk_manager.trade_history = [
        {'pnl': 100.0, 'timestamp': datetime.utcnow()},   # Win
        {'pnl': -50.0, 'timestamp': datetime.utcnow()},   # Loss
        {'pnl': 150.0, 'timestamp': datetime.utcnow()},   # Win
        {'pnl': -25.0, 'timestamp': datetime.utcnow()},   # Loss
        {'pnl': 200.0, 'timestamp': datetime.utcnow()},   # Win
        {'pnl': -30.0, 'timestamp': datetime.utcnow()},   # Loss
        {'pnl': 120.0, 'timestamp': datetime.utcnow()},   # Win
        {'pnl': -40.0, 'timestamp': datetime.utcnow()}    # Loss
    ]
    
    risk_manager.daily_pnl = 425.0  # Sum of all PnL
    risk_manager.total_exposure = 5000.0
    risk_manager.peak_balance = 12000.0
    risk_manager.current_balance = 11750.0
    
    # Set drawdown for testing
    risk_manager.limits['drawdown'].current = 0.0208  # (12000-11750)/12000
    
    metrics = risk_manager.get_risk_metrics()
    
    assert metrics.daily_pnl == 425.0, "Daily PnL should be correct"
    assert metrics.win_rate == 0.5, "Win rate should be 50% (4 wins out of 8)"
    assert metrics.profit_factor > 0, "Profit factor should be positive"
    
    logger.info("✓ Risk metrics calculated correctly")


async def test_circuit_breaker_reset():
    """Test circuit breaker manual reset functionality."""
    logger.info("Testing circuit breaker reset...")
    
    risk_manager = RiskManager(max_daily_loss=100.0)
    
    # Trigger circuit breaker
    risk_manager.daily_pnl = -150.0  # Exceeds $100 limit
    
    order = Order(
        id="test_cb",
        symbol="BTC/USDT",
        side=OrderSide.BUY,
        type=OrderType.MARKET,
        quantity=0.01,
        price=50000.0  # Ensure price is not None
    )
    
    await risk_manager.validate_order(order, {})
    
    assert risk_manager.circuit_breaker_active, "Circuit breaker should be active"
    
    # Reset circuit breaker
    risk_manager.reset_circuit_breaker()
    
    assert not risk_manager.circuit_breaker_active, "Circuit breaker should be reset"
    assert risk_manager.circuit_breaker_reason is None, "Reason should be cleared"
    
    logger.info("✓ Circuit breaker reset successful")


async def test_risk_report_generation():
    """Test risk report generation."""
    logger.info("Testing risk report generation...")
    
    risk_manager = RiskManager(max_position_size=1000.0, max_daily_loss=100.0)
    
    # Set some state
    risk_manager.daily_pnl = 50.0
    risk_manager.total_exposure = 5000.0
    risk_manager.current_balance = 10500.0
    risk_manager.limits['drawdown'].current = 0.05
    
    report = risk_manager.get_risk_report()
    
    # Verify report structure
    assert 'timestamp' in report, "Report should include timestamp"
    assert 'circuit_breaker' in report, "Report should include circuit breaker status"
    assert 'limits' in report, "Report should include limits"
    assert 'metrics' in report, "Report should include metrics"
    assert 'account' in report, "Report should include account info"
    
    # Verify values
    assert report['metrics']['daily_pnl'] == 50.0, "Daily PnL should match"
    assert report['account']['current_balance'] == 10500.0, "Balance should match"
    assert 'vpin' in report['metrics'], "VPIN should be included"
    
    logger.info("✓ Risk report generated successfully")


async def main():
    """Run all risk manager tests."""
    logger.info("Starting Risk Manager Validation Tests...")
    
    tests = [
        test_position_size_limit,
        test_daily_loss_limit,
        test_drawdown_limit,
        test_leverage_limit,
        test_order_frequency_limit,
        test_risk_metrics_calculation,
        test_circuit_breaker_reset,
        test_risk_report_generation
    ]
    
    passed = 0
    failed = 0
    
    for test in tests:
        try:
            await test()
            passed += 1
        except Exception as e:
            logger.error(f"Test failed: {test.__name__} - {e}")
            failed += 1
    
    logger.info(f"\nRisk Manager Test Results:")
    logger.info(f"✓ Passed: {passed}")
    logger.info(f"✗ Failed: {failed}")
    
    if failed == 0:
        logger.info("\n🎉 All risk manager tests passed!")
        return True
    else:
        logger.error(f"\n❌ {failed} tests failed!")
        return False


if __name__ == "__main__":
    success = asyncio.run(main())
    sys.exit(0 if success else 1)
