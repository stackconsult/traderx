"""
Strategy validation module for calculating Information Coefficient (IC)
and Deflated Sharpe Ratio (DSR) to assess strategy performance.
"""

import logging
import numpy as np
import pandas as pd
from typing import List, Dict, Tuple, Optional
from datetime import datetime, timedelta
from dataclasses import dataclass
from scipy import stats

from ..core.models import Position


@dataclass
class StrategyMetrics:
    """Container for strategy performance metrics."""
    information_coefficient: float
    ic_p_value: float
    deflated_sharpe_ratio: float
    sharpe_ratio: float
    skewness: float
    kurtosis: float
    hurst_exponent: float
    is_valid: bool
    termination_reason: Optional[str] = None


class StrategyValidator:
    """
    Validates strategy performance using IC and DSR calculations.
    
    IC measures correlation between signals and future returns.
    DSR adjusts Sharpe ratio for skewness and kurtosis.
    """
    
    def __init__(self, 
                 min_ic: float = 0.05,
                 min_dsr: float = 0.5,
                 lookforward_days: int = 21,
                 min_samples: int = 30,
                 min_hurst: float = 0.5):
        """
        Initialize validator with thresholds.
        
        Args:
            min_ic: Minimum Information Coefficient (default: 0.05)
            min_dsr: Minimum Deflated Sharpe Ratio (default: 0.5)
            lookforward_days: Days to look forward for returns (default: 21)
            min_samples: Minimum samples for statistical significance
            min_hurst: Minimum Hurst exponent for trending regime (default: 0.5)
        """
        self.logger = logging.getLogger(__name__)
        self.min_ic = min_ic
        self.min_dsr = min_dsr
        self.lookforward_days = lookforward_days
        self.min_samples = min_samples
        self.min_hurst = min_hurst
        
        # Store signal-return pairs
        self.signal_history: List[Dict] = []
    
    def add_signal(self, 
                   timestamp: datetime, 
                   signal_strength: float, 
                   symbol: str,
                   signal_type: str = 'prediction'):
        """
        Add a signal to the history for later IC calculation.
        
        Args:
            timestamp: When signal was generated
            signal_strength: Signal value (-1 to 1 or similar)
            symbol: Trading symbol
            signal_type: Type of signal (prediction, strength, etc.)
        """
        self.signal_history.append({
            'timestamp': timestamp,
            'signal': signal_strength,
            'symbol': symbol,
            'type': signal_type,
            'forward_return': None  # To be filled later
        })
        
        # Keep only recent signals (last 2 years)
        cutoff = datetime.utcnow() - timedelta(days=730)
        self.signal_history = [s for s in self.signal_history 
                              if s['timestamp'] > cutoff]
    
    def add_return(self, timestamp: datetime, symbol: str, return_value: float):
        """
        Add a return value and match it with corresponding signal.
        
        Args:
            timestamp: Return period start
            symbol: Trading symbol
            return_value: Return over lookforward period
        """
        # Find matching signal from lookforward_days ago
        signal_time = timestamp - timedelta(days=self.lookforward_days)
        
        for signal in self.signal_history:
            if (signal['symbol'] == symbol and 
                signal['timestamp'] >= signal_time - timedelta(hours=1) and
                signal['timestamp'] <= signal_time + timedelta(hours=1) and
                signal['forward_return'] is None):
                signal['forward_return'] = return_value
                break
    
    def calculate_ic(self) -> Tuple[float, float]:
        """
        Calculate Information Coefficient.
        
        Returns:
            Tuple of (IC value, p-value)
        """
        # Extract signal-return pairs
        pairs = [(s['signal'], s['forward_return']) 
                for s in self.signal_history 
                if s['forward_return'] is not None]
        
        if len(pairs) < self.min_samples:
            self.logger.warning(f"Insufficient samples for IC: {len(pairs)} < {self.min_samples}")
            return 0.0, 1.0
        
        signals, returns = zip(*pairs)
        
        # Calculate Pearson correlation
        ic, p_value = stats.pearsonr(signals, returns)
        
        self.logger.info(f"IC calculated: {ic:.4f} (p-value: {p_value:.4f}) from {len(pairs)} samples")
        
        return ic, p_value
    
    def calculate_sharpe_ratio(self, returns: List[float]) -> float:
        """
        Calculate traditional Sharpe ratio.
        
        Args:
            returns: List of return values
            
        Returns:
            Sharpe ratio (annualized)
        """
        if len(returns) < 2:
            return 0.0
        
        returns_array = np.array(returns)
        
        # Annualized Sharpe (assuming daily returns)
        sharpe = np.mean(returns_array) / np.std(returns_array) * np.sqrt(252)
        
        return sharpe
    
    def calculate_deflated_sharpe_ratio(self, returns: List[float]) -> float:
        """
        Calculate Deflated Sharpe Ratio adjusting for skewness and kurtosis.
        
        Args:
            returns: List of return values
            
        Returns:
            Deflated Sharpe Ratio
        """
        if len(returns) < 4:
            return 0.0
        
        returns_array = np.array(returns)
        
        # Calculate moments
        mean = np.mean(returns_array)
        std = np.std(returns_array)
        skew = stats.skew(returns_array)
        kurt = stats.kurtosis(returns_array, fisher=False)  # Pearson's kurtosis
        
        # Traditional Sharpe
        sharpe = mean / std * np.sqrt(252) if std > 0 else 0.0
        
        # Deflation factor based on skewness and kurtosis
        # Using the method from López de Prado (2020)
        n = len(returns)
        
        if std > 0:
            # Third moment adjustment
            skew_adj = (skew / 6) * sharpe
            
            # Fourth moment adjustment
            kurt_adj = ((kurt - 3) / 24) * sharpe ** 2
            
            # Deflated Sharpe
            dsr = sharpe * (1 - skew_adj - kurt_adj)
        else:
            dsr = 0.0
        
        return max(dsr, 0.0)  # Cannot be negative
    
    def calculate_hurst_exponent(self, price_series: List[float]) -> float:
        """
        Calculate Hurst exponent using R/S analysis.
        
        H < 0.5: Mean-reverting series
        H = 0.5: Random walk
        H > 0.5: Trending/persistent series
        
        Args:
            price_series: List of price values
            
        Returns:
            Hurst exponent value
        """
        if len(price_series) < 100:
            self.logger.warning("Insufficient data for Hurst calculation, returning 0.5")
            return 0.5
        
        # Convert to log returns
        log_returns = []
        for i in range(1, len(price_series)):
            if price_series[i] > 0 and price_series[i-1] > 0:
                log_returns.append(np.log(price_series[i] / price_series[i-1]))
        
        if len(log_returns) < 50:
            return 0.5
        
        log_returns = np.array(log_returns)
        
        # Calculate R/S for different time windows
        max_window = len(log_returns) // 4
        window_sizes = []
        rs_values = []
        
        for window in range(10, max_window, 5):
            # Split into chunks
            chunks = len(log_returns) // window
            if chunks < 2:
                continue
            
            rs_chunk = []
            for i in range(chunks):
                chunk = log_returns[i*window:(i+1)*window]
                
                # Calculate cumulative deviation
                mean = np.mean(chunk)
                cum_dev = np.cumsum(chunk - mean)
                
                # Range
                R = np.max(cum_dev) - np.min(cum_dev)
                
                # Standard deviation
                S = np.std(chunk)
                
                if S > 0:
                    rs_chunk.append(R / S)
            
            if rs_chunk:
                window_sizes.append(window)
                rs_values.append(np.mean(rs_chunk))
        
        if len(window_sizes) < 2:
            return 0.5
        
        # Log-log regression to find H
        log_window = np.log(window_sizes)
        log_rs = np.log(rs_values)
        
        # Fit line y = H * x + c
        H, c = np.polyfit(log_window, log_rs, 1)
        
        return H
    
    def calculate_strategy_metrics(self, returns: List[float], price_series: Optional[List[float]] = None) -> StrategyMetrics:
        """
        Calculate all strategy metrics and determine validity.
        
        Args:
            returns: List of strategy returns
            price_series: List of price values for Hurst calculation
            
        Returns:
            StrategyMetrics object with all calculations
        """
        # Calculate IC
        ic, ic_p_value = self.calculate_ic()
        
        # Calculate Sharpe ratios
        sharpe = self.calculate_sharpe_ratio(returns)
        dsr = self.calculate_deflated_sharpe_ratio(returns)
        
        # Calculate higher moments
        if len(returns) > 2:
            returns_array = np.array(returns)
            skewness = stats.skew(returns_array)
            kurtosis = stats.kurtosis(returns_array, fisher=False)
        else:
            skewness = 0.0
            kurtosis = 3.0  # Normal distribution
        
        # Calculate Hurst exponent if price series provided
        if price_series:
            hurst = self.calculate_hurst_exponent(price_series)
        else:
            hurst = 0.5  # Default to random walk
        
        # Determine validity
        is_valid = True
        termination_reason = None
        
        if ic < self.min_ic:
            is_valid = False
            termination_reason = f"IC {ic:.4f} below minimum {self.min_ic}"
        
        elif dsr < self.min_dsr:
            is_valid = False
            termination_reason = f"DSR {dsr:.4f} below minimum {self.min_dsr}"
        
        elif hurst < self.min_hurst:
            is_valid = False
            termination_reason = f"Hurst {hurst:.4f} below minimum {self.min_hurst} (mean-reverting regime)"
        
        return StrategyMetrics(
            information_coefficient=ic,
            ic_p_value=ic_p_value,
            deflated_sharpe_ratio=dsr,
            sharpe_ratio=sharpe,
            skewness=skewness,
            kurtosis=kurtosis,
            hurst_exponent=hurst,
            is_valid=is_valid,
            termination_reason=termination_reason
        )
    
    def should_terminate_strategy(self, returns: List[float], price_series: Optional[List[float]] = None) -> bool:
        """
        Determine if strategy should be terminated based on metrics.
        
        Args:
            returns: List of recent strategy returns
            price_series: List of price values for Hurst calculation
            
        Returns:
            True if strategy should be terminated
        """
        metrics = self.calculate_strategy_metrics(returns, price_series)
        
        if not metrics.is_valid:
            self.logger.warning(f"Strategy termination recommended: {metrics.termination_reason}")
            self.logger.warning(f"IC: {metrics.information_coefficient:.4f}, DSR: {metrics.deflated_sharpe_ratio:.4f}, H: {metrics.hurst_exponent:.4f}")
            return True
        
        return False
    
    def get_validation_report(self, returns: List[float], price_series: Optional[List[float]] = None) -> Dict:
        """
        Generate comprehensive validation report.
        
        Args:
            returns: List of strategy returns
            price_series: List of price values for Hurst calculation
            
        Returns:
            Dictionary with all metrics and recommendations
        """
        metrics = self.calculate_strategy_metrics(returns, price_series)
        
        report = {
            'timestamp': datetime.utcnow().isoformat(),
            'metrics': {
                'information_coefficient': metrics.information_coefficient,
                'ic_p_value': metrics.ic_p_value,
                'sharpe_ratio': metrics.sharpe_ratio,
                'deflated_sharpe_ratio': metrics.deflated_sharpe_ratio,
                'skewness': metrics.skewness,
                'kurtosis': metrics.kurtosis,
                'hurst_exponent': metrics.hurst_exponent,
                'regime': 'trending' if metrics.hurst_exponent > 0.5 else 'mean_reverting'
            },
            'thresholds': {
                'min_ic': self.min_ic,
                'min_dsr': self.min_dsr,
                'min_hurst': self.min_hurst
            },
            'validation': {
                'is_valid': metrics.is_valid,
                'termination_reason': metrics.termination_reason,
                'recommendation': 'CONTINUE' if metrics.is_valid else 'TERMINATE'
            },
            'sample_size': len([s for s in self.signal_history if s['forward_return'] is not None])
        }
        
        return report
