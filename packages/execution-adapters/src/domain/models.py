"""
Domain models for liquidity adapters.
Unified representations across all venues.
"""

from dataclasses import dataclass, field
from typing import Optional, Dict, Any, List
from enum import Enum
from datetime import datetime
import uuid


class OrderSide(Enum):
    """Order side."""
    BUY = "BUY"
    SELL = "SELL"


class OrderType(Enum):
    """Order type."""
    MARKET = "MARKET"
    LIMIT = "LIMIT"
    STOP = "STOP"
    STOP_LIMIT = "STOP_LIMIT"


class TimeInForce(Enum):
    """Time in force."""
    IOC = "IOC"  # Immediate or Cancel
    GTC = "GTC"  # Good Till Cancelled
    FOK = "FOK"  # Fill or Kill
    DAY = "DAY"  # Day Order


class OrderStatus(Enum):
    """Order status."""
    NEW = "NEW"
    PENDING = "PENDING"
    PARTIALLY_FILLED = "PARTIALLY_FILLED"
    FILLED = "FILLED"
    CANCELLED = "CANCELLED"
    REJECTED = "REJECTED"
    EXPIRED = "EXPIRED"


@dataclass
class UnifiedOrder:
    """Unified order representation across all venues."""
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    tenant_id: str = ""
    symbol: str = ""
    side: OrderSide = OrderSide.BUY
    quantity: float = 0.0
    order_type: OrderType = OrderType.MARKET
    price: Optional[float] = None
    stop_price: Optional[float] = None
    time_in_force: TimeInForce = TimeInForce.IOC
    client_order_id: Optional[str] = None
    metadata: Dict[str, Any] = field(default_factory=dict)
    created_at: datetime = field(default_factory=datetime.utcnow)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "id": self.id,
            "tenant_id": self.tenant_id,
            "symbol": self.symbol,
            "side": self.side.value,
            "quantity": self.quantity,
            "order_type": self.order_type.value,
            "price": self.price,
            "stop_price": self.stop_price,
            "time_in_force": self.time_in_force.value,
            "client_order_id": self.client_order_id,
            "metadata": self.metadata,
            "created_at": self.created_at.isoformat()
        }


@dataclass
class ExecutionResult:
    """Order execution result."""
    order_id: str
    venue_order_id: Optional[str]
    status: OrderStatus
    filled_quantity: float = 0.0
    remaining_quantity: float = 0.0
    average_price: Optional[float] = None
    execution_venue: str = ""
    fees: Dict[str, float] = field(default_factory=dict)
    timestamp: datetime = field(default_factory=datetime.utcnow)
    error_message: Optional[str] = None
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class Position:
    """Position representation."""
    symbol: str
    side: OrderSide
    quantity: float
    entry_price: float
    current_price: Optional[float] = None
    unrealized_pnl: Optional[float] = None
    realized_pnl: float = 0.0
    venue: str = ""
    timestamp: datetime = field(default_factory=datetime.utcnow)


@dataclass
class AccountInfo:
    """Account information."""
    account_id: str
    venue: str
    balance: Dict[str, float] = field(default_factory=dict)
    positions: List[Position] = field(default_factory=list)
    margin_available: float = 0.0
    margin_used: float = 0.0
    buying_power: float = 0.0
    last_updated: datetime = field(default_factory=datetime.utcnow)


@dataclass
class VenueInfo:
    """Venue information and capabilities."""
    name: str
    venue_type: str  # "FIX", "WebSocket", "REST"
    supported_symbols: List[str] = field(default_factory=list)
    supported_order_types: List[OrderType] = field(default_factory=list)
    supported_tif: List[TimeInForce] = field(default_factory=list)
    min_order_size: float = 0.0
    max_order_size: float = float('inf')
    tick_size: float = 0.01
    connection_status: bool = False
    latency_ms: float = 0.0
    rate_limits: Dict[str, Any] = field(default_factory=dict)


@dataclass
class MarketData:
    """Market data tick."""
    symbol: str
    venue: str
    bid: Optional[float] = None
    ask: Optional[float] = None
    bid_size: Optional[float] = None
    ask_size: Optional[float] = None
    last: Optional[float] = None
    last_size: Optional[float] = None
    volume: Optional[float] = None
    timestamp: datetime = field(default_factory=datetime.utcnow)


@dataclass
class LiquidityRequest:
    """Request for liquidity execution."""
    order: UnifiedOrder
    venues: List[str] = field(default_factory=list)
    routing_strategy: str = "BEST_PRICE"  # BEST_PRICE, FASTEST, SPLIT
    max_slippage: float = 0.01  # 1%
    timeout_ms: int = 5000


@dataclass
class LiquidityResponse:
    """Response from liquidity execution."""
    request_id: str
    status: str  # "SUCCESS", "PARTIAL", "FAILED", "TIMEOUT"
    executions: List[ExecutionResult] = field(default_factory=list)
    total_filled: float = 0.0
    average_price: Optional[float] = None
    total_fees: float = 0.0
    latency_ms: float = 0.0
    error_message: Optional[str] = None
