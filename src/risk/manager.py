import logging
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
from datetime import datetime, timedelta
from enum import Enum

from ..core.models import Order, Position, OrderSide, OrderStatus


class RiskLevel(Enum):
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"


@dataclass
class RiskLimit:
    name: str
    value: float
    current: float = 0.0
    level: RiskLevel = RiskLevel.LOW
    breached: bool = False


@dataclass
class RiskMetrics:
    total_exposure: float
    daily_pnl: float
    max_drawdown: float
    var_95: float  # Value at Risk 95%
    sharpe_ratio: float
    win_rate: float
    profit_factor: float


class RiskManager:
    """
    Risk management system that monitors and controls trading risk.
    Implements position limits, drawdown controls, and real-time risk monitoring.
    """
    
    def __init__(self, 
                 max_position_size: float = 10000.0,
                 max_daily_loss: float = 1000.0,
                 max_drawdown: float = 0.20,
                 max_leverage: float = 3.0,
                 max_orders_per_minute: int = 10):
        self.logger = logging.getLogger(__name__)
        self.max_position_size = max_position_size
        self.max_daily_loss = max_daily_loss
        self.max_drawdown = max_drawdown
        self.max_leverage = max_leverage
        self.max_orders_per_minute = max_orders_per_minute
        
        # State tracking
        self.positions: Dict[str, Position] = {}
        self.open_orders: List[Order] = []
        self.daily_pnl = 0.0
        self.total_exposure = 0.0
        self.current_balance = 0.0
        self.daily_start_balance = 0.0
        self.peak_balance = 0.0
        
        # Risk limits tracking
        self.limits = {
            'position_size': RiskLimit('Position Size', max_position_size, 0.0),
            'daily_loss': RiskLimit('Daily Loss', max_daily_loss, 0.0),
            'drawdown': RiskLimit('Max Drawdown', max_drawdown, 0.0),
            'leverage': RiskLimit('Leverage', max_leverage, 0.0),
            'order_frequency': RiskLimit('Order Frequency', max_orders_per_minute, 0.0)
        }
        
        # Circuit breaker state
        self.circuit_breaker_active = False
        self.circuit_breaker_reason = None
        self.circuit_breaker_time = None
        
        # VPIN tracking for toxicity detection
        self.vpin_buckets = []  # List of volume buckets
        self.vpin_window_size = 50  # Number of buckets to consider
        self.vpin_threshold = 0.75  # Trigger circuit breaker above this
        self.bucket_volume_target = 10000  # Target volume per bucket
        self.current_bucket_volume = 0.0
        self.current_bucket_abs_price_change = 0.0
        
        # Trade and PnL history
        self.trade_history = []
        self.pnl_history = []
        
    async def validate_order(self, order: Order, positions: Dict[str, Position]) -> bool:
        """
        Validate an order against risk limits before submission.
        Returns True if order is allowed, False otherwise.
        """
        # Check circuit breaker
        if self.circuit_breaker_active:
            self.logger.warning(f"Order rejected - Circuit breaker active: {self.circuit_breaker_reason}")
            return False
        
        # Check position size limit
        position = positions.get(order.symbol, Position(order.symbol, 0, 0, 0))
        new_quantity = position.quantity
        
        if order.side == OrderSide.BUY:
            new_quantity += order.quantity
        else:
            new_quantity -= order.quantity
        
        if abs(new_quantity * order.price or 0) > self.limits['position_size'].value:
            self.logger.warning(f"Order rejected - Position size limit exceeded")
            return False
        
        # Check daily loss limit
        if self.daily_pnl < -self.limits['daily_loss'].value:
            self.logger.warning(f"Order rejected - Daily loss limit exceeded")
            self._activate_circuit_breaker("Daily loss limit exceeded")
            return False
        
        # Check leverage
        total_value = sum(
            abs(pos.quantity * pos.current_price) 
            for pos in positions.values()
        )
        if total_value > self.current_balance * self.limits['leverage'].value:
            self.logger.warning(f"Order rejected - Leverage limit exceeded")
            return False
        
        # Check order frequency (anti-manipulation)
        recent_orders = [
            o for o in self.open_orders 
            if (datetime.utcnow() - o.timestamp).seconds < 60
        ]
        if len(recent_orders) > 10:  # Max 10 orders per minute
            self.logger.warning(f"Order rejected - Order frequency limit exceeded")
            return False
        
        return True
    
    async def update_metrics(self, positions: Dict[str, Position], balance: float):
        """Update risk metrics based on current positions and balance."""
        self.current_balance = balance
        
        if self.daily_start_balance == 0:
            self.daily_start_balance = balance
            self.peak_balance = balance
        
        # Update daily PnL
        self.daily_pnl = balance - self.daily_start_balance
        
        # Update peak balance and drawdown
        if balance > self.peak_balance:
            self.peak_balance = balance
        
        current_drawdown = (self.peak_balance - balance) / self.peak_balance
        self.limits['drawdown'].current = current_drawdown
        
        # Check drawdown limit
        if current_drawdown > self.limits['drawdown'].value:
            self._activate_circuit_breaker("Maximum drawdown exceeded")
        
        # Calculate total exposure
        self.total_exposure = sum(
            abs(pos.quantity * pos.current_price) 
            for pos in positions.values()
        )
        
        # Update PnL history for VaR
        self.pnl_history.append(self.daily_pnl)
        if len(self.pnl_history) > 100:  # Keep last 100 days
            self.pnl_history.pop(0)
    
    def calculate_var(self, confidence: float = 0.95) -> float:
        """Calculate Value at Risk (VaR) using historical method."""
        if len(self.pnl_history) < 30:
            return 0.0
        
        sorted_pnl = sorted(self.pnl_history)
        index = int((1 - confidence) * len(sorted_pnl))
        return -sorted_pnl[index] if index < len(sorted_pnl) else 0.0
    
    def get_risk_metrics(self) -> RiskMetrics:
        """Get current risk metrics."""
        # Calculate Sharpe ratio (simplified)
        if len(self.pnl_history) < 2:
            sharpe_ratio = 0.0
        else:
            returns = [
                (self.pnl_history[i] - self.pnl_history[i-1]) / abs(self.pnl_history[i-1]) if self.pnl_history[i-1] != 0 else 0
                for i in range(1, len(self.pnl_history))
            ]
            avg_return = sum(returns) / len(returns) if returns else 0
            std_return = (sum((r - avg_return) ** 2 for r in returns) / len(returns)) ** 0.5 if returns else 0
            sharpe_ratio = avg_return / std_return if std_return > 0 else 0
        
        # Calculate win rate and profit factor
        winning_trades = sum(1 for t in self.trade_history if t['pnl'] > 0)
        total_trades = len(self.trade_history)
        win_rate = winning_trades / total_trades if total_trades > 0 else 0
        
        gross_profit = sum(t['pnl'] for t in self.trade_history if t['pnl'] > 0)
        gross_loss = abs(sum(t['pnl'] for t in self.trade_history if t['pnl'] < 0))
        profit_factor = gross_profit / gross_loss if gross_loss > 0 else 0
        
        return RiskMetrics(
            total_exposure=self.total_exposure,
            daily_pnl=self.daily_pnl,
            max_drawdown=self.limits['drawdown'].current,
            var_95=self.calculate_var(),
            sharpe_ratio=sharpe_ratio,
            win_rate=win_rate,
            profit_factor=profit_factor
        )
    
    def _activate_circuit_breaker(self, reason: str):
        """Activate circuit breaker to stop all trading."""
        self.circuit_breaker_active = True
        self.circuit_breaker_reason = reason
        self.circuit_breaker_time = datetime.utcnow()
        self.logger.critical(f"Circuit breaker activated: {reason}")
    
    def reset_circuit_breaker(self):
        """Reset circuit breaker (manual intervention required)."""
        self.circuit_breaker_active = False
        self.circuit_breaker_reason = None
        self.circuit_breaker_time = None
        self.logger.info("Circuit breaker reset")
    
    def add_trade(self, symbol: str, pnl: float, quantity: float, price: float):
        """Add a completed trade to history."""
        self.trade_history.append({
            'timestamp': datetime.utcnow(),
            'symbol': symbol,
            'pnl': pnl,
            'quantity': quantity,
            'price': price
        })
    
    def get_risk_report(self) -> Dict:
        """Generate comprehensive risk report."""
        metrics = self.get_risk_metrics()
        
        return {
            'timestamp': datetime.utcnow().isoformat(),
            'circuit_breaker': {
                'active': self.circuit_breaker_active,
                'reason': self.circuit_breaker_reason,
                'time': self.circuit_breaker_time.isoformat() if self.circuit_breaker_time else None
            },
            'limits': {
                name: {
                    'limit': limit.value,
                    'current': limit.current,
                    'utilization': limit.current / limit.value if limit.value > 0 else 0,
                    'breached': limit.breached
                }
                for name, limit in self.limits.items()
            },
            'metrics': {
                'total_exposure': metrics.total_exposure,
                'daily_pnl': metrics.daily_pnl,
                'max_drawdown': metrics.max_drawdown,
                'var_95': metrics.var_95,
                'sharpe_ratio': metrics.sharpe_ratio,
                'win_rate': metrics.win_rate,
                'profit_factor': metrics.profit_factor,
                'vpin': self.get_vpin()
            },
            'account': {
                'current_balance': self.current_balance,
                'daily_start_balance': self.daily_start_balance,
                'peak_balance': self.peak_balance,
                'daily_return': self.daily_pnl / self.daily_start_balance if self.daily_start_balance > 0 else 0
            }
        }
    
    def update_vpin(self, volume: float, price_change: float):
        """
        Update VPIN calculation with new volume and price data.
        
        Args:
            volume: Trade volume for this update
            price_change: Absolute price change for this update
        """
        # Add to current bucket
        self.current_bucket_volume += volume
        self.current_bucket_abs_price_change += abs(price_change)
        
        # Check if bucket is complete
        if self.current_bucket_volume >= self.bucket_volume_target:
            # Create new bucket
            bucket = {
                'volume': self.current_bucket_volume,
                'abs_price_change': self.current_bucket_abs_price_change,
                'timestamp': datetime.utcnow()
            }
            
            self.vpin_buckets.append(bucket)
            
            # Keep only recent buckets
            if len(self.vpin_buckets) > self.vpin_window_size:
                self.vpin_buckets = self.vpin_buckets[-self.vpin_window_size:]
            
            # Reset current bucket
            self.current_bucket_volume = 0.0
            self.current_bucket_abs_price_change = 0.0
            
            # Check VPIN toxicity
            self._check_vpin_toxicity()
    
    def _check_vpin_toxicity(self):
        """Check if VPIN indicates toxic order flow."""
        if len(self.vpin_buckets) < 10:  # Need minimum buckets
            return
        
        # Calculate VPIN
        total_volume = sum(b['volume'] for b in self.vpin_buckets)
        total_abs_price_change = sum(b['abs_price_change'] for b in self.vpin_buckets)
        
        if total_volume > 0:
            # VPIN is average absolute price change per unit volume
            vpin = total_abs_price_change / total_volume
            
            self.logger.info(f"VPIN calculated: {vpin:.6f}")
            
            # Check threshold
            if vpin > self.vpin_threshold:
                self.logger.critical(f"Toxic flow detected - VPIN {vpin:.6f} > {self.vpin_threshold}")
                self._activate_circuit_breaker(f"VPIN toxicity threshold exceeded: {vpin:.6f}")
    
    def get_vpin(self) -> float:
        """
        Get current VPIN value.
        
        Returns:
            Current VPIN or 0.0 if insufficient data
        """
        if len(self.vpin_buckets) < 10:
            return 0.0
        
        total_volume = sum(b['volume'] for b in self.vpin_buckets)
        total_abs_price_change = sum(b['abs_price_change'] for b in self.vpin_buckets)
        
        if total_volume > 0:
            return total_abs_price_change / total_volume
        
        return 0.0
