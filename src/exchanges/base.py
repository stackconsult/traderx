from abc import ABC, abstractmethod
from typing import Dict, List, Optional, Any
import asyncio
import logging
from datetime import datetime

from ..core.models import Order, OrderStatus


class BaseExchange(ABC):
    """
    Abstract base class for exchange implementations.
    All exchange integrations should inherit from this class.
    """
    
    def __init__(self, name: str, config: Dict[str, Any]):
        self.name = name
        self.config = config
        self.logger = logging.getLogger(f"{__name__}.{name}")
        self.connected = False
        
    @abstractmethod
    async def connect(self):
        """Establish connection to the exchange."""
        pass
    
    @abstractmethod
    async def disconnect(self):
        """Close connection to the exchange."""
        pass
    
    @abstractmethod
    async def submit_order(self, order: Order) -> bool:
        """Submit an order to the exchange."""
        pass
    
    @abstractmethod
    async def cancel_order(self, order_id: str) -> bool:
        """Cancel an existing order."""
        pass
    
    @abstractmethod
    async def get_order_status(self, order_id: str) -> Optional[Order]:
        """Get the current status of an order."""
        pass
    
    @abstractmethod
    async def get_ticker(self, symbol: str) -> Optional[Dict[str, float]]:
        """Get current ticker data for a symbol."""
        pass
    
    @abstractmethod
    async def get_ohlcv(
        self, 
        symbol: str, 
        timeframe: str, 
        limit: int = 100
    ) -> Optional[List[List[float]]]:
        """Get OHLCV candlestick data."""
        pass
    
    @abstractmethod
    async def get_balance(self) -> Dict[str, float]:
        """Get account balance."""
        pass
    
    @abstractmethod
    async def get_positions(self) -> List[Dict[str, Any]]:
        """Get open positions."""
        pass
