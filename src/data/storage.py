import asyncio
import logging
from typing import Dict, List, Optional, Any
from datetime import datetime
from dataclasses import asdict
import json

from sqlalchemy.ext.asyncio import create_async_engine, AsyncSession, async_sessionmaker
from sqlalchemy.orm import declarative_base
from sqlalchemy import Column, Integer, String, Float, DateTime, Text, Boolean

from ..core.models import Order, Position, OrderStatus, OrderType, OrderSide


Base = declarative_base()


class OrderModel(Base):
    __tablename__ = 'orders'
    
    id = Column(String, primary_key=True)
    symbol = Column(String, nullable=False)
    side = Column(String, nullable=False)
    type = Column(String, nullable=False)
    quantity = Column(Float, nullable=False)
    price = Column(Float, nullable=True)
    status = Column(String, nullable=False)
    timestamp = Column(DateTime, nullable=False)
    filled_quantity = Column(Float, default=0.0)
    filled_price = Column(Float, nullable=True)


class PositionModel(Base):
    __tablename__ = 'positions'
    
    symbol = Column(String, primary_key=True)
    quantity = Column(Float, nullable=False)
    entry_price = Column(Float, nullable=False)
    current_price = Column(Float, nullable=False)
    unrealized_pnl = Column(Float, default=0.0)
    realized_pnl = Column(Float, default=0.0)
    updated_at = Column(DateTime, nullable=False)


class TradeModel(Base):
    __tablename__ = 'trades'
    
    id = Column(Integer, primary_key=True, autoincrement=True)
    order_id = Column(String, nullable=False)
    symbol = Column(String, nullable=False)
    side = Column(String, nullable=False)
    quantity = Column(Float, nullable=False)
    price = Column(Float, nullable=False)
    fee = Column(Float, nullable=True)
    timestamp = Column(DateTime, nullable=False)


class DataStorage:
    """
    Data storage layer for persisting trading data.
    Supports both SQL database and Redis for caching.
    """
    
    def __init__(self, database_url: str, redis_url: Optional[str] = None):
        self.logger = logging.getLogger(__name__)
        self.database_url = database_url
        self.redis_url = redis_url
        
        # Initialize database engine
        self.engine = create_async_engine(database_url, echo=False)
        self.SessionLocal = async_sessionmaker(
            self.engine, 
            class_=AsyncSession, 
            expire_on_commit=False
        )
        
        # Initialize Redis if available
        self.redis = None
        if redis_url:
            try:
                import redis.asyncio as redis
                self.redis = redis.from_url(redis_url)
            except ImportError:
                self.logger.warning("Redis not available, caching disabled")
    
    async def initialize(self):
        """Initialize database tables."""
        async with self.engine.begin() as conn:
            await conn.run_sync(Base.metadata.create_all)
    
    async def save_order(self, order: Order):
        """Save order to database."""
        async with self.SessionLocal() as session:
            order_model = OrderModel(
                id=order.id,
                symbol=order.symbol,
                side=order.side.value,
                type=order.type.value,
                quantity=order.quantity,
                price=order.price,
                status=order.status.value,
                timestamp=order.timestamp,
                filled_quantity=order.filled_quantity,
                filled_price=order.filled_price
            )
            
            session.add(order_model)
            await session.commit()
            
            # Cache in Redis
            if self.redis:
                await self.redis.setex(
                    f"order:{order.id}",
                    3600,  # 1 hour TTL
                    json.dumps(asdict(order), default=str)
                )
    
    async def get_order(self, order_id: str) -> Optional[Order]:
        """Get order by ID."""
        # Try cache first
        if self.redis:
            cached = await self.redis.get(f"order:{order_id}")
            if cached:
                data = json.loads(cached)
                return Order(
                    id=data['id'],
                    symbol=data['symbol'],
                    side=OrderSide(data['side']),
                    type=OrderType(data['type']),
                    quantity=data['quantity'],
                    price=data['price'],
                    status=OrderStatus(data['status']),
                    timestamp=datetime.fromisoformat(data['timestamp']),
                    filled_quantity=data['filled_quantity'],
                    filled_price=data['filled_price']
                )
        
        # Query database
        async with self.SessionLocal() as session:
            from sqlalchemy import select
            result = await session.execute(
                select(OrderModel).where(OrderModel.id == order_id)
            )
            order_model = result.scalar_one_or_none()
            
            if order_model:
                return Order(
                    id=order_model.id,
                    symbol=order_model.symbol,
                    side=OrderSide(order_model.side),
                    type=OrderType(order_model.type),
                    quantity=order_model.quantity,
                    price=order_model.price,
                    status=OrderStatus(order_model.status),
                    timestamp=order_model.timestamp,
                    filled_quantity=order_model.filled_quantity,
                    filled_price=order_model.filled_price
                )
        
        return None
    
    async def save_position(self, position: Position):
        """Save position to database."""
        async with self.SessionLocal() as session:
            position_model = PositionModel(
                symbol=position.symbol,
                quantity=position.quantity,
                entry_price=position.entry_price,
                current_price=position.current_price,
                unrealized_pnl=position.unrealized_pnl,
                realized_pnl=position.realized_pnl,
                updated_at=datetime.utcnow()
            )
            
            await session.merge(position_model)
            await session.commit()
            
            # Cache in Redis
            if self.redis:
                await self.redis.setex(
                    f"position:{position.symbol}",
                    300,  # 5 minutes TTL
                    json.dumps(asdict(position), default=str)
                )
    
    async def get_positions(self) -> Dict[str, Position]:
        """Get all positions."""
        # Try cache first
        if self.redis:
            keys = await self.redis.keys("position:*")
            if keys:
                positions = {}
                for key in keys:
                    cached = await self.redis.get(key)
                    if cached:
                        data = json.loads(cached)
                        symbol = key.decode().split(':')[1]
                        positions[symbol] = Position(
                            symbol=data['symbol'],
                            quantity=data['quantity'],
                            entry_price=data['entry_price'],
                            current_price=data['current_price'],
                            unrealized_pnl=data['unrealized_pnl'],
                            realized_pnl=data['realized_pnl']
                        )
                return positions
        
        # Query database
        async with self.SessionLocal() as session:
            from sqlalchemy import select
            result = await session.execute(select(PositionModel))
            positions = {}
            
            for position_model in result.scalars():
                positions[position_model.symbol] = Position(
                    symbol=position_model.symbol,
                    quantity=position_model.quantity,
                    entry_price=position_model.entry_price,
                    current_price=position_model.current_price,
                    unrealized_pnl=position_model.unrealized_pnl,
                    realized_pnl=position_model.realized_pnl
                )
            
            return positions
    
    async def save_trade(self, order_id: str, symbol: str, side: str, 
                        quantity: float, price: float, fee: Optional[float] = None):
        """Save executed trade to database."""
        async with self.SessionLocal() as session:
            trade = TradeModel(
                order_id=order_id,
                symbol=symbol,
                side=side,
                quantity=quantity,
                price=price,
                fee=fee,
                timestamp=datetime.utcnow()
            )
            
            session.add(trade)
            await session.commit()
    
    async def get_trades(self, symbol: Optional[str] = None, 
                        limit: int = 100) -> List[Dict[str, Any]]:
        """Get trades history."""
        async with self.SessionLocal() as session:
            from sqlalchemy import select, desc
            
            query = select(TradeModel)
            if symbol:
                query = query.where(TradeModel.symbol == symbol)
            
            query = query.order_by(desc(TradeModel.timestamp)).limit(limit)
            result = await session.execute(query)
            
            trades = []
            for trade in result.scalars():
                trades.append({
                    'id': trade.id,
                    'order_id': trade.order_id,
                    'symbol': trade.symbol,
                    'side': trade.side,
                    'quantity': trade.quantity,
                    'price': trade.price,
                    'fee': trade.fee,
                    'timestamp': trade.timestamp
                })
            
            return trades
    
    async def get_performance_metrics(self, symbol: Optional[str] = None) -> Dict[str, Any]:
        """Calculate performance metrics from trades."""
        trades = await self.get_trades(symbol, limit=1000)
        
        if not trades:
            return {}
        
        # Basic metrics
        total_trades = len(trades)
        winning_trades = sum(1 for t in trades if t['side'] == 'sell')  # Simplified
        
        # Calculate PnL (simplified)
        total_pnl = 0.0
        buy_prices = {}
        
        for trade in trades:
            if trade['side'] == 'buy':
                buy_prices[trade['symbol']] = trade['price']
            elif trade['side'] == 'sell' and trade['symbol'] in buy_prices:
                pnl = (trade['price'] - buy_prices[trade['symbol']]) * trade['quantity']
                total_pnl += pnl
        
        return {
            'total_trades': total_trades,
            'winning_trades': winning_trades,
            'win_rate': winning_trades / total_trades if total_trades > 0 else 0,
            'total_pnl': total_pnl,
            'avg_trade_size': sum(t['quantity'] for t in trades) / total_trades if total_trades > 0 else 0
        }
    
    async def close(self):
        """Close database connections."""
        await self.engine.dispose()
        if self.redis:
            await self.redis.close()
