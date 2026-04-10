"""
Production REST Adapter for venues with REST APIs only.
Handles authentication, rate limiting, and order execution via HTTP.
"""

import asyncio
import aiohttp
import logging
import hmac
import hashlib
import time
from typing import List, Optional, Dict, Any, AsyncGenerator
from datetime import datetime
import json
import os

from ..ports.liquidity_port import LiquidityPort
from ..domain.models import (
    UnifiedOrder, ExecutionResult, Position, AccountInfo,
    VenueInfo, MarketData, OrderStatus, OrderType, TimeInForce, OrderSide
)

logger = logging.getLogger(__name__)


class RESTAdapter(LiquidityPort):
    """REST adapter for venues with HTTP APIs."""
    
    def __init__(self, venue_name: str, config: Dict[str, Any]):
        super().__init__(venue_name, config)
        
        # API configuration
        self.api_key = config.get("api_key")
        self.api_secret = config.get("api_secret")
        self.base_url = config.get("base_url")
        self.testnet = config.get("testnet", False)
        
        # HTTP session
        self._session: Optional[aiohttp.ClientSession] = None
        
        # Rate limiting
        self._rate_limiter = asyncio.Semaphore(config.get("rate_limit", 5))
        self._last_request_time = 0.0
        self._min_request_interval = 1.0 / config.get("requests_per_second", 5)
        
        # Authentication type
        self.auth_type = config.get("auth_type", "hmac")  # hmac, bearer, api_key
        
    async def connect(self) -> bool:
        """Initialize HTTP session and test connection."""
        try:
            # Create HTTP session with timeout
            timeout = aiohttp.ClientTimeout(total=30, connect=10)
            self._session = aiohttp.ClientSession(
                timeout=timeout,
                headers=self._get_default_headers()
            )
            
            # Test connection
            if await self._test_connection():
                self.is_connected = True
                await self._initialize_venue_info()
                logger.info(f"Connected to {self.venue_name} via REST API")
                return True
            else:
                await self.disconnect()
                return False
                
        except Exception as e:
            logger.error(f"Failed to connect to {self.venue_name}: {e}")
            self.is_connected = False
            return False
            
    async def disconnect(self) -> None:
        """Close HTTP session."""
        if self._session:
            await self._session.close()
            self._session = None
        self.is_connected = False
        logger.info(f"Disconnected from {self.venue_name}")
        
    async def submit_order(self, order: UnifiedOrder) -> ExecutionResult:
        """Submit order via REST API."""
        if not self._session:
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
            
        async with self._rate_limiter:
            try:
                # Rate limiting
                await self._wait_for_rate_limit()
                
                # Prepare request
                endpoint = self._get_order_endpoint()
                payload = self._prepare_order_payload(order)
                headers = await self._sign_request("POST", endpoint, payload)
                
                # Send request
                async with self._session.post(
                    f"{self.base_url}{endpoint}",
                    json=payload,
                    headers=headers
                ) as response:
                    data = await response.json()
                    
                    if response.status == 200:
                        return self._parse_order_response(data, order.id)
                    else:
                        return ExecutionResult(
                            order_id=order.id,
                            venue_order_id=None,
                            status=OrderStatus.REJECTED,
                            error_message=data.get("message", f"HTTP {response.status}")
                        )
                        
            except Exception as e:
                logger.error(f"Failed to submit order: {e}")
                return ExecutionResult(
                    order_id=order.id,
                    venue_order_id=None,
                    status=OrderStatus.REJECTED,
                    error_message=str(e)
                )
                
    async def cancel_order(self, order_id: str, venue_order_id: Optional[str] = None) -> bool:
        """Cancel order via REST API."""
        if not self._session:
            return False
            
        async with self._rate_limiter:
            try:
                await self._wait_for_rate_limit()
                
                endpoint = self._get_cancel_endpoint()
                payload = {
                    "order_id": venue_order_id,
                    "client_order_id": order_id
                }
                headers = await self._sign_request("DELETE", endpoint, payload)
                
                async with self._session.delete(
                    f"{self.base_url}{endpoint}",
                    json=payload,
                    headers=headers
                ) as response:
                    return response.status == 200
                    
            except Exception as e:
                logger.error(f"Failed to cancel order {order_id}: {e}")
                return False
                
    async def get_order_status(self, order_id: str) -> Optional[ExecutionResult]:
        """Get order status via REST API."""
        if not self._session:
            return None
            
        async with self._rate_limiter:
            try:
                await self._wait_for_rate_limit()
                
                endpoint = self._get_order_status_endpoint(order_id)
                headers = await self._sign_request("GET", endpoint, {})
                
                async with self._session.get(
                    f"{self.base_url}{endpoint}",
                    headers=headers
                ) as response:
                    if response.status == 200:
                        data = await response.json()
                        return self._parse_order_response(data, order_id)
                        
            except Exception as e:
                logger.error(f"Failed to get order status: {e}")
                
        return None
        
    async def get_positions(self) -> List[Position]:
        """Get positions via REST API."""
        if not self._session:
            return []
            
        async with self._rate_limiter:
            try:
                await self._wait_for_rate_limit()
                
                endpoint = self._get_positions_endpoint()
                headers = await self._sign_request("GET", endpoint, {})
                
                async with self._session.get(
                    f"{self.base_url}{endpoint}",
                    headers=headers
                ) as response:
                    if response.status == 200:
                        data = await response.json()
                        return self._parse_positions_response(data)
                        
            except Exception as e:
                logger.error(f"Failed to get positions: {e}")
                
        return []
        
    async def get_account_info(self) -> AccountInfo:
        """Get account info via REST API."""
        if not self._session:
            return AccountInfo(
                account_id="rest-error",
                venue=self.venue_name,
                balance={},
                positions=[]
            )
            
        async with self._rate_limiter:
            try:
                await self._wait_for_rate_limit()
                
                # Get balance
                balance_endpoint = self._get_balance_endpoint()
                headers = await self._sign_request("GET", balance_endpoint, {})
                
                async with self._session.get(
                    f"{self.base_url}{balance_endpoint}",
                    headers=headers
                ) as response:
                    balance_data = await response.json() if response.status == 200 else {}
                    
                # Get positions
                positions = await self.get_positions()
                
                return AccountInfo(
                    account_id=balance_data.get("account_id", "rest-account"),
                    venue=self.venue_name,
                    balance=balance_data.get("balances", {}),
                    positions=positions
                )
                
            except Exception as e:
                logger.error(f"Failed to get account info: {e}")
                return AccountInfo(
                    account_id="rest-error",
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
        """Get market data via REST polling."""
        if not self._session:
            return
            
        while self.is_connected:
            try:
                for symbol in symbols:
                    await self._wait_for_rate_limit()
                    
                    endpoint = self._get_ticker_endpoint(symbol)
                    headers = await self._sign_request("GET", endpoint, {})
                    
                    async with self._session.get(
                        f"{self.base_url}{endpoint}",
                        headers=headers
                    ) as response:
                        if response.status == 200:
                            data = await response.json()
                            yield self._parse_ticker_response(data, symbol)
                            
                await asyncio.sleep(1.0)  # Poll every second
                
            except Exception as e:
                logger.error(f"Error in market data polling: {e}")
                await asyncio.sleep(5.0)
                
    async def _test_connection(self) -> bool:
        """Test API connection."""
        try:
            endpoint = self._get_status_endpoint()
            headers = await self._sign_request("GET", endpoint, {})
            
            async with self._session.get(
                f"{self.base_url}{endpoint}",
                headers=headers
            ) as response:
                return response.status == 200
                
        except:
            return False
            
    async def _wait_for_rate_limit(self):
        """Wait for rate limit."""
        elapsed = time.time() - self._last_request_time
        if elapsed < self._min_request_interval:
            await asyncio.sleep(self._min_request_interval - elapsed)
        self._last_request_time = time.time()
        
    def _get_default_headers(self) -> Dict[str, str]:
        """Get default HTTP headers."""
        return {
            "User-Agent": "TraderX/1.0",
            "Content-Type": "application/json",
            "Accept": "application/json"
        }
        
    async def _sign_request(self, method: str, endpoint: str, payload: Dict[str, Any]) -> Dict[str, str]:
        """Sign HTTP request."""
        headers = {}
        
        if self.auth_type == "bearer" and self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"
        elif self.auth_type == "api_key" and self.api_key:
            headers["X-API-Key"] = self.api_key
        elif self.auth_type == "hmac" and self.api_key and self.api_secret:
            timestamp = str(int(time.time()))
            message = f"{timestamp}{method}{endpoint}{json.dumps(payload)}"
            signature = hmac.new(
                self.api_secret.encode(),
                message.encode(),
                hashlib.sha256
            ).hexdigest()
            
            headers.update({
                "X-API-Key": self.api_key,
                "X-Timestamp": timestamp,
                "X-Signature": signature
            })
            
        return headers
        
    def _prepare_order_payload(self, order: UnifiedOrder) -> Dict[str, Any]:
        """Prepare order payload for venue."""
        payload = {
            "symbol": order.symbol,
            "side": order.side.value.lower(),
            "type": order.order_type.value.lower(),
            "quantity": order.quantity,
            "client_order_id": order.id,
            "time_in_force": order.time_in_force.value
        }
        
        if order.price:
            payload["price"] = order.price
            
        if order.stop_price:
            payload["stop_price"] = order.stop_price
            
        return payload
        
    def _parse_order_response(self, data: Dict[str, Any], client_order_id: str) -> ExecutionResult:
        """Parse order response from venue."""
        # Override in venue-specific implementations
        return ExecutionResult(
            order_id=client_order_id,
            venue_order_id=data.get("order_id"),
            status=OrderStatus(data.get("status", "NEW")),
            filled_quantity=float(data.get("filled_quantity", 0)),
            remaining_quantity=float(data.get("remaining_quantity", 0)),
            average_price=float(data.get("average_price")) if data.get("average_price") else None,
            execution_venue=self.venue_name,
            fees=data.get("fees", {}),
            timestamp=datetime.utcnow()
        )
        
    def _parse_positions_response(self, data: Dict[str, Any]) -> List[Position]:
        """Parse positions response."""
        # Override in venue-specific implementations
        return []
        
    def _parse_ticker_response(self, data: Dict[str, Any], symbol: str) -> MarketData:
        """Parse ticker response."""
        # Override in venue-specific implementations
        return MarketData(
            symbol=symbol,
            venue=self.venue_name,
            timestamp=datetime.utcnow()
        )
        
    # Abstract methods to be overridden by venue implementations
    def _get_order_endpoint(self) -> str:
        raise NotImplementedError
        
    def _get_cancel_endpoint(self) -> str:
        raise NotImplementedError
        
    def _get_order_status_endpoint(self, order_id: str) -> str:
        raise NotImplementedError
        
    def _get_positions_endpoint(self) -> str:
        raise NotImplementedError
        
    def _get_balance_endpoint(self) -> str:
        raise NotImplementedError
        
    def _get_ticker_endpoint(self, symbol: str) -> str:
        raise NotImplementedError
        
    def _get_status_endpoint(self) -> str:
        raise NotImplementedError
        
    async def _initialize_venue_info(self):
        """Initialize venue information."""
        self.venue_info = VenueInfo(
            name=self.venue_name,
            venue_type="REST",
            supported_symbols=[],
            supported_order_types=[OrderType.MARKET, OrderType.LIMIT],
            supported_tif=[TimeInForce.IOC, TimeInForce.GTC],
            min_order_size=0.001,
            max_order_size=1000.0,
            tick_size=0.01,
            connection_status=self.is_connected,
            latency_ms=await self._measure_latency()
        )
        
    async def _measure_latency(self) -> float:
        """Measure API latency."""
        if not self._session:
            return float('inf')
            
        start_time = time.time()
        try:
            endpoint = self._get_status_endpoint()
            headers = await self._sign_request("GET", endpoint, {})
            
            async with self._session.get(
                f"{self.base_url}{endpoint}",
                headers=headers
            ) as response:
                return (time.time() - start_time) * 1000 if response.status == 200 else float('inf')
        except:
            return float('inf')
