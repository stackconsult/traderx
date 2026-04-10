import asyncio
import logging
import random
from typing import Dict, List, Optional, Any
from datetime import datetime, timedelta
import uuid

from .base import BaseExchange
from ..core.engine import Order, OrderStatus, OrderType, OrderSide


class PaperExchange(BaseExchange):
    """
    Paper trading exchange that simulates order execution without real money.
    Uses realistic market simulation with slippage and latency.
    """
    
    def __init__(self, config: Dict[str, Any]):
        super().__init__("paper", config)
        
        # Simulation parameters
        self.latency_ms = config.get('latency_ms', 100)  # Simulated network latency
        self.slippage_bps = config.get('slippage_bps', 5)  # 0.05% slippage
        self.fill_probability = config.get('fill_probability', 0.95)  # 95% fill rate
        self.min_fill_size = config.get('min_fill_size', 0.001)
        
        # Exchange state
        self.balances = {'USDT': 10000.0}  # Starting balance
        self.positions = {}
        self.orders = {}
        self.order_books = {}  # Simulated order books
        self.trade_history = []
        
        # Market data simulation
        self.market_prices = {}
        self.price_volatility = config.get('price_volatility', 0.002)  # 0.2% volatility
        
        self.logger.info("PaperExchange initialized with simulated balance")
    
    async def connect(self):
        """Connect to paper trading exchange."""
        await asyncio.sleep(self.latency_ms / 1000)  # Simulate connection latency
        self.connected = True
        self.logger.info("Connected to PaperExchange")
        return True
    
    async def disconnect(self):
        """Disconnect from paper trading exchange."""
        self.connected = False
        self.logger.info("Disconnected from PaperExchange")
    
    async def submit_order(self, order: Order) -> bool:
        """Submit an order to paper exchange."""
        if not self.connected:
            raise Exception("Not connected to exchange")
        
        # Simulate order submission latency
        await asyncio.sleep(self.latency_ms / 1000)
        
        # Store order
        self.orders[order.id] = order
        
        # Simulate order execution
        asyncio.create_task(self._simulate_order_fill(order))
        
        self.logger.info(f"Order submitted: {order.id}")
        return True
    
    async def cancel_order(self, order_id: str) -> bool:
        """Cancel an order on paper exchange."""
        if not self.connected:
            raise Exception("Not connected to exchange")
        
        await asyncio.sleep(self.latency_ms / 1000)
        
        order = self.orders.get(order_id)
        if order and order.status in [OrderStatus.PENDING, OrderStatus.SUBMITTED]:
            order.status = OrderStatus.CANCELLED
            self.logger.info(f"Order cancelled: {order_id}")
            return True
        
        return False
    
    async def get_order_status(self, order_id: str) -> Optional[Order]:
        """Get order status from paper exchange."""
        return self.orders.get(order_id)
    
    async def get_ticker(self, symbol: str) -> Optional[Dict[str, float]]:
        """Get simulated ticker data."""
        if not self.connected:
            return None
        
        # Generate or update market price
        if symbol not in self.market_prices:
            self.market_prices[symbol] = self._generate_initial_price(symbol)
        else:
            # Simulate price movement
            self.market_prices[symbol] = self._simulate_price_movement(
                self.market_prices[symbol]
            )
        
        price = self.market_prices[symbol]
        
        return {
            'last': price,
            'bid': price * (1 - 0.0001),  # 0.01% spread
            'ask': price * (1 + 0.0001),
            'high': price * 1.01,
            'low': price * 0.99,
            'volume': random.uniform(100, 1000),
            'timestamp': datetime.utcnow().timestamp() * 1000
        }
    
    async def get_ohlcv(
        self, 
        symbol: str, 
        timeframe: str = '1m', 
        limit: int = 100
    ) -> Optional[List[List[float]]]:
        """Get simulated OHLCV data."""
        if not self.connected:
            return None
        
        # Generate realistic OHLCV data
        ohlcv = []
        current_time = datetime.utcnow()
        base_price = self.market_prices.get(symbol, 50000)
        
        for i in range(limit):
            timestamp = int((current_time - timedelta(minutes=i)).timestamp() * 1000)
            
            # Simulate price movement
            volatility = 0.001  # 0.1% per minute
            change = random.gauss(0, volatility)
            price = base_price * (1 + change * i)
            
            # Generate OHLC
            high = price * (1 + random.uniform(0, 0.001))
            low = price * (1 - random.uniform(0, 0.001))
            close = price
            open_price = price * (1 + random.gauss(0, 0.0005))
            volume = random.uniform(100, 1000)
            
            ohlcv.append([timestamp, open_price, high, low, close, volume])
        
        return list(reversed(ohlcv))
    
    async def get_balance(self) -> Dict[str, float]:
        """Get account balance from paper exchange."""
        return self.balances.copy()
    
    async def get_positions(self) -> List[Dict[str, Any]]:
        """Get open positions from paper exchange."""
        positions = []
        for symbol, quantity in self.positions.items():
            if abs(quantity) > self.min_fill_size:
                positions.append({
                    'symbol': symbol,
                    'size': quantity,
                    'side': 'long' if quantity > 0 else 'short'
                })
        return positions
    
    async def _simulate_order_fill(self, order: Order):
        """Simulate order execution with realistic behavior."""
        # Wait for realistic fill time
        fill_delay = random.uniform(0.1, 2.0)  # 100ms to 2s
        await asyncio.sleep(fill_delay)
        
        # Check if order should be filled
        if random.random() > self.fill_probability:
            order.status = OrderStatus.REJECTED
            return
        
        # Get current market price
        ticker = await self.get_ticker(order.symbol)
        if not ticker:
            order.status = OrderStatus.REJECTED
            return
        
        market_price = ticker['last']
        
        # Apply slippage
        if order.type == OrderType.MARKET:
            slippage = random.uniform(-self.slippage_bps/10000, self.slippage_bps/10000)
            fill_price = market_price * (1 + slippage)
        else:
            # Limit orders fill at limit price or better
            if order.side == OrderSide.BUY and order.price >= market_price:
                fill_price = min(order.price, market_price)
            elif order.side == OrderSide.SELL and order.price <= market_price:
                fill_price = max(order.price, market_price)
            else:
                # Limit order not filled
                return
        
        # Update order
        order.status = OrderStatus.FILLED
        order.filled_quantity = order.quantity
        order.filled_price = fill_price
        
        # Update balance and positions
        await self._update_balance_and_position(order)
        
        # Record trade
        self.trade_history.append({
            'order_id': order.id,
            'symbol': order.symbol,
            'side': order.side.value,
            'quantity': order.quantity,
            'price': fill_price,
            'timestamp': datetime.utcnow()
        })
        
        self.logger.info(f"Order filled: {order.id} at {fill_price}")
    
    async def _update_balance_and_position(self, order: Order):
        """Update account balance and positions after order fill."""
        symbol = order.symbol
        base_asset = symbol.split('/')[0]
        quote_asset = symbol.split('/')[1]
        
        # Initialize balances if needed
        if base_asset not in self.balances:
            self.balances[base_asset] = 0.0
        if quote_asset not in self.balances:
            self.balances[quote_asset] = 0.0
        
        # Update balances
        if order.side == OrderSide.BUY:
            cost = order.filled_quantity * order.filled_price
            self.balances[quote_asset] -= cost
            self.balances[base_asset] += order.filled_quantity
            
            # Update position
            self.positions[symbol] = self.positions.get(symbol, 0) + order.filled_quantity
            
        else:  # SELL
            proceeds = order.filled_quantity * order.filled_price
            self.balances[quote_asset] += proceeds
            self.balances[base_asset] -= order.filled_quantity
            
            # Update position
            self.positions[symbol] = self.positions.get(symbol, 0) - order.filled_quantity
        
        # Remove zero positions
        if abs(self.positions.get(symbol, 0)) < self.min_fill_size:
            del self.positions[symbol]
    
    def _generate_initial_price(self, symbol: str) -> float:
        """Generate initial price for a symbol."""
        # Base prices for common pairs
        base_prices = {
            'BTC/USDT': 45000,
            'ETH/USDT': 3000,
            'BNB/USDT': 300,
            'ADA/USDT': 1.2,
            'DOT/USDT': 25,
            'LINK/USDT': 20,
            'UNI/USDT': 15,
            'LTC/USDT': 150
        }
        
        return base_prices.get(symbol, random.uniform(10, 1000))
    
    def _simulate_price_movement(self, current_price: float) -> float:
        """Simulate realistic price movement."""
        # Random walk with volatility
        change = random.gauss(0, self.price_volatility)
        new_price = current_price * (1 + change)
        
        # Add some momentum
        if random.random() < 0.1:  # 10% chance of trend
            trend = random.choice([-1, 1]) * 0.005  # 0.5% trend
            new_price *= (1 + trend)
        
        return max(new_price, current_price * 0.9)  # Limit single move to 10%
