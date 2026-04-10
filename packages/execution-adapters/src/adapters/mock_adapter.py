"""
Mock Adapter for Testing
Simulates venue behavior with configurable latency and success rates.
"""

import asyncio
import random
import time
from typing import List, Optional, Dict, Any, AsyncGenerator
from datetime import datetime

from ..ports.liquidity_port import LiquidityPort
from ..domain.models import (
    UnifiedOrder, ExecutionResult, Position, AccountInfo,
    VenueInfo, MarketData, OrderStatus, OrderType, TimeInForce, OrderSide
)


class MockAdapter(LiquidityPort):
    """Mock adapter for testing."""
    
    def __init__(self, venue_name: str, config: Dict[str, Any]):
        super().__init__(venue_name, config)
        self.latency_ms = config.get("latency_ms", 5.0)
        self.success_rate = config.get("success_rate", 0.95)
        self.supported_symbols = config.get("symbols", ["BTCUSDT", "ETHUSDT"])
        self.order_book: Dict[str, Dict[str, float]] = {}
        
        # Initialize order book
        for symbol in self.supported_symbols:
            base_price = 50000.0 if "BTC" in symbol else 3000.0
            self.order_book[symbol] = {
                "bid": base_price * (1 - random.uniform(0, 0.001)),
                "ask": base_price * (1 + random.uniform(0, 0.001)),
                "last": base_price
            }
            
    async def connect(self) -> bool:
        """Simulate connection."""
        await asyncio.sleep(self.latency_ms / 1000)
        self.is_connected = random.random() > 0.01  # 99% success rate
        return self.is_connected
        
    async def disconnect(self) -> None:
        """Simulate disconnection."""
        self.is_connected = False
        
    async def submit_order(self, order: UnifiedOrder) -> ExecutionResult:
        """Simulate order submission."""
        if not self.is_connected:
            return ExecutionResult(
                order_id=order.id,
                venue_order_id=None,
                status=OrderStatus.REJECTED,
                error_message="Not connected"
            )
            
        # Simulate network latency
        await asyncio.sleep(self.latency_ms / 1000)
        
        # Simulate success/failure
        if random.random() > self.success_rate:
            return ExecutionResult(
                order_id=order.id,
                venue_order_id=f"mock_{order.id}",
                status=OrderStatus.REJECTED,
                error_message="Mock rejection"
            )
            
        # Simulate execution
        if order.order_type == OrderType.MARKET:
            # Market order - fill at current price
            order_book = self.order_book.get(order.symbol, {})
            if order.side == OrderSide.BUY:
                fill_price = order_book.get("ask", 50000.0)
            else:
                fill_price = order_book.get("bid", 50000.0)
        else:
            # Limit order - use specified price
            fill_price = order.price
            
        # Random slippage
        slippage = random.uniform(-0.001, 0.001)
        fill_price *= (1 + slippage)
        
        return ExecutionResult(
            order_id=order.id,
            venue_order_id=f"mock_{order.id}",
            status=OrderStatus.FILLED,
            filled_quantity=order.quantity,
            remaining_quantity=0.0,
            average_price=fill_price,
            execution_venue=self.venue_name,
            fees={"commission": fill_price * order.quantity * 0.001}
        )
        
    async def cancel_order(self, order_id: str, venue_order_id: Optional[str] = None) -> bool:
        """Simulate order cancellation."""
        await asyncio.sleep(self.latency_ms / 1000)
        return random.random() > 0.05  # 95% success rate
        
    async def get_order_status(self, order_id: str) -> Optional[ExecutionResult]:
        """Simulate order status check."""
        await asyncio.sleep(self.latency_ms / 1000)
        return None
        
    async def get_positions(self) -> List[Position]:
        """Simulate position retrieval."""
        await asyncio.sleep(self.latency_ms / 1000)
        return []
        
    async def get_account_info(self) -> AccountInfo:
        """Simulate account info retrieval."""
        await asyncio.sleep(self.latency_ms / 1000)
        return AccountInfo(
            account_id="mock-account",
            venue=self.venue_name,
            balance={"USDT": 100000.0, "BTC": 2.0},
            positions=[]
        )
        
    async def get_venue_info(self) -> VenueInfo:
        """Get venue information."""
        if not self.venue_info:
            self.venue_info = VenueInfo(
                name=self.venue_name,
                venue_type="Mock",
                supported_symbols=self.supported_symbols,
                supported_order_types=[OrderType.MARKET, OrderType.LIMIT],
                supported_tif=[TimeInForce.IOC, TimeInForce.GTC, TimeInForce.FOK],
                min_order_size=0.001,
                max_order_size=1000.0,
                tick_size=0.01,
                connection_status=self.is_connected,
                latency_ms=self.latency_ms
            )
        return self.venue_info
        
    async def get_market_data(self, symbols: List[str]) -> AsyncGenerator[MarketData, None]:
        """Simulate market data stream."""
        while self.is_connected:
            for symbol in symbols:
                if symbol in self.order_book:
                    # Random price movement
                    movement = random.uniform(-0.0001, 0.0001)
                    self.order_book[symbol]["bid"] *= (1 + movement)
                    self.order_book[symbol]["ask"] *= (1 + movement)
                    self.order_book[symbol]["last"] = (
                        self.order_book[symbol]["bid"] + self.order_book[symbol]["ask"]
                    ) / 2
                    
                    yield MarketData(
                        symbol=symbol,
                        venue=self.venue_name,
                        bid=self.order_book[symbol]["bid"],
                        ask=self.order_book[symbol]["ask"],
                        last=self.order_book[symbol]["last"],
                        timestamp=datetime.utcnow()
                    )
                    
            await asyncio.sleep(0.1)  # 10 Hz update rate
            
    async def _measure_latency(self) -> float:
        """Return configured latency."""
        return self.latency_ms
