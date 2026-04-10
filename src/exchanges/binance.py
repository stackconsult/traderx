import asyncio
import logging
from typing import Dict, List, Optional, Any
import os
from datetime import datetime

import ccxt.async_support as ccxt

from .base import BaseExchange
from ..core.engine import Order, OrderStatus, OrderType, OrderSide


class BinanceExchange(BaseExchange):
    """Binance exchange implementation using CCXT library."""
    
    def __init__(self, config: Dict[str, Any]):
        super().__init__("binance", config)
        
        # Initialize CCXT exchange
        api_key = config.get('api_key') or os.getenv('BINANCE_API_KEY')
        secret_key = config.get('secret_key') or os.getenv('BINANCE_SECRET_KEY')
        sandbox = config.get('sandbox', True)
        
        self.exchange = ccxt.binance({
            'apiKey': api_key,
            'secret': secret_key,
            'sandbox': sandbox,
            'enableRateLimit': True,
            'options': {
                'defaultType': 'spot',
            }
        })
        
        self.logger.info(f"BinanceExchange initialized (sandbox={sandbox})")
    
    async def connect(self):
        """Connect to Binance exchange."""
        try:
            # Test connection
            await self.exchange.load_markets()
            balance = await self.exchange.fetch_balance()
            self.connected = True
            self.logger.info("Connected to Binance exchange")
            return True
        except Exception as e:
            self.logger.error(f"Failed to connect to Binance: {e}")
            return False
    
    async def disconnect(self):
        """Disconnect from Binance exchange."""
        await self.exchange.close()
        self.connected = False
        self.logger.info("Disconnected from Binance exchange")
    
    async def submit_order(self, order: Order) -> bool:
        """Submit an order to Binance."""
        if not self.connected:
            raise Exception("Not connected to exchange")
        
        try:
            # Convert order to CCXT format
            ccxt_order = await self.exchange.create_order(
                symbol=order.symbol,
                type=order.type.value,
                side=order.side.value,
                amount=order.quantity,
                price=order.price if order.type != OrderType.MARKET else None
            )
            
            self.logger.info(f"Order submitted: {ccxt_order['id']}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to submit order: {e}")
            raise
    
    async def cancel_order(self, order_id: str) -> bool:
        """Cancel an order on Binance."""
        if not self.connected:
            raise Exception("Not connected to exchange")
        
        try:
            await self.exchange.cancel_order(order_id)
            self.logger.info(f"Order cancelled: {order_id}")
            return True
        except Exception as e:
            self.logger.error(f"Failed to cancel order {order_id}: {e}")
            return False
    
    async def get_order_status(self, order_id: str) -> Optional[Order]:
        """Get order status from Binance."""
        if not self.connected:
            return None
        
        try:
            ccxt_order = await self.exchange.fetch_order(order_id)
            
            # Map CCXT status to our OrderStatus
            status_map = {
                'open': OrderStatus.SUBMITTED,
                'closed': OrderStatus.FILLED,
                'canceled': OrderStatus.CANCELLED,
                'expired': OrderStatus.CANCELLED,
                'rejected': OrderStatus.REJECTED
            }
            
            return Order(
                id=ccxt_order['id'],
                symbol=ccxt_order['symbol'],
                side=OrderSide(ccxt_order['side']),
                type=OrderType(ccxt_order['type']),
                quantity=ccxt_order['amount'],
                price=ccxt_order.get('price'),
                status=status_map.get(ccxt_order['status'], OrderStatus.PENDING),
                timestamp=datetime.fromtimestamp(ccxt_order['timestamp'] / 1000),
                filled_quantity=ccxt_order['filled'],
                filled_price=ccxt_order.get('average')
            )
        except Exception as e:
            self.logger.error(f"Failed to get order status: {e}")
            return None
    
    async def get_ticker(self, symbol: str) -> Optional[Dict[str, float]]:
        """Get ticker data from Binance."""
        if not self.connected:
            return None
        
        try:
            ticker = await self.exchange.fetch_ticker(symbol)
            return {
                'last': ticker['last'],
                'bid': ticker['bid'],
                'ask': ticker['ask'],
                'high': ticker['high'],
                'low': ticker['low'],
                'volume': ticker['baseVolume'],
                'timestamp': ticker['timestamp']
            }
        except Exception as e:
            self.logger.error(f"Failed to get ticker for {symbol}: {e}")
            return None
    
    async def get_ohlcv(
        self, 
        symbol: str, 
        timeframe: str = '1m', 
        limit: int = 100
    ) -> Optional[List[List[float]]]:
        """Get OHLCV data from Binance."""
        if not self.connected:
            return None
        
        try:
            ohlcv = await self.exchange.fetch_ohlcv(symbol, timeframe, limit=limit)
            return ohlcv
        except Exception as e:
            self.logger.error(f"Failed to get OHLCV for {symbol}: {e}")
            return None
    
    async def get_balance(self) -> Dict[str, float]:
        """Get account balance from Binance."""
        if not self.connected:
            return {}
        
        try:
            balance = await self.exchange.fetch_balance()
            return balance['free']
        except Exception as e:
            self.logger.error(f"Failed to get balance: {e}")
            return {}
    
    async def get_positions(self) -> List[Dict[str, Any]]:
        """Get open positions from Binance (spot trading doesn't have positions)."""
        # For spot trading, we calculate positions from balance
        balance = await self.get_balance()
        positions = []
        
        for asset, amount in balance.items():
            if amount > 0 and asset != 'USDT':
                positions.append({
                    'symbol': f'{asset}/USDT',
                    'size': amount,
                    'side': 'long'
                })
        
        return positions
