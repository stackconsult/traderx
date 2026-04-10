from abc import ABC, abstractmethod
from typing import Dict, List, Optional, Any
import logging
from dataclasses import dataclass
from datetime import datetime
from enum import Enum


class SignalType(Enum):
    BUY = "buy"
    SELL = "sell"
    CLOSE = "close"
    HOLD = "hold"


class SignalStrength(Enum):
    WEAK = 1
    MODERATE = 2
    STRONG = 3
    VERY_STRONG = 4


@dataclass
class Signal:
    symbol: str
    type: SignalType
    strength: SignalStrength
    price: Optional[float] = None
    quantity: Optional[float] = None
    stop_loss: Optional[float] = None
    take_profit: Optional[float] = None
    timestamp: datetime = None
    metadata: Dict[str, Any] = None
    
    def __post_init__(self):
        if self.timestamp is None:
            self.timestamp = datetime.utcnow()
        if self.metadata is None:
            self.metadata = {}


class BaseStrategy(ABC):
    """
    Abstract base class for trading strategies.
    All strategy implementations should inherit from this class.
    """
    
    def __init__(self, name: str, symbols: List[str], config: Dict[str, Any]):
        self.name = name
        self.symbols = symbols
        self.config = config
        self.logger = logging.getLogger(f"{__name__}.{name}")
        
        # Strategy state
        self.active = True
        self.positions: Dict[str, float] = {}  # symbol -> quantity
        self.indicators: Dict[str, Dict] = {}  # symbol -> indicator values
        self.signals_history: List[Signal] = []
        
        # Performance tracking
        self.trades_count = 0
        self.winning_trades = 0
        self.losing_trades = 0
        self.total_pnl = 0.0
        self.max_drawdown = 0.0
        self.peak_pnl = 0.0
    
    @abstractmethod
    async def generate_signals(self, symbol: str, data: List[List[float]]) -> List[Signal]:
        """
        Generate trading signals based on market data.
        
        Args:
            symbol: Trading pair symbol
            data: OHLCV data [[timestamp, open, high, low, close, volume], ...]
            
        Returns:
            List of Signal objects
        """
        pass
    
    @abstractmethod
    async def calculate_indicators(self, symbol: str, data: List[List[float]]) -> Dict[str, float]:
        """
        Calculate technical indicators for the strategy.
        
        Args:
            symbol: Trading pair symbol
            data: OHLCV data
            
        Returns:
            Dictionary of indicator values
        """
        pass
    
    async def update_position(self, symbol: str, quantity: float, pnl: float):
        """Update strategy position and track performance."""
        self.positions[symbol] = quantity
        
        # Update performance metrics
        self.total_pnl += pnl
        if pnl > 0:
            self.winning_trades += 1
        else:
            self.losing_trades += 1
        
        if self.total_pnl > self.peak_pnl:
            self.peak_pnl = self.total_pnl
        
        drawdown = self.peak_pnl - self.total_pnl
        if drawdown > self.max_drawdown:
            self.max_drawdown = drawdown
    
    def get_performance_metrics(self) -> Dict[str, Any]:
        """Get strategy performance metrics."""
        total_trades = self.winning_trades + self.losing_trades
        win_rate = self.winning_trades / total_trades if total_trades > 0 else 0
        
        return {
            'strategy_name': self.name,
            'active': self.active,
            'symbols': self.symbols,
            'total_trades': total_trades,
            'winning_trades': self.winning_trades,
            'losing_trades': self.losing_trades,
            'win_rate': win_rate,
            'total_pnl': self.total_pnl,
            'max_drawdown': self.max_drawdown,
            'current_positions': self.positions,
            'last_signal': self.signals_history[-1].timestamp if self.signals_history else None
        }
    
    def validate_signal(self, signal: Signal) -> bool:
        """Validate signal before execution."""
        # Check if strategy is active
        if not self.active:
            return False
        
        # Check signal age (should be recent)
        if (datetime.utcnow() - signal.timestamp).seconds > 60:
            return False
        
        # Check if we already have a position in the opposite direction
        current_position = self.positions.get(signal.symbol, 0)
        if signal.type == SignalType.BUY and current_position < 0:
            return True  # Allow to close short
        if signal.type == SignalType.SELL and current_position > 0:
            return True  # Allow to close long
        if signal.type == SignalType.CLOSE:
            return current_position != 0
        
        # Additional validation can be added here
        return True
    
    async def on_fill(self, symbol: str, side: str, quantity: float, price: float):
        """Handle order fill event."""
        self.logger.info(f"Strategy {self.name}: Filled {side} {quantity} {symbol} at {price}")
    
    async def on_error(self, error: Exception):
        """Handle error events."""
        self.logger.error(f"Strategy {self.name} error: {error}")
    
    def activate(self):
        """Activate the strategy."""
        self.active = True
        self.logger.info(f"Strategy {self.name} activated")
    
    def deactivate(self):
        """Deactivate the strategy."""
        self.active = False
        self.logger.info(f"Strategy {self.name} deactivated")
