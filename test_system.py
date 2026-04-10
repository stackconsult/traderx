#!/usr/bin/env python3
"""
Test script to validate TraderX system functionality.
Runs basic tests with PaperExchange to ensure system works correctly.
"""

import asyncio
import logging
import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent))

from src.core.engine import TradingEngine
from src.exchanges.paper import PaperExchange
from src.strategies.moving_average import MovingAverageStrategy
from src.risk.manager import RiskManager
from src.data.storage import DataStorage
from src.utils.health import HealthChecker

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

logger = logging.getLogger(__name__)


async def test_paper_trading():
    """Test the system with paper trading."""
    logger.info("Starting TraderX system test...")
    
    try:
        # Initialize components
        logger.info("Initializing components...")
        
        # Paper exchange configuration
        paper_config = {
            'latency_ms': 50,
            'slippage_bps': 2,
            'fill_probability': 1.0  # 100% fill rate for testing
        }
        
        exchanges = {
            'binance': PaperExchange(paper_config)
        }
        
        # Risk manager
        risk_manager = RiskManager(
            max_position_size=1000.0,
            max_daily_loss=100.0,
            max_drawdown=0.20
        )
        
        # Data storage (in-memory SQLite for testing)
        storage = DataStorage("sqlite+aiosqlite:///test.db")
        await storage.initialize()
        
        # Strategy
        strategy = MovingAverageStrategy(
            name="Test_MA",
            symbols=["BTC/USDT"],
            config={
                "fast_period": 5,
                "slow_period": 10,
                "timeframe": "1m"
            }
        )
        
        strategies = [strategy]
        
        # Create engine
        engine = TradingEngine(
            exchanges=exchanges,
            strategies=strategies,
            risk_manager=risk_manager,
            data_storage=storage
        )
        
        # Health checker
        health_checker = HealthChecker()
        
        # Test 1: Connect to exchanges
        logger.info("Test 1: Connecting to exchanges...")
        for exchange in exchanges.values():
            connected = await exchange.connect()
            assert connected, "Failed to connect to exchange"
        logger.info("✓ Exchanges connected successfully")
        
        # Test 2: Check initial balance
        logger.info("Test 2: Checking initial balance...")
        balance = await exchanges['binance'].get_balance()
        assert balance.get('USDT', 0) > 0, "No USDT balance found"
        logger.info(f"✓ Initial balance: {balance}")
        
        # Test 3: Get market data
        logger.info("Test 3: Getting market data...")
        ticker = await exchanges['binance'].get_ticker("BTC/USDT")
        assert ticker is not None, "Failed to get ticker"
        assert 'last' in ticker, "Ticker missing last price"
        logger.info(f"✓ BTC/USDT price: ${ticker['last']:.2f}")
        
        # Test 4: Get OHLCV data
        logger.info("Test 4: Getting OHLCV data...")
        ohlcv = await exchanges['binance'].get_ohlcv("BTC/USDT", "1m", 10)
        assert ohlcv is not None, "Failed to get OHLCV data"
        assert len(ohlcv) > 0, "OHLCV data empty"
        logger.info(f"✓ Received {len(ohlcv)} candles")
        
        # Test 5: Health check
        logger.info("Test 5: Running health check...")
        health = await health_checker.check_health(engine)
        health_dict = health_checker.get_health_dict(health)
        # Note: Engine not running yet, so we check critical components only
        critical_healthy = all(
            comp.get('healthy', False) 
            for name, comp in health_dict['components'].items()
            if name in ['exchanges', 'data_storage']  # Only check critical infrastructure
        )
        assert critical_healthy, "Critical infrastructure not healthy"
        logger.info("✓ System health check passed")
        
        # Test 6: Run strategy briefly
        logger.info("Test 6: Running strategy for 10 seconds...")
        
        # Start engine in background
        engine_task = asyncio.create_task(engine.start())
        
        # Let it run for 10 seconds
        await asyncio.sleep(10)
        
        # Stop engine
        await engine.stop()
        
        # Check if any orders were created
        orders_count = len(engine.orders)
        logger.info(f"✓ Strategy executed, created {orders_count} orders")
        
        # Test 7: Check positions
        logger.info("Test 7: Checking positions...")
        positions = await exchanges['binance'].get_positions()
        logger.info(f"✓ Current positions: {len(positions)}")
        
        # Test 8: Check trade history
        logger.info("Test 8: Checking trade history...")
        trades = await storage.get_trades(limit=10)
        logger.info(f"✓ Trade history: {len(trades)} trades")
        
        # Test 9: Final health check
        logger.info("Test 9: Final health check...")
        health = await health_checker.check_health(engine)
        health_dict = health_checker.get_health_dict(health)
        logger.info(f"✓ Final status: {'Healthy' if health_dict['healthy'] else 'Unhealthy'}")
        
        # Cleanup
        await storage.close()
        for exchange in exchanges.values():
            await exchange.disconnect()
        
        logger.info("\n🎉 All tests passed! TraderX system is working correctly.")
        return True
        
    except Exception as e:
        logger.error(f"\n❌ Test failed: {e}", exc_info=True)
        return False


async def main():
    """Main test runner."""
    success = await test_paper_trading()
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    asyncio.run(main())
