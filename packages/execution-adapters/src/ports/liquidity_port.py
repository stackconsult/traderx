"""
Liquidity Port - Hexagonal Architecture Pattern
Abstract interface for all liquidity adapters.
"""

from abc import ABC, abstractmethod
from typing import List, Optional, Dict, Any, AsyncGenerator
from datetime import datetime
import asyncio

# Import domain types to avoid circular imports
from ..domain.models import (
    UnifiedOrder, ExecutionResult, Position, AccountInfo,
    VenueInfo, MarketData, LiquidityRequest, LiquidityResponse,
    OrderStatus, OrderType, TimeInForce
)


class LiquidityPort(ABC):
    """
    Abstract port for liquidity operations.
    All venue adapters must implement this interface.
    """
    
    def __init__(self, venue_name: str, config: Dict[str, Any]):
        self.venue_name = venue_name
        self.config = config
        self.is_connected = False
        self.venue_info: Optional[VenueInfo] = None
        self._connection_lock = asyncio.Lock()
        
    @abstractmethod
    async def connect(self) -> bool:
        """
        Establish connection to the venue.
        
        Returns:
            True if connection successful
        """
        pass
        
    @abstractmethod
    async def disconnect(self) -> None:
        """Close connection to the venue."""
        pass
        
    @abstractmethod
    async def submit_order(self, order: UnifiedOrder) -> ExecutionResult:
        """
        Submit an order to the venue.
        
        Args:
            order: Unified order to submit
            
        Returns:
            Execution result
        """
        pass
        
    @abstractmethod
    async def cancel_order(self, order_id: str, venue_order_id: Optional[str] = None) -> bool:
        """
        Cancel an existing order.
        
        Args:
            order_id: Client order ID
            venue_order_id: Venue-specific order ID
            
        Returns:
            True if cancellation successful
        """
        pass
        
    @abstractmethod
    async def get_order_status(self, order_id: str) -> Optional[ExecutionResult]:
        """
        Get current status of an order.
        
        Args:
            order_id: Order ID to check
            
        Returns:
            Current execution result or None
        """
        pass
        
    @abstractmethod
    async def get_positions(self) -> List[Position]:
        """
        Get all current positions.
        
        Returns:
            List of positions
        """
        pass
        
    @abstractmethod
    async def get_account_info(self) -> AccountInfo:
        """
        Get account information.
        
        Returns:
            Account info
        """
        pass
        
    @abstractmethod
    async def get_venue_info(self) -> VenueInfo:
        """
        Get venue information and capabilities.
        
        Returns:
            Venue info
        """
        pass
        
    async def get_market_data(self, symbols: List[str]) -> AsyncGenerator[MarketData, None]:
        """
        Stream market data for symbols.
        
        Args:
            symbols: Symbols to subscribe
            
        Yields:
            Market data ticks
        """
        # Default implementation - override in adapters that support streaming
        return
        yield  # type: ignore
        
    async def health_check(self) -> Dict[str, Any]:
        """
        Perform health check on the connection.
        
        Returns:
            Health status information
        """
        return {
            "venue": self.venue_name,
            "connected": self.is_connected,
            "timestamp": datetime.utcnow().isoformat(),
            "latency_ms": await self._measure_latency()
        }
        
    async def _measure_latency(self) -> float:
        """Measure round-trip latency to venue."""
        # Default implementation - override in concrete adapters
        return 0.0
        
    async def validate_order(self, order: UnifiedOrder) -> Optional[str]:
        """
        Validate order against venue constraints.
        
        Args:
            order: Order to validate
            
        Returns:
            Error message if invalid, None if valid
        """
        if not self.venue_info:
            return "Venue info not available"
            
        # Check symbol support
        if self.venue_info.supported_symbols and order.symbol not in self.venue_info.supported_symbols:
            return f"Symbol {order.symbol} not supported"
            
        # Check order type support
        if order.order_type not in self.venue_info.supported_order_types:
            return f"Order type {order.order_type.value} not supported"
            
        # Check time in force support
        if order.time_in_force not in self.venue_info.supported_tif:
            return f"Time in force {order.time_in_force.value} not supported"
            
        # Check order size
        if order.quantity < self.venue_info.min_order_size:
            return f"Order size {order.quantity} below minimum {self.venue_info.min_order_size}"
            
        if order.quantity > self.venue_info.max_order_size:
            return f"Order size {order.quantity} above maximum {self.venue_info.max_order_size}"
            
        # Check price constraints for limit orders
        if order.order_type in [OrderType.LIMIT, OrderType.STOP_LIMIT]:
            if not order.price:
                return "Price required for limit order"
                
            # Check tick size
            if order.price % self.venue_info.tick_size != 0:
                return f"Price must be multiple of tick size {self.venue_info.tick_size}"
                
        return None
        
    def _convert_order_to_venue_format(self, order: UnifiedOrder) -> Dict[str, Any]:
        """
        Convert unified order to venue-specific format.
        Override in concrete adapters.
        
        Args:
            order: Unified order
            
        Returns:
            Venue-specific order dictionary
        """
        return {
            "client_order_id": order.id,
            "symbol": order.symbol,
            "side": order.side.value,
            "quantity": order.quantity,
            "order_type": order.order_type.value,
            "price": order.price,
            "time_in_force": order.time_in_force.value
        }
        
    def _convert_result_from_venue_format(self, venue_result: Dict[str, Any]) -> ExecutionResult:
        """
        Convert venue-specific result to unified format.
        Override in concrete adapters.
        
        Args:
            venue_result: Venue-specific result
            
        Returns:
            Unified execution result
        """
        return ExecutionResult(
            order_id=venue_result.get("client_order_id", ""),
            venue_order_id=venue_result.get("order_id"),
            status=OrderStatus(venue_result.get("status", "NEW")),
            filled_quantity=venue_result.get("filled_quantity", 0.0),
            remaining_quantity=venue_result.get("remaining_quantity", 0.0),
            average_price=venue_result.get("average_price"),
            execution_venue=self.venue_name,
            fees=venue_result.get("fees", {}),
            error_message=venue_result.get("error_message")
        )
