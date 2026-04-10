"""Information Coefficient and Hurst Exponent Filters
Used for strategy performance evaluation and regime detection.
"""

import numpy as np
from scipy import stats
from typing import List, Tuple, Optional
import structlog

logger = structlog.get_logger(__name__)


class ICHFilters:
    """
    Information Coefficient (IC) and Hurst Exponent (H) filters.
    
    IC: Measures predictive power of signals (correlation with returns)
    H: Measures market regime (mean-reverting vs trending)
    """
    
    def __init__(self, ic_window: int = 21, hurst_window: int = 100):
        self.ic_window = ic_window
        self.hurst_window = hurst_window
        
    def calculate_ic(self, 
                    predictions: np.ndarray,
                    returns: np.ndarray) -> Tuple[float, float]:
        """
        Calculate Information Coefficient (Spearman correlation).
        
        Args:
            predictions: Strategy predictions/signals
            returns: Actual returns
            
        Returns:
            (ic_value, p_value)
        """
        if len(predictions) != len(returns):
            min_len = min(len(predictions), len(returns))
            predictions = predictions[-min_len:]
            returns = returns[-min_len:]
            
        if len(predictions) < 5:
            return 0.0, 1.0
            
        # Spearman rank correlation
        ic, p_value = stats.spearmanr(predictions, returns)
        
        return float(ic), float(p_value)
    
    def calculate_rolling_ic(self,
                          predictions: np.ndarray,
                          returns: np.ndarray) -> List[float]:
        """Calculate rolling IC over window."""
        rolling_ics = []
        
        for i in range(len(predictions) - self.ic_window + 1):
            window_pred = predictions[i:i+self.ic_window]
            window_ret = returns[i:i+self.ic_window]
            ic, _ = self.calculate_ic(window_pred, window_ret)
            rolling_ics.append(ic)
            
        return rolling_ics
    
    def calculate_hurst(self, prices: np.ndarray) -> float:
        """
        Calculate Hurst Exponent for regime detection.
        
        H < 0.5: Mean-reverting
        H = 0.5: Random walk
        H > 0.5: Trending
        
        Uses R/S analysis.
        """
        if len(prices) < self.hurst_window:
            return 0.5  # Default to random walk
            
        # Use recent window
        series = prices[-self.hurst_window:]
        
        # R/S analysis
        lags = range(2, min(100, len(series)//4))
        tau = [np.std(np.subtract(series[lag:], series[:-lag])) for lag in lags]
        
        # Linear regression on log-log plot
        log_lags = np.log(list(lags))
        log_tau = np.log(tau)
        
        # Fit line
        slope, intercept, r_value, p_value, std_err = stats.linregress(log_lags, log_tau)
        
        hurst = slope / 2.0
        
        # Clamp to valid range
        return float(np.clip(hurst, 0.0, 1.0))
    
    def detect_regime(self, hurst: float) -> str:
        """Detect market regime from Hurst exponent."""
        if hurst < 0.4:
            return "mean_reverting"
        elif hurst > 0.6:
            return "trending"
        else:
            return "random_walk"
    
    def should_hibernate_strategy(self,
                                 rolling_ics: List[float],
                                 threshold: float = 0.05,
                                 window_hours: int = 48) -> Tuple[bool, str]:
        """
        Determine if strategy should hibernate due to poor IC.
        
        Per Phase 5 spec: IC < 0.05 for 48h window triggers hibernation.
        """
        if len(rolling_ics) == 0:
            return False, "insufficient_data"
            
        # Check recent window
        recent_ics = rolling_ics[-window_hours:] if len(rolling_ics) >= window_hours else rolling_ics
        
        avg_ic = np.mean(recent_ics)
        
        if avg_ic < threshold:
            return True, f"IC_decay: {avg_ic:.4f} < {threshold}"
            
        return False, f"IC_healthy: {avg_ic:.4f}"
    
    def evaluate_strategy(self,
                         predictions: np.ndarray,
                         returns: np.ndarray,
                         prices: np.ndarray) -> dict:
        """
        Comprehensive strategy evaluation using IC/H filters.
        """
        # Calculate metrics
        ic, ic_pvalue = self.calculate_ic(predictions, returns)
        rolling_ics = self.calculate_rolling_ic(predictions, returns)
        hurst = self.calculate_hurst(prices)
        regime = self.detect_regime(hurst)
        
        # Hibernation check
        should_hibernate, hibernate_reason = self.should_hibernate_strategy(rolling_ics)
        
        return {
            "ic": ic,
            "ic_pvalue": ic_pvalue,
            "ic_rolling_avg": np.mean(rolling_ics) if rolling_ics else 0,
            "hurst": hurst,
            "regime": regime,
            "should_hibernate": should_hibernate,
            "hibernate_reason": hibernate_reason,
            "recommendation": "continue" if not should_hibernate else "hibernate"
        }
