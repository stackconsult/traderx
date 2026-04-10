"""
WebSocket Adapter for Crypto Exchanges
Handles real-time communication with venues like Bybit, MEXC, Binance.
"""

import asyncio
import json
import websockets
import logging
from typing import List, Optional, Dict, Any, AsyncGenerator
from datetime import datetime
import hmac
import hashlib
import time
from urllib.parse import urlencode

from ..ports.liquidity_port import LiquidityPort
from ..domain.models import (
    UnifiedOrder, ExecutionResult, Position, AccountInfo,
    VenueInfo, MarketData, OrderStatus, OrderType, TimeInForce, OrderSide
)

logger = logging.getLogger(__name__)


class WebSocketAdapter(LiquidityPort):
    """WebSocket adapter for crypto exchanges."""
    
    def __init__(self, venue_name: str, config: Dict[str, Any]):
        super().__init__(venue_name, config)
        self.ws_url: Optional[str] = None
        self.api_key: Optional[str] = None
        self.api_secret: Optional[str] = None
        self.testnet: bool = config.get("testnet", False)
        
        # WebSocket connection
        self._ws: Optional[websockets.WebSocketServerProtocol] = None
        self._subscription_id = 0
        self._subscriptions: Dict[str, asyncio.Queue] = {}
        self._message_handlers: Dict[str, callable] = {}
        
        # Order tracking
        self._pending_orders: Dict[str, UnifiedOrder] = {}
        self._order_results: Dict[str, asyncio.Future] = {}
        
    async def connect(self) -> bool:
        """Establish WebSocket connection."""
        try:
            # Build WebSocket URL based on venue
            if self.venue_name.lower() == "bybit":
                self.ws_url = "wss://stream.bybit.com/v5/public/linear" if not self.testnet \
                           else "wss://stream-testnet.bybit.com/v5/public/linear"
                self.api_key = self.config.get("api_key")
                self.api_secret = self.config.get("api_secret")
                
            elif self.venue_name.lower() == "mexc":
                self.ws_url = "wss://wbs.mexc.com/ws" if not self.testnet \
                           else "wss://wbs.testnet.mexc.com/ws"
                           
            elif self.venue_name.lower() == "binance":
                self.ws_url = "wss://fstream.binance.com/ws" if not self.testnet \
                           else "wss://stream.binancefuture.com/ws"
                           
            else:
                raise ValueError(f"Unsupported venue: {self.venue_name}")
                
            # Connect WebSocket
            self._ws = await websockets.connect(self.ws_url)
            self.is_connected = True
            
            # Start message handler
            asyncio.create_task(self._message_handler())
            
            # Initialize venue info
            await self._initialize_venue_info()
            
            logger.info(f"Connected to {self.venue_name} via WebSocket")
            return True
            
        except Exception as e:
            logger.error(f"Failed to connect to {self.venue_name}: {e}")
            self.is_connected = False
            return False
            
    async def disconnect(self) -> None:
        """Close WebSocket connection."""
        if self._ws:
            await self._ws.close()
            self._ws = None
        self.is_connected = False
        logger.info(f"Disconnected from {self.venue_name}")
        
    async def submit_order(self, order: UnifiedOrder) -> ExecutionResult:
        """Submit order via WebSocket."""
        if not self.is_connected:
            return ExecutionResult(
                order_id=order.id,
                venue_order_id=None,
                status=OrderStatus.REJECTED,
                error_message="Not connected"
            )
            
        # Validate order
        error = await self.validate_order(order)
        if error:
            return ExecutionResult(
                order_id=order.id,
                venue_order_id=None,
                status=OrderStatus.REJECTED,
                error_message=error
            )
            
        # Store pending order
        self._pending_orders[order.id] = order
        self._order_results[order.id] = asyncio.Future()
        
        # Convert to venue format
        venue_order = self._convert_order_to_venue_format(order)
        
        # Send order
        try:
            if self.venue_name.lower() == "bybit":
                await self._submit_bybit_order(venue_order)
            elif self.venue_name.lower() == "mexc":
                await self._submit_mexc_order(venue_order)
            elif self.venue_name.lower() == "binance":
                await self._submit_binance_order(venue_order)
                
            # Wait for response
            result = await asyncio.wait_for(
                self._order_results[order.id],
                timeout=5.0
            )
            
            return result
            
        except asyncio.TimeoutError:
            del self._pending_orders[order.id]
            del self._order_results[order.id]
            return ExecutionResult(
                order_id=order.id,
                venue_order_id=None,
                status=OrderStatus.REJECTED,
                error_message="Order timeout"
            )
        except Exception as e:
            del self._pending_orders[order.id]
            del self._order_results[order.id]
            return ExecutionResult(
                order_id=order.id,
                venue_order_id=None,
                status=OrderStatus.REJECTED,
                error_message=str(e)
            )
            
    async def cancel_order(self, order_id: str, venue_order_id: Optional[str] = None) -> bool:
        """Cancel order via WebSocket."""
        if not self.is_connected:
            return False
            
        try:
            if self.venue_name.lower() == "bybit":
                await self._cancel_bybit_order(venue_order_id or order_id)
            elif self.venue_name.lower() == "mexc":
                await self._cancel_mexc_order(venue_order_id or order_id)
            elif self.venue_name.lower() == "binance":
                await self._cancel_binance_order(venue_order_id or order_id)
                
            return True
            
        except Exception as e:
            logger.error(f"Failed to cancel order {order_id}: {e}")
            return False
            
    async def get_order_status(self, order_id: str) -> Optional[ExecutionResult]:
        """Get order status."""
        # For WebSocket, we rely on real-time updates
        # This would typically query REST API as fallback
        return None
        
    async def get_positions(self) -> List[Position]:
        """Get current positions."""
        # WebSocket adapters typically rely on REST API for positions
        # This is a placeholder implementation
        return []
        
    async def get_account_info(self) -> AccountInfo:
        """Get account information."""
        # WebSocket adapters typically rely on REST API for account info
        # This is a placeholder implementation
        return AccountInfo(
            account_id="ws-account",
            venue=self.venue_name,
            balance={},
            positions=[]
        )
        
    async def get_venue_info(self) -> VenueInfo:
        """Get venue information."""
        if not self.venue_info:
            await self._initialize_venue_info()
        return self.venue_info
        
    async def get_market_data(self, symbols: List[str]) -> AsyncGenerator[MarketData, None]:
        """Stream market data."""
        if not self.is_connected:
            return
            
        # Subscribe to ticker updates
        for symbol in symbols:
            await self._subscribe_ticker(symbol)
            
        # Yield market data updates
        while self.is_connected:
            try:
                # Get data from subscription queue
                queue_key = f"ticker.{symbols[0]}"  # Simplified
                if queue_key in self._subscriptions:
                    data = await self._subscriptions[queue_key].get()
                    yield self._parse_market_data(data)
            except Exception as e:
                logger.error(f"Error in market data stream: {e}")
                await asyncio.sleep(1)
                
    async def _message_handler(self):
        """Handle incoming WebSocket messages."""
        try:
            async for message in self._ws:
                data = json.loads(message)
                await self._process_message(data)
        except websockets.exceptions.ConnectionClosed:
            logger.warning(f"WebSocket connection closed for {self.venue_name}")
            self.is_connected = False
        except Exception as e:
            logger.error(f"Error in message handler: {e}")
            
    async def _process_message(self, data: Dict[str, Any]):
        """Process incoming message."""
        if self.venue_name.lower() == "bybit":
            await self._process_bybit_message(data)
        elif self.venue_name.lower() == "mexc":
            await self._process_mexc_message(data)
        elif self.venue_name.lower() == "binance":
            await self._process_binance_message(data)
            
    async def _submit_bybit_order(self, order: Dict[str, Any]):
        """Submit order to Bybit."""
        payload = {
            "id": str(self._subscription_id + 1),
            "method": "private/order/create",
            "params": {
                "category": "linear",
                "symbol": order["symbol"],
                "side": order["side"].lower(),
                "orderType": order["order_type"].lower(),
                "qty": str(order["quantity"]),
                "price": str(order["price"]) if order["price"] else None,
                "timeInForce": order["time_in_force"],
                "positionIdx": 0
            }
        }
        
        # Sign request
        if self.api_key and self.api_secret:
            payload["params"]["apiKey"] = self.api_key
            payload["params"]["timestamp"] = int(time.time() * 1000)
            payload["params"]["sign"] = self._sign_bybit_request(payload["params"])
            
        await self._ws.send(json.dumps(payload))
        
    async def _submit_mexc_order(self, order: Dict[str, Any]):
        """Submit order to MEXC."""
        # Implementation similar to Bybit but with MEXC-specific format
        pass
        
    async def _submit_binance_order(self, order: Dict[str, Any]):
        """Submit order to Binance."""
        # Implementation with Binance-specific WebSocket API
        pass
        
    async def _cancel_bybit_order(self, order_id: str):
        """Cancel order on Bybit."""
        payload = {
            "id": str(self._subscription_id + 1),
            "method": "private/order/cancel",
            "params": {
                "category": "linear",
                "orderId": order_id
            }
        }
        
        if self.api_key and self.api_secret:
            payload["params"]["apiKey"] = self.api_key
            payload["params"]["timestamp"] = int(time.time() * 1000)
            payload["params"]["sign"] = self._sign_bybit_request(payload["params"])
            
        await self._ws.send(json.dumps(payload))
        
    def _sign_bybit_request(self, params: Dict[str, Any]) -> str:
        """Sign Bybit request."""
        # Sort parameters
        sorted_params = sorted(params.items())
        query_string = urlencode(sorted_params)
        
        # Create signature
        signature = hmac.new(
            self.api_secret.encode(),
            query_string.encode(),
            hashlib.sha256
        ).hexdigest()
        
        return signature
        
    async def _initialize_venue_info(self):
        """Initialize venue information."""
        self.venue_info = VenueInfo(
            name=self.venue_name,
            venue_type="WebSocket",
            supported_symbols=["BTCUSDT", "ETHUSDT", "SOLUSDT"],  # Simplified
            supported_order_types=[OrderType.MARKET, OrderType.LIMIT],
            supported_tif=[TimeInForce.IOC, TimeInForce.GTC, TimeInForce.FOK],
            min_order_size=0.001,
            tick_size=0.01
        )
        
    async def _measure_latency(self) -> float:
        """Measure WebSocket latency."""
        if not self.is_connected:
            return float('inf')
            
        start_time = time.time()
        try:
            # Send ping
            await self._ws.ping()
            return (time.time() - start_time) * 1000
        except:
            return float('inf')
            
    def _parse_market_data(self, data: Dict[str, Any]) -> MarketData:
        """Parse market data from venue format."""
        # Implementation depends on venue format
        return MarketData(
            symbol=data.get("symbol", ""),
            venue=self.venue_name,
            bid=data.get("bid"),
            ask=data.get("ask"),
            last=data.get("last")
        )
        
    async def _subscribe_ticker(self, symbol: str):
        """Subscribe to ticker updates."""
        # Implementation depends on venue
        pass
        
    async def _process_bybit_message(self, data: Dict[str, Any]):
        """Process Bybit message."""
        if "topic" in data:
            if "order" in data["topic"]:
                # Order update
                await self._handle_order_update(data["data"])
            elif "ticker" in data["topic"]:
                # Market data update
                queue_key = f"ticker.{data['topic'].split('.')[2]}"
                if queue_key not in self._subscriptions:
                    self._subscriptions[queue_key] = asyncio.Queue()
                await self._subscriptions[queue_key].put(data["data"])
                
    async def _handle_order_update(self, order_data: Dict[str, Any]):
        """Handle order update message."""
        order_id = order_data.get("orderLinkId")
        if order_id and order_id in self._order_results:
            result = self._convert_result_from_venue_format({
                "client_order_id": order_id,
                "order_id": order_data.get("orderId"),
                "status": order_data.get("orderStatus"),
                "filled_quantity": float(order_data.get("cumExecQty", 0)),
                "remaining_quantity": float(order_data.get("leavesQty", 0)),
                "average_price": float(order_data.get("avgPrice", 0)) if order_data.get("avgPrice") else None
            })
            
            self._order_results[order_id].set_result(result)
            del self._order_results[order_id]
            del self._pending_orders[order_id]
