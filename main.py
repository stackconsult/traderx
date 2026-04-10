#!/usr/bin/env python3
"""
TraderX - Automated Trading System
Main entry point for the trading application.
"""

import asyncio
import logging
import signal
import sys
from pathlib import Path
from typing import Dict, List

# Add src to path
sys.path.insert(0, str(Path(__file__).parent))

from config.settings import settings, Environment
from src.core.engine import TradingEngine
from src.exchanges.binance import BinanceExchange
from src.strategies.moving_average import MovingAverageStrategy
from src.risk.manager import RiskManager
from src.data.storage import DataStorage


# Configure logging
logging.basicConfig(
    level=getattr(logging, settings.log_level),
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('traderx.log'),
        logging.StreamHandler()
    ]
)

logger = logging.getLogger(__name__)


class TraderXApp:
    """Main application class for TraderX."""
    
    def __init__(self):
        self.engine: TradingEngine = None
        self.running = False
        
    async def initialize(self):
        """Initialize the trading system."""
        logger.info("Initializing TraderX...")
        
        # Validate environment
        if settings.environment == Environment.LIVE:
            logger.warning("RUNNING IN LIVE TRADING MODE!")
            response = input("Type 'LIVE' to confirm: ")
            if response != "LIVE":
                logger.info("Aborting live trading")
                sys.exit(1)
        
        # Initialize data storage
        storage = DataStorage(settings.database_url, settings.redis_url)
        await storage.initialize()
        
        # Initialize risk manager
        risk_manager = RiskManager(
            max_position_size=settings.max_position_size,
            max_daily_loss=settings.max_daily_loss,
            max_drawdown=settings.max_drawdown
        )
        
        # Initialize exchanges
        exchanges = {}
        if settings.default_exchange == "binance":
            binance_config = settings.exchange_configs.get("binance", {})
            exchanges["binance"] = BinanceExchange(binance_config)
        
        # Initialize strategies
        strategies = []
        if settings.environment in [Environment.DEVELOPMENT, Environment.PAPER_TRADING]:
            # Add test strategy for paper trading
            ma_strategy = MovingAverageStrategy(
                name="MA_Cross",
                symbols=["BTC/USDT", "ETH/USDT"],
                config={
                    "fast_period": 10,
                    "slow_period": 30,
                    "timeframe": "1m"
                }
            )
            strategies.append(ma_strategy)
        
        # Initialize trading engine
        self.engine = TradingEngine(
            exchanges=exchanges,
            strategies=strategies,
            risk_manager=risk_manager,
            data_storage=storage
        )
        
        logger.info("TraderX initialized successfully")
    
    async def start(self):
        """Start the trading application."""
        if not self.engine:
            await self.initialize()
        
        self.running = True
        logger.info(f"Starting TraderX in {settings.environment.value} mode...")
        
        # Setup signal handlers
        signal.signal(signal.SIGINT, self._signal_handler)
        signal.signal(signal.SIGTERM, self._signal_handler)
        
        try:
            await self.engine.start()
        except KeyboardInterrupt:
            logger.info("Received interrupt signal")
        except Exception as e:
            logger.error(f"Fatal error: {e}", exc_info=True)
        finally:
            await self.stop()
    
    async def stop(self):
        """Stop the trading application."""
        if not self.running:
            return
        
        self.running = False
        logger.info("Stopping TraderX...")
        
        if self.engine:
            await self.engine.stop()
        
        logger.info("TraderX stopped")
    
    def _signal_handler(self, signum, frame):
        """Handle system signals."""
        logger.info(f"Received signal {signum}")
        asyncio.create_task(self.stop())


async def main():
    """Main entry point."""
    app = TraderXApp()
    await app.start()


if __name__ == "__main__":
    asyncio.run(main())
