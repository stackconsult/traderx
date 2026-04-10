"""
Market Data Port Base Class
Base interface for all market data adapters
"""

from abc import ABC, abstractmethod
from typing import Dict, Any, Optional, List, Callable
import asyncio
import logging

logger = logging.getLogger(__name__)


class MarketDataPort(ABC):
    """Base class for market data adapters."""
    
    def __init__(self, venue_name: str, config: Dict[str, Any]):
        self.venue_name = venue_name
        self.config = config
        self.is_connected = False
        self._event_handlers = {
            'trade': [],
            'quote': [],
            'orderbook': [],
            'bbo': [],
        }
    
    @abstractmethod
    async def connect(self) -> bool:
        """Connect to the venue."""
        pass
    
    @abstractmethod
    async def disconnect(self) -> None:
        """Disconnect from the venue."""
        pass
    
    @abstractmethod
    async def subscribe(self, symbols: List[str]) -> bool:
        """Subscribe to market data for symbols."""
        pass
    
    @abstractmethod
    async def unsubscribe(self, symbols: List[str]) -> bool:
        """Unsubscribe from market data for symbols."""
        pass
    
    def add_event_handler(self, event_type: str, handler: Callable) -> None:
        """Add event handler for specific market data type."""
        if event_type in self._event_handlers:
            self._event_handlers[event_type].append(handler)
    
    def remove_event_handler(self, event_type: str, handler: Callable) -> None:
        """Remove event handler."""
        if event_type in self._event_handlers:
            try:
                self._event_handlers[event_type].remove(handler)
            except ValueError:
                pass
    
    async def _emit_event(self, event_type: str, data: Any) -> None:
        """Emit event to all registered handlers."""
        for handler in self._event_handlers.get(event_type, []):
            try:
                if asyncio.iscoroutinefunction(handler):
                    await handler(data)
                else:
                    handler(data)
            except Exception as e:
                logger.error(f"Error in {event_type} handler: {e}")
    
    def get_status(self) -> Dict[str, Any]:
        """Get connection status."""
        return {
            'venue': self.venue_name,
            'connected': self.is_connected,
            'config': self.config,
        }
