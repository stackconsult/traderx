import logging
from typing import Dict, List, Optional, Any
import numpy as np

from .base import BaseStrategy, Signal, SignalType, SignalStrength


class MovingAverageStrategy(BaseStrategy):
    """
    Simple moving average crossover strategy.
    Generates buy signals when fast MA crosses above slow MA,
    and sell signals when fast MA crosses below slow MA.
    """
    
    def __init__(self, name: str, symbols: List[str], config: Dict[str, Any]):
        super().__init__(name, symbols, config)
        
        # Strategy parameters
        self.fast_period = config.get('fast_period', 10)
        self.slow_period = config.get('slow_period', 30)
        self.timeframe = config.get('timeframe', '1m')
        
        # Strategy state
        self.fast_mavgs: Dict[str, List[float]] = {}
        self.slow_mavgs: Dict[str, List[float]] = {}
        self.last_signals: Dict[str, SignalType] = {}
        
        self.logger.info(f"MA Strategy initialized: fast={self.fast_period}, slow={self.slow_period}")
    
    async def calculate_indicators(self, symbol: str, data: List[List[float]]) -> Dict[str, float]:
        """
        Calculate moving averages for the given symbol.
        
        Args:
            symbol: Trading pair symbol
            data: OHLCV data [[timestamp, open, high, low, close, volume], ...]
            
        Returns:
            Dictionary with indicator values
        """
        if len(data) < self.slow_period:
            return {}
        
        # Extract closing prices
        closes = np.array([candle[4] for candle in data])
        
        # Calculate moving averages
        fast_ma = np.mean(closes[-self.fast_period:])
        slow_ma = np.mean(closes[-self.slow_period:])
        
        # Store history
        if symbol not in self.fast_mavgs:
            self.fast_mavgs[symbol] = []
            self.slow_mavgs[symbol] = []
        
        self.fast_mavgs[symbol].append(fast_ma)
        self.slow_mavgs[symbol].append(slow_ma)
        
        # Keep only recent values
        max_history = 100
        if len(self.fast_mavgs[symbol]) > max_history:
            self.fast_mavgs[symbol] = self.fast_mavgs[symbol][-max_history:]
            self.slow_mavgs[symbol] = self.slow_mavgs[symbol][-max_history:]
        
        return {
            'fast_ma': fast_ma,
            'slow_ma': slow_ma,
            'ma_diff': fast_ma - slow_ma,
            'ma_ratio': fast_ma / slow_ma if slow_ma > 0 else 0
        }
    
    async def generate_signals(self, symbol: str, data: List[List[float]]) -> List[Signal]:
        """
        Generate trading signals based on MA crossover.
        
        Args:
            symbol: Trading pair symbol
            data: OHLCV data
            
        Returns:
            List of Signal objects
        """
        signals = []
        
        # Calculate indicators
        indicators = await self.calculate_indicators(symbol, data)
        
        if not indicators:
            return signals
        
        fast_ma = indicators['fast_ma']
        slow_ma = indicators['slow_ma']
        current_price = data[-1][4]  # Last close price
        
        # Get previous MA values to detect crossover
        prev_fast = self.fast_mavgs[symbol][-2] if len(self.fast_mavgs[symbol]) > 1 else fast_ma
        prev_slow = self.slow_mavgs[symbol][-2] if len(self.slow_mavgs[symbol]) > 1 else slow_ma
        
        # Detect crossovers
        prev_diff = prev_fast - prev_slow
        curr_diff = fast_ma - slow_ma
        
        # Bullish crossover (fast crosses above slow)
        if prev_diff <= 0 and curr_diff > 0:
            # Check if we're not already in a long position
            current_position = self.positions.get(symbol, 0)
            if current_position <= 0:
                signal = Signal(
                    symbol=symbol,
                    type=SignalType.BUY,
                    strength=self._calculate_signal_strength(curr_diff, slow_ma),
                    price=current_price,
                    quantity=self._calculate_position_size(symbol, current_price),
                    stop_loss=current_price * 0.98,  # 2% stop loss
                    take_profit=current_price * 1.06,  # 6% take profit
                    metadata={
                        'fast_ma': fast_ma,
                        'slow_ma': slow_ma,
                        'ma_diff': curr_diff
                    }
                )
                signals.append(signal)
                self.last_signals[symbol] = SignalType.BUY
        
        # Bearish crossover (fast crosses below slow)
        elif prev_diff >= 0 and curr_diff < 0:
            # Check if we're in a long position
            current_position = self.positions.get(symbol, 0)
            if current_position > 0:
                signal = Signal(
                    symbol=symbol,
                    type=SignalType.SELL,
                    strength=self._calculate_signal_strength(abs(curr_diff), slow_ma),
                    price=current_price,
                    quantity=abs(current_position),
                    metadata={
                        'fast_ma': fast_ma,
                        'slow_ma': slow_ma,
                        'ma_diff': curr_diff
                    }
                )
                signals.append(signal)
                self.last_signals[symbol] = SignalType.SELL
        
        # Close signal if position is losing too much
        elif curr_diff < -slow_ma * 0.01:  # More than 1% divergence
            current_position = self.positions.get(symbol, 0)
            if current_position != 0:
                signal = Signal(
                    symbol=symbol,
                    type=SignalType.CLOSE,
                    strength=SignalStrength.MODERATE,
                    price=current_price,
                    quantity=abs(current_position),
                    metadata={
                        'reason': 'MA_divergence',
                        'fast_ma': fast_ma,
                        'slow_ma': slow_ma
                    }
                )
                signals.append(signal)
                self.last_signals[symbol] = SignalType.CLOSE
        
        return signals
    
    def _calculate_signal_strength(self, diff: float, slow_ma: float) -> SignalStrength:
        """Calculate signal strength based on MA difference."""
        diff_percent = abs(diff) / slow_ma
        
        if diff_percent > 0.02:  # > 2% difference
            return SignalStrength.VERY_STRONG
        elif diff_percent > 0.01:  # > 1% difference
            return SignalStrength.STRONG
        elif diff_percent > 0.005:  # > 0.5% difference
            return SignalStrength.MODERATE
        else:
            return SignalStrength.WEAK
    
    def _calculate_position_size(self, symbol: str, price: float) -> float:
        """Calculate position size based on risk management."""
        # Simple position sizing - risk 1% per trade
        # This should be integrated with the risk manager for proper sizing
        risk_amount = 100.0  # Default risk amount
        position_size = risk_amount / price
        
        # Round to appropriate precision
        if 'BTC' in symbol:
            position_size = round(position_size, 6)
        elif 'ETH' in symbol:
            position_size = round(position_size, 5)
        else:
            position_size = round(position_size, 2)
        
        return max(position_size, 0.001)  # Minimum position size
