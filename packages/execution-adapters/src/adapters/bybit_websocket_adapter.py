"""
Production Bybit WebSocket Adapter
Real implementation with Bybit API v5 authentication and order execution.
"""

import asyncio
import json
import websockets
import logging
import hmac
import hashlib
import time
from typing import List, Optional, Dict, Any, AsyncGenerator
from datetime import datetime
from urllib.parse import urlencode
import os

from ..ports.liquidity_port import LiquidityPort
from ..domain.models import (
    UnifiedOrder, ExecutionResult, Position, AccountInfo,
    VenueInfo, MarketData, OrderStatus, OrderType, TimeInForce, OrderSide
)

logger = logging.getLogger(__name__)


class BybitWebSocketAdapter(LiquidityPort):
    """Production WebSocket adapter for Bybit exchange."""
    
    def __init__(self, config: Dict[str, Any]):
        super().__init__("Bybit", config)
        
        # API credentials
        self.api_key = config.get("api_key") or os.getenv("BYBIT_API_KEY")
        self.api_secret = config.get("api_secret") or os.getenv("BYBIT_API_SECRET")
        self.testnet = config.get("testnet", False) or os.getenv("BYBIT_TESTNET", "false").lower() == "true"
        
        # WebSocket URLs
        if self.testnet:
            self.public_ws_url = "wss://stream-testnet.bybit.com/v5/public/linear"
            self.private_ws_url = "wss://stream-testnet.bybit.com/v5/private"
            self.rest_url = "https://api-testnet.bybit.com"
        else:
            self.public_ws_url = "wss://stream.bybit.com/v5/public/linear"
            self.private_ws_url = "wss://stream.bybit.com/v5/private"
            self.rest_url = "https://api.bybit.com"
            
        # WebSocket connections
        self._public_ws: Optional[websockets.WebSocketServerProtocol] = None
        self._private_ws: Optional[websockets.WebSocketServerProtocol] = None
        
        # Message handling
        self._message_id = 0
        self._pending_requests: Dict[str, asyncio.Future] = {}
        self._subscriptions: Dict[str, asyncio.Queue] = {}
        
        # Rate limiting
        self._rate_limiter = asyncio.Semaphore(5)  # 5 concurrent requests
        
    async def connect(self) -> bool:
        """Establish WebSocket connections to Bybit."""
        try:
            # Connect to public stream
            self._public_ws = await websockets.connect(self.public_ws_url)
            logger.info("Connected to Bybit public stream")
            
            # Connect to private stream if credentials provided
            if self.api_key and self.api_secret:
                await self._connect_private_stream()
                
            # Start message handlers
            asyncio.create_task(self._public_message_handler())
            if self._private_ws:
                asyncio.create_task(self._private_message_handler())
                
            self.is_connected = True
            
            # Initialize venue info
            await self._initialize_venue_info()
            
            logger.info(f"Bybit adapter connected (testnet={self.testnet})")
            return True
            
        except Exception as e:
            logger.error(f"Failed to connect to Bybit: {e}")
            self.is_connected = False
            return False
            
    async def _connect_private_stream(self):
        """Connect and authenticate to private stream."""
        self._private_ws = await websockets.connect(self.private_ws_url)
        
        # Authenticate
        expires = int(time.time() * 1000) + 10000
        signature = hmac.new(
            self.api_secret.encode(),
            f"GET/realtime{expires}".encode(),
            hashlib.sha256
        ).hexdigest()
        
        auth_msg = {
            "op": "auth",
            "args": [self.api_key, expires, signature]
        }
        
        await self._private_ws.send(json.dumps(auth_msg))
        
        # Wait for auth response
        response = await self._private_ws.recv()
        auth_data = json.loads(response)
        
        if auth_data.get("success") is not True:
            raise Exception(f"Bybit authentication failed: {auth_data}")
            
        logger.info("Bybit private stream authenticated")
        
    async def disconnect(self) -> None:
        """Close WebSocket connections."""
        if self._public_ws:
            await self._public_ws.close()
            self._public_ws = None
            
        if self._private_ws:
            await self._private_ws.close()
            self._private_ws = None
            
        self.is_connected = False
        logger.info("Disconnected from Bybit")
        
    async def submit_order(self, order: UnifiedOrder) -> ExecutionResult:
        """Submit order to Bybit via private WebSocket."""
        if not self._private_ws:
            return ExecutionResult(
                order_id=order.id,
                venue_order_id=None,
                status=OrderStatus.REJECTED,
                error_message="Private stream not connected"
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
            
        async with self._rate_limiter:
            try:
                # Prepare order request
                request_id = str(self._message_id + 1)
                self._message_id += 1
                
                order_params = {
                    "category": "linear",
                    "symbol": order.symbol,
                    "side": order.side.value.lower(),
                    "orderType": self._convert_order_type(order.order_type),
                    "qty": str(order.quantity),
                    "timeInForce": order.time_in_force.value,
                    "positionIdx": 0,
                    "orderLinkId": order.id
                }
                
                # Add price for limit orders
                if order.order_type in [OrderType.LIMIT, OrderType.STOP_LIMIT]:
                    if order.price:
                        order_params["price"] = str(order.price)
                    else:
                        return ExecutionResult(
                            order_id=order.id,
                            venue_order_id=None,
                            status=OrderStatus.REJECTED,
                            error_message="Price required for limit order"
                        )
                        
                # Add stop price for stop orders
                if order.order_type in [OrderType.STOP, OrderType.STOP_LIMIT]:
                    if order.stop_price:
                        order_params["triggerPrice"] = str(order.stop_price)
                    else:
                        return ExecutionResult(
                            order_id=order.id,
                            venue_order_id=None,
                            status=OrderStatus.REJECTED,
                            error_message="Stop price required for stop order"
                        )
                        
                # Create future for response
                self._pending_requests[order.id] = asyncio.Future()
                
                # Send order request
                request = {
                    "id": request_id,
                    "method": "private/order/create",
                    "params": order_params
                }
                
                await self._private_ws.send(json.dumps(request))
                
                # Wait for response
                result = await asyncio.wait_for(
                    self._pending_requests[order.id],
                    timeout=10.0
                )
                
                return result
                
            except asyncio.TimeoutError:
                del self._pending_requests[order.id]
                return ExecutionResult(
                    order_id=order.id,
                    venue_order_id=None,
                    status=OrderStatus.REJECTED,
                    error_message="Order timeout"
                )
            except Exception as e:
                if order.id in self._pending_requests:
                    del self._pending_requests[order.id]
                return ExecutionResult(
                    order_id=order.id,
                    venue_order_id=None,
                    status=OrderStatus.REJECTED,
                    error_message=str(e)
                )
                
    async def cancel_order(self, order_id: str, venue_order_id: Optional[str] = None) -> bool:
        """Cancel order on Bybit."""
        if not self._private_ws:
            return False
            
        async with self._rate_limiter:
            try:
                request_id = str(self._message_id + 1)
                self._message_id += 1
                
                request = {
                    "id": request_id,
                    "method": "private/order/cancel",
                    "params": {
                        "category": "linear",
                        "orderLinkId": order_id
                    }
                }
                
                if venue_order_id:
                    request["params"]["orderId"] = venue_order_id
                    
                await self._private_ws.send(json.dumps(request))
                return True
                
            except Exception as e:
                logger.error(f"Failed to cancel order {order_id}: {e}")
                return False
                
    async def get_order_status(self, order_id: str) -> Optional[ExecutionResult]:
        """Get order status from Bybit."""
        # Use REST API as fallback
        import aiohttp
        
        if not self.api_key or not self.api_secret:
            return None
            
        try:
            # Create signature
            timestamp = int(time.time() * 1000)
            query = f"category=linear&orderLinkId={order_id}"
            signature = hmac.new(
                self.api_secret.encode(),
                f"{timestamp}{self.api_key}{query}".encode(),
                hashlib.sha256
            ).hexdigest()
            
            headers = {
                "X-BAPI-API-KEY": self.api_key,
                "X-BAPI-SIGN": signature,
                "X-BAPI-SIGN-TYPE": "2",
                "X-BAPI-TIMESTAMP": str(timestamp),
                "X-BAPI-RECV-WINDOW": "5000"
            }
            
            async with aiohttp.ClientSession() as session:
                url = f"{self.rest_url}/v5/order/realtime?{query}"
                async with session.get(url, headers=headers) as response:
                    data = await response.json()
                    
                    if data.get("retCode") == 0 and data.get("result", {}).get("list"):
                        order_info = data["result"]["list"][0]
                        return self._convert_order_info(order_info)
                        
        except Exception as e:
            logger.error(f"Failed to get order status: {e}")
            
        return None
        
    async def get_positions(self) -> List[Position]:
        """Get current positions from Bybit."""
        import aiohttp
        
        if not self.api_key or not self.api_secret:
            return []
            
        try:
            # Create signature
            timestamp = int(time.time() * 1000)
            query = "category=linear"
            signature = hmac.new(
                self.api_secret.encode(),
                f"{timestamp}{self.api_key}{query}".encode(),
                hashlib.sha256
            ).hexdigest()
            
            headers = {
                "X-BAPI-API-KEY": self.api_key,
                "X-BAPI-SIGN": signature,
                "X-BAPI-SIGN-TYPE": "2",
                "X-BAPI-TIMESTAMP": str(timestamp),
                "X-BAPI-RECV-WINDOW": "5000"
            }
            
            async with aiohttp.ClientSession() as session:
                url = f"{self.rest_url}/v5/position/list?{query}"
                async with session.get(url, headers=headers) as response:
                    data = await response.json()
                    
                    positions = []
                    if data.get("retCode") == 0:
                        for pos_data in data.get("result", {}).get("list", []):
                            if float(pos_data.get("size", 0)) != 0:
                                position = Position(
                                    symbol=pos_data["symbol"],
                                    side=OrderSide.BUY if pos_data["side"] == "Buy" else OrderSide.SELL,
                                    quantity=float(pos_data["size"]),
                                    entry_price=float(pos_data["avgPrice"]),
                                    unrealized_pnl=float(pos_data.get("unrealisedPnl", 0)),
                                    venue="Bybit"
                                )
                                positions.append(position)
                                
                    return positions
                    
        except Exception as e:
            logger.error(f"Failed to get positions: {e}")
            return []
            
    async def get_account_info(self) -> AccountInfo:
        """Get account information from Bybit."""
        import aiohttp
        
        if not self.api_key or not self.api_secret:
            return AccountInfo(
                account_id="bybit-demo",
                venue="Bybit",
                balance={},
                positions=[]
            )
            
        try:
            # Get wallet balance
            timestamp = int(time.time() * 1000)
            query = "accountType=UNIFIED"
            signature = hmac.new(
                self.api_secret.encode(),
                f"{timestamp}{self.api_key}{query}".encode(),
                hashlib.sha256
            ).hexdigest()
            
            headers = {
                "X-BAPI-API-KEY": self.api_key,
                "X-BAPI-SIGN": signature,
                "X-BAPI-SIGN-TYPE": "2",
                "X-BAPI-TIMESTAMP": str(timestamp),
                "X-BAPI-RECV-WINDOW": "5000"
            }
            
            async with aiohttp.ClientSession() as session:
                url = f"{self.rest_url}/v5/account/wallet-balance?{query}"
                async with session.get(url, headers=headers) as response:
                    data = await response.json()
                    
                    balance = {}
                    if data.get("retCode") == 0:
                        for coin in data.get("result", {}).get("list", []):
                            for asset in coin.get("coin", []):
                                balance[asset["coin"]] = float(asset["walletBalance"])
                                
                    positions = await self.get_positions()
                    
                    return AccountInfo(
                        account_id="bybit-unified",
                        venue="Bybit",
                        balance=balance,
                        positions=positions
                    )
                    
        except Exception as e:
            logger.error(f"Failed to get account info: {e}")
            return AccountInfo(
                account_id="bybit-error",
                venue="Bybit",
                balance={},
                positions=[]
            )
            
    async def get_venue_info(self) -> VenueInfo:
        """Get Bybit venue information."""
        if not self.venue_info:
            await self._initialize_venue_info()
        return self.venue_info
        
    async def get_market_data(self, symbols: List[str]) -> AsyncGenerator[MarketData, None]:
        """Stream market data from Bybit."""
        if not self._public_ws:
            return
            
        # Subscribe to ticker updates
        for symbol in symbols:
            subscribe_msg = {
                "op": "subscribe",
                "args": [f"tickers.{symbol}"]
            }
            await self._public_ws.send(json.dumps(subscribe_msg))
            
        # Stream updates
        while self._public_ws and self.is_connected:
            try:
                queue_key = f"tickers.{symbols[0]}"
                if queue_key in self._subscriptions:
                    data = await self._subscriptions[queue_key].get()
                    yield self._parse_ticker_data(data)
            except Exception as e:
                logger.error(f"Error in market data stream: {e}")
                await asyncio.sleep(1)
                
    async def _public_message_handler(self):
        """Handle public stream messages."""
        try:
            async for message in self._public_ws:
                data = json.loads(message)
                
                if "topic" in data and "tickers" in data["topic"]:
                    queue_key = data["topic"]
                    if queue_key not in self._subscriptions:
                        self._subscriptions[queue_key] = asyncio.Queue()
                    await self._subscriptions[queue_key].put(data["data"])
                    
        except websockets.exceptions.ConnectionClosed:
            logger.warning("Bybit public stream connection closed")
            self.is_connected = False
            
    async def _private_message_handler(self):
        """Handle private stream messages."""
        try:
            async for message in self._private_ws:
                data = json.loads(message)
                
                # Handle order responses
                if data.get("topic") == "order":
                    order_data = data["data"][0] if data.get("data") else {}
                    await self._handle_order_update(order_data)
                    
                # Handle request responses
                elif "request" in data and data.get("request", {}).get("operation") == "private/order/create":
                    order_data = data.get("data", {})
                    await self._handle_order_response(order_data)
                    
        except websockets.exceptions.ConnectionClosed:
            logger.warning("Bybit private stream connection closed")
            self.is_connected = False
            
    async def _handle_order_response(self, order_data: Dict[str, Any]):
        """Handle order creation response."""
        order_link_id = order_data.get("orderLinkId")
        if order_link_id and order_link_id in self._pending_requests:
            result = self._convert_order_info(order_data)
            self._pending_requests[order_link_id].set_result(result)
            del self._pending_requests[order_link_id]
            
    async def _handle_order_update(self, order_data: Dict[str, Any]):
        """Handle order update."""
        # Could be used for real-time order status updates
        pass
        
    def _convert_order_info(self, order_data: Dict[str, Any]) -> ExecutionResult:
        """Convert Bybit order info to unified format."""
        status_map = {
            "New": OrderStatus.NEW,
            "PartiallyFilled": OrderStatus.PARTIALLY_FILLED,
            "Filled": OrderStatus.FILLED,
            "Cancelled": OrderStatus.CANCELLED,
            "Rejected": OrderStatus.REJECTED,
            "PartiallyFilledCanceled": OrderStatus.CANCELLED
        }
        
        return ExecutionResult(
            order_id=order_data.get("orderLinkId", ""),
            venue_order_id=order_data.get("orderId"),
            status=status_map.get(order_data.get("orderStatus"), OrderStatus.NEW),
            filled_quantity=float(order_data.get("cumExecQty", 0)),
            remaining_quantity=float(order_data.get("leavesQty", 0)),
            average_price=float(order_data.get("avgPrice", 0)) if order_data.get("avgPrice") else None,
            execution_venue="Bybit",
            fees={"takerFee": float(order_data.get("cumExecFee", 0))},
            timestamp=datetime.fromisoformat(order_data.get("updatedTime", "").replace("Z", "+00:00"))
        )
        
    def _convert_order_type(self, order_type: OrderType) -> str:
        """Convert unified order type to Bybit format."""
        type_map = {
            OrderType.MARKET: "Market",
            OrderType.LIMIT: "Limit",
            OrderType.STOP: "Stop",
            OrderType.STOP_LIMIT: "StopLimit"
        }
        return type_map.get(order_type, "Market")
        
    def _parse_ticker_data(self, data: Dict[str, Any]) -> MarketData:
        """Parse ticker data from Bybit."""
        if isinstance(data, list):
            data = data[0]
            
        return MarketData(
            symbol=data.get("symbol", ""),
            venue="Bybit",
            bid=float(data.get("bid1Price", 0)) if data.get("bid1Price") else None,
            ask=float(data.get("ask1Price", 0)) if data.get("ask1Price") else None,
            bid_size=float(data.get("bid1Size", 0)) if data.get("bid1Size") else None,
            ask_size=float(data.get("ask1Size", 0)) if data.get("ask1Size") else None,
            last=float(data.get("lastPrice", 0)) if data.get("lastPrice") else None,
            volume=float(data.get("turnover24h", 0)) if data.get("turnover24h") else None,
            timestamp=datetime.fromisoformat(data.get("time", "").replace("Z", "+00:00"))
        )
        
    async def _initialize_venue_info(self):
        """Initialize venue information."""
        self.venue_info = VenueInfo(
            name="Bybit",
            venue_type="WebSocket",
            supported_symbols=[
                "BTCUSDT", "ETHUSDT", "SOLUSDT", "ADAUSDT", "DOTUSDT",
                "LINKUSDT", "MATICUSDT", "AVAXUSDT", "UNIUSDT", "AAVEUSDT"
            ],
            supported_order_types=[OrderType.MARKET, OrderType.LIMIT, OrderType.STOP, OrderType.STOP_LIMIT],
            supported_tif=[TimeInForce.IOC, TimeInForce.GTC, TimeInForce.FOK, TimeInForce.DAY],
            min_order_size=0.001,
            max_order_size=1000.0,
            tick_size=0.01,
            connection_status=self.is_connected,
            latency_ms=await self._measure_latency(),
            rate_limits={
                "order_rate": 10,  # orders per second
                "connect_rate": 5   # connections per second
            }
        )
        
    async def _measure_latency(self) -> float:
        """Measure WebSocket latency."""
        if not self._public_ws:
            return float('inf')
            
        start_time = time.time()
        try:
            await self._public_ws.ping()
            return (time.time() - start_time) * 1000
        except:
            return float('inf')
