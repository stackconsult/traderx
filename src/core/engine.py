import asyncio
import logging
from typing import Dict, List, Optional, Any
from datetime import datetime

from .models import Order, OrderStatus, OrderType, OrderSide, Position
from ..strategies.base import Signal, SignalType

from ..exchanges.base import BaseExchange
from ..strategies.base import BaseStrategy
from ..risk.manager import RiskManager
from ..data.storage import DataStorage


class TradingEngine:
    """
    Core trading engine that orchestrates strategy execution, order management,
    and risk controls.
    """
    
    def __init__(
        self,
        exchanges: Dict[str, BaseExchange],
        strategies: List[BaseStrategy],
        risk_manager: RiskManager,
        data_storage: DataStorage
    ):
        self.exchanges = exchanges
        self.strategies = strategies
        self.risk_manager = risk_manager
        self.data_storage = data_storage
        self.logger = logging.getLogger(__name__)
        
        # Engine state
        self.running = False
        self.positions: Dict[str, Position] = {}
        self.orders: Dict[str, Order] = {}
        self.order_queue = asyncio.Queue()
        
    async def start(self):
        """Start the trading engine."""
        self.running = True
        self.logger.info("Starting trading engine...")
        
        # Start tasks
        tasks = [
            asyncio.create_task(self._process_orders()),
            asyncio.create_task(self._monitor_positions()),
            asyncio.create_task(self._run_strategies())
        ]
        
        # Start exchange connections
        for exchange in self.exchanges.values():
            await exchange.connect()
        
        try:
            await asyncio.gather(*tasks)
        except Exception as e:
            self.logger.error(f"Engine error: {e}")
            await self.stop()
    
    async def stop(self):
        """Stop the trading engine."""
        self.running = False
        self.logger.info("Stopping trading engine...")
        
        # Cancel all pending orders
        for order in self.orders.values():
            if order.status in [OrderStatus.PENDING, OrderStatus.SUBMITTED]:
                await self._cancel_order(order)
        
        # Disconnect exchanges
        for exchange in self.exchanges.values():
            await exchange.disconnect()
    
    async def submit_order(self, order: Order) -> bool:
        """Submit an order to the exchange."""
        # Risk check
        if not await self.risk_manager.validate_order(order, self.positions):
            self.logger.warning(f"Order {order.id} rejected by risk manager")
            order.status = OrderStatus.REJECTED
            return False
        
        # Submit to exchange
        exchange = self.exchanges.get(order.symbol.split('/')[0])  # Simplified
        if not exchange:
            self.logger.error(f"No exchange found for symbol {order.symbol}")
            return False
        
        try:
            await exchange.submit_order(order)
            order.status = OrderStatus.SUBMITTED
            self.orders[order.id] = order
            await self.order_queue.put(order)
            return True
        except Exception as e:
            self.logger.error(f"Failed to submit order {order.id}: {e}")
            order.status = OrderStatus.REJECTED
            return False
    
    async def _process_orders(self):
        """Process order updates from exchanges."""
        while self.running:
            try:
                order = await asyncio.wait_for(self.order_queue.get(), timeout=1.0)
                
                # Update order status based on exchange feedback
                exchange = self.exchanges.get(order.symbol.split('/')[0])
                if exchange:
                    updated_order = await exchange.get_order_status(order.id)
                    if updated_order:
                        self.orders[order.id] = updated_order
                        
                        # Handle fills
                        if updated_order.status == OrderStatus.FILLED:
                            await self._handle_fill(updated_order)
                
            except asyncio.TimeoutError:
                continue
            except Exception as e:
                self.logger.error(f"Error processing order: {e}")
    
    async def _handle_fill(self, order: Order):
        """Handle order fill and update positions."""
        self.logger.info(f"Order {order.id} filled: {order.filled_quantity}@{order.filled_price}")
        
        # Update position
        position = self.positions.get(order.symbol)
        if position is None:
            position = Position(
                symbol=order.symbol,
                quantity=0.0,
                entry_price=0.0,
                current_price=order.filled_price
            )
            self.positions[order.symbol] = position
        
        # Calculate new position
        if order.side == OrderSide.BUY:
            new_quantity = position.quantity + order.filled_quantity
            if position.quantity > 0:
                # Average up the entry price
                total_cost = (position.quantity * position.entry_price + 
                            order.filled_quantity * order.filled_price)
                position.entry_price = total_cost / new_quantity
            else:
                position.entry_price = order.filled_price
            position.quantity = new_quantity
        else:
            position.quantity -= order.filled_quantity
            # Calculate realized PnL
            if position.quantity < 0:
                position.realized_pnl += (order.filled_price - position.entry_price) * order.filled_quantity
                position.quantity = 0
        
        # Store to database
        await self.data_storage.save_order(order)
        await self.data_storage.save_position(position)
    
    async def _monitor_positions(self):
        """Monitor and update position values."""
        while self.running:
            try:
                for symbol, position in self.positions.items():
                    if position.quantity != 0:
                        exchange = self.exchanges.get(symbol.split('/')[0])
                        if exchange:
                            ticker = await exchange.get_ticker(symbol)
                            if ticker:
                                position.current_price = ticker['last']
                                position.unrealized_pnl = (
                                    (position.current_price - position.entry_price) * 
                                    position.quantity
                                )
                
                await asyncio.sleep(1.0)
            except Exception as e:
                self.logger.error(f"Error monitoring positions: {e}")
    
    async def _run_strategies(self):
        """Run all trading strategies."""
        while self.running:
            try:
                for strategy in self.strategies:
                    # Get market data
                    for symbol in strategy.symbols:
                        exchange = self.exchanges.get(symbol.split('/')[0])
                        if exchange:
                            data = await exchange.get_ohlcv(symbol, '1m', limit=100)
                            if data:
                                # Generate signals
                                signals = await strategy.generate_signals(symbol, data)
                                
                                # Execute signals
                                for signal in signals:
                                    order = self._create_order_from_signal(signal)
                                    if order:
                                        await self.submit_order(order)
                
                await asyncio.sleep(1.0)
            except Exception as e:
                self.logger.error(f"Error running strategies: {e}")
    
    def _create_order_from_signal(self, signal: Signal) -> Optional[Order]:
        """Create an order from strategy signal."""
        try:
            # Extract signal data from Signal dataclass
            symbol = signal.symbol
            signal_type = signal.type
            quantity = signal.quantity
            price = signal.price
            stop_loss = signal.stop_loss
            take_profit = signal.take_profit
            
            if not all([symbol, signal_type, quantity]):
                self.logger.error(f"Invalid signal: missing required fields")
                return None
            
            # Determine order side and type
            if signal_type == SignalType.BUY:
                side = OrderSide.BUY
                order_type = OrderType.MARKET if price is None else OrderType.LIMIT
            elif signal_type == SignalType.SELL:
                side = OrderSide.SELL
                order_type = OrderType.MARKET if price is None else OrderType.LIMIT
            elif signal_type == SignalType.CLOSE:
                # Close existing position
                position = self.positions.get(symbol)
                if position is None or position.quantity == 0:
                    return None
                
                side = OrderSide.SELL if position.quantity > 0 else OrderSide.BUY
                quantity = abs(position.quantity)
                order_type = OrderType.MARKET
            else:
                self.logger.error(f"Unknown signal type: {signal_type}")
                return None
            
            # Generate unique order ID
            order_id = f"{symbol}_{side.value}_{int(datetime.utcnow().timestamp() * 1000)}"
            
            # Create order
            order = Order(
                id=order_id,
                symbol=symbol,
                side=side,
                type=order_type,
                quantity=quantity,
                price=price if order_type == OrderType.LIMIT else None
            )
            
            # Store stop loss and take profit for later
            if stop_loss or take_profit:
                order.metadata = {
                    'stop_loss': stop_loss,
                    'take_profit': take_profit,
                    'signal_strength': signal.strength.value
                }
            
            return order
            
        except Exception as e:
            self.logger.error(f"Error creating order from signal: {e}")
            return None
    
    async def _cancel_order(self, order: Order):
        """Cancel an order."""
        exchange = self.exchanges.get(order.symbol.split('/')[0])
        if exchange:
            try:
                await exchange.cancel_order(order.id)
                order.status = OrderStatus.CANCELLED
            except Exception as e:
                self.logger.error(f"Failed to cancel order {order.id}: {e}")
