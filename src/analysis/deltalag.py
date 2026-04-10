"""
DeltaLag Cross-Attention Mechanism with SugaFormer Logic

Analyzes cross-correlations between markets to identify lead-lag relationships.
Enhanced with Information Coefficient (IC) and Hurst exponent (H) persistence filters.
"""

import logging
import numpy as np
import pandas as pd
import json
from typing import Dict, List, Tuple, Optional, NamedTuple
from datetime import datetime, timedelta
from dataclasses import dataclass, asdict
from pathlib import Path
from scipy import signal
from scipy.stats import pearsonr

# Import StrategyValidator for IC and Hurst calculations
try:
    from ..strategies.validator import StrategyValidator
except ImportError:
    # Fallback for testing
    class StrategyValidator:
        def calculate_hurst_exponent(self, returns):
            """Mock Hurst exponent calculation."""
            return 0.65

logger = logging.getLogger(__name__)


@dataclass
class LeadLagResult:
    """Results from lead-lag analysis."""
    leader: str
    follower: str
    lag_hours: int
    correlation: float
    significance: float
    confidence: float
    timestamp: datetime
    information_coefficient: float = 0.0  # IC for signal quality
    hurst_exponent: float = 0.5  # H for regime detection
    activation_score: float = 0.0  # Combined activation score


@dataclass
class AttentionWeights:
    """Cross-attention weights between market pairs."""
    leader: str
    follower: str
    attention_score: float
    time_delta: float  # Learnable lag in hours
    ic: float  # Information Coefficient
    hurst: float  # Hurst exponent
    last_updated: datetime


class DeltaLagAnalyzer:
    """
    Enhanced DeltaLag analyzer with SugaFormer logic.
    
    Integrates Information Coefficient and Hurst exponent filters
    for adaptive leader-lagger pair selection.
    """
    
    def __init__(self, 
                 max_lag_hours: int = 24,
                 min_correlation: float = 0.3,
                 confidence_threshold: float = 0.95,
                 ic_threshold: float = 0.05,  # Minimum IC for activation
                 hurst_threshold: float = 0.5,  # Minimum H for trending regime
                 weights_file: Optional[Path] = None):
        """
        Initialize enhanced DeltaLag analyzer.
        
        Args:
            max_lag_hours: Maximum lag to analyze (default: 24 hours)
            min_correlation: Minimum correlation to consider (default: 0.3)
            confidence_threshold: Statistical confidence threshold (default: 0.95)
            ic_threshold: Minimum Information Coefficient for activation
            hurst_threshold: Minimum Hurst exponent for trending regime
            weights_file: Path to save/load attention weights
        """
        self.max_lag_hours = max_lag_hours
        self.min_correlation = min_correlation
        self.confidence_threshold = confidence_threshold
        self.ic_threshold = ic_threshold
        self.hurst_threshold = hurst_threshold
        self.weights_file = weights_file or Path("deltalag-weights.json")
        
        # Data storage
        self.price_data: Dict[str, pd.DataFrame] = {}
        self.returns_data: Dict[str, pd.Series] = {}
        self.attention_weights: Dict[str, AttentionWeights] = {}
        
        # Strategy validator for IC and Hurst calculations
        self.validator = StrategyValidator()
        
        # Cache for performance
        self._cache: Dict[str, LeadLagResult] = {}
        self._cache_ttl = timedelta(hours=1)
        
        # Load existing weights if available
        self.load_attention_weights()
        
    def load_attention_weights(self):
        """Load attention weights from file."""
        if self.weights_file.exists():
            try:
                with open(self.weights_file, 'r') as f:
                    data = json.load(f)
                    
                for pair_data in data.get('pairs', []):
                    weights = AttentionWeights(
                        leader=pair_data['leader'],
                        follower=pair_data['follower'],
                        attention_score=pair_data['attention_score'],
                        time_delta=pair_data['time_delta'],
                        ic=pair_data['ic'],
                        hurst=pair_data['hurst'],
                        last_updated=datetime.fromisoformat(pair_data['last_updated'])
                    )
                    key = f"{weights.leader}->{weights.follower}"
                    self.attention_weights[key] = weights
                    
                logger.info(f"Loaded {len(self.attention_weights)} attention weight pairs")
                
            except Exception as e:
                logger.warning(f"Failed to load attention weights: {e}")
                
    def save_attention_weights(self):
        """Save attention weights to file."""
        try:
            data = {
                'timestamp': datetime.utcnow().isoformat(),
                'pairs': []
            }
            
            for weights in self.attention_weights.values():
                data['pairs'].append({
                    'leader': weights.leader,
                    'follower': weights.follower,
                    'attention_score': weights.attention_score,
                    'time_delta': weights.time_delta,
                    'ic': weights.ic,
                    'hurst': weights.hurst,
                    'last_updated': weights.last_updated.isoformat()
                })
                
            with open(self.weights_file, 'w') as f:
                json.dump(data, f, indent=2)
                
            logger.info(f"Saved {len(self.attention_weights)} attention weight pairs")
            
        except Exception as e:
            logger.error(f"Failed to save attention weights: {e}")
            
    def add_price_data(self, 
                      symbol: str, 
                      timestamps: List[datetime], 
                      prices: List[float],
                      volumes: Optional[List[float]] = None):
        """
        Add price data for a symbol.
        
        Args:
            symbol: Market symbol (e.g., 'BTC/USDT')
            timestamps: List of timestamps
            prices: List of price values
            volumes: Optional list of volumes
        """
        df = pd.DataFrame({
            'timestamp': timestamps,
            'price': prices
        })
        
        # Calculate returns using pandas diff
        df['returns'] = np.log(df['price']).diff()
        
        if volumes:
            df['volume'] = volumes
        
        # Remove NaN values
        df = df.dropna()
        
        # Set timestamp as index
        df.set_index('timestamp', inplace=True)
        
        # Resample to hourly data for consistency
        agg_dict = {
            'price': 'last',
            'returns': 'sum'
        }
        if volumes:
            agg_dict['volume'] = 'sum'
        
        df = df.resample('1h').agg(agg_dict).dropna()
        
        self.price_data[symbol] = df
        logger.info(f"Added price data for {symbol}: {len(df)} hourly points")
    
    def analyze_lead_lag(self, 
                        symbol1: str, 
                        symbol2: str) -> Optional[LeadLagResult]:
        """
        Analyze lead-lag relationship between two symbols.
        
        Args:
            symbol1: First symbol
            symbol2: Second symbol
            
        Returns:
            LeadLagResult if significant relationship found
        """
        # Check cache first
        cache_key = f"{symbol1}-{symbol2}"
        if cache_key in self._cache:
            cached = self._cache[cache_key]
            if datetime.utcnow() - cached.timestamp < self._cache_ttl:
                return cached
        
        # Get data
        if symbol1 not in self.price_data or symbol2 not in self.price_data:
            logger.warning(f"Missing data for {symbol1} or {symbol2}")
            return None
        
        # Align data on common timestamps
        df1 = self.price_data[symbol1]
        df2 = self.price_data[symbol2]
        
        # Find common timestamps
        common_times = df1.index.intersection(df2.index)
        if len(common_times) < 100:  # Need sufficient data
            logger.warning(f"Insufficient overlapping data: {len(common_times)}")
            return None
        
        returns1 = df1.loc[common_times, 'returns'].values
        returns2 = df2.loc[common_times, 'returns'].values
        
        # Calculate cross-correlation
        correlation = signal.correlate(returns1, returns2, mode='full')
        lags = signal.correlation_lags(len(returns1), len(returns2), mode='full')
        
        # Find peak correlation
        peak_idx = np.argmax(np.abs(correlation))
        peak_lag = lags[peak_idx]
        
        # Properly normalize correlation to [-1, 1] range
        n = len(returns1)
        # Shift data to align for correlation calculation
        if peak_lag > 0:  # returns1 leads returns2
            shifted_returns1 = returns1[:-peak_lag] if peak_lag < len(returns1) else returns1
            shifted_returns2 = returns2[peak_lag:] if peak_lag < len(returns2) else returns2
        elif peak_lag < 0:  # returns2 leads returns1
            shifted_returns1 = returns1[abs(peak_lag):] if abs(peak_lag) < len(returns1) else returns1
            shifted_returns2 = returns2[:peak_lag] if peak_lag < len(returns2) else returns2
        else:  # No lag
            shifted_returns1 = returns1
            shifted_returns2 = returns2
            
        # Calculate Pearson correlation for the aligned data
        if len(shifted_returns1) > 10 and len(shifted_returns2) > 10:
            peak_corr, _ = pearsonr(shifted_returns1, shifted_returns2)
            if np.isnan(peak_corr):
                peak_corr = 0.0
        else:
            peak_corr = 0.0
        
        # Debug logging for correlation
        logger.info(f"DeltaLag Debug - Raw correlation at peak: {correlation[peak_idx]:.3f}")
        logger.info(f"DeltaLag Debug - Normalized correlation: {peak_corr:.3f}")
        logger.info(f"DeltaLag Debug - Min correlation threshold: {self.min_correlation}")
        
        # Convert lag to hours (assuming hourly data)
        lag_hours = int(peak_lag)
        
        # Check if correlation meets threshold
        if abs(peak_corr) < self.min_correlation:
            logger.debug(f"Correlation too low: {peak_corr:.3f} < {self.min_correlation}")
            return None
            
        # Calculate Information Coefficient (IC)
        # IC = Correlation(Signal_t, Return_{t+21})
        # Here we use the leader's returns as signal for follower's future returns
        future_returns = np.roll(returns2, -21)[:-21]  # 21-period ahead returns
        signal_returns = returns1[:-21]  # Remove last 21 to align
        
        if len(future_returns) > 50:  # Need sufficient data for IC
            ic, ic_pvalue = pearsonr(signal_returns, future_returns)
            if np.isnan(ic):
                ic = 0.0
        else:
            ic = 0.0
            
        # Calculate Hurst exponent for regime detection
        # Use combined returns for more robust H calculation
        combined_returns = (returns1 + returns2) / 2
        hurst = self.validator.calculate_hurst_exponent(combined_returns)
        
        # Calculate activation score
        # Combines correlation, IC, and Hurst exponent
        activation_score = (
            abs(peak_corr) * 0.4 +  # 40% weight on correlation
            max(0, ic) * 0.3 +       # 30% weight on positive IC
            max(0, hurst - 0.5) * 0.6  # 30% weight on trending regime
        )
        
        # Debug logging
        logger.info(f"DeltaLag Debug - Correlation: {peak_corr:.3f}, IC: {ic:.3f}, Hurst: {hurst:.3f}")
        logger.info(f"DeltaLag Debug - Thresholds: corr>{self.min_correlation}, |IC|>{self.ic_threshold}, H>{self.hurst_threshold}")
        
        # Apply persistence filters
        if hurst < self.hurst_threshold:
            logger.debug(f"Hurst too low: {hurst:.3f} < {self.hurst_threshold}")
            return None
            
        if abs(ic) < self.ic_threshold:  # Use absolute value for IC
            logger.debug(f"IC too low: {ic:.3f} < {self.ic_threshold}")
            return None
            
        # Calculate statistical significance
        n = len(returns1)
        if n > 2:
            t_stat = peak_corr * np.sqrt((n - 2) / (1 - peak_corr**2))
            from scipy.stats import t
            p_value = 2 * (1 - t.cdf(abs(t_stat), n - 2))
        else:
            p_value = 1.0
            
        # Create result
        result = LeadLagResult(
            leader=symbol1 if lag_hours >= 0 else symbol2,
            follower=symbol2 if lag_hours >= 0 else symbol1,
            lag_hours=abs(lag_hours),
            correlation=peak_corr,
            significance=p_value,
            confidence=1 - p_value,
            timestamp=datetime.utcnow(),
            information_coefficient=ic,
            hurst_exponent=hurst,
            activation_score=activation_score
        )
        
        # Update cache
        self._cache[cache_key] = result
        
        # Update attention weights
        key = f"{result.leader}->{result.follower}"
        self.attention_weights[key] = AttentionWeights(
            leader=result.leader,
            follower=result.follower,
            attention_score=activation_score,
            time_delta=result.lag_hours,
            ic=ic,
            hurst=hurst,
            last_updated=datetime.utcnow()
        )
        
        logger.info(f"Lead-lag found: {result.leader} leads {result.follower} "
                   f"by {result.lag_hours}h (IC={ic:.3f}, H={hurst:.3f})")
        
        return result
    
    def analyze_all_pairs(self, symbols: Optional[List[str]] = None) -> List[LeadLagResult]:
        """
        Analyze all possible pairs for lead-lag relationships.
        
        Args:
            symbols: List of symbols to analyze (default: all available)
            
        Returns:
            List of significant lead-lag relationships
        """
        if symbols is None:
            symbols = list(self.price_data.keys())
        
        results = []
        
        # Analyze all unique pairs
        for i, symbol1 in enumerate(symbols):
            for symbol2 in symbols[i+1:]:
                result = self.analyze_lead_lag(symbol1, symbol2)
                if result:
                    results.append(result)
        
        # Sort by correlation strength
        results.sort(key=lambda x: abs(x.correlation), reverse=True)
        
        return results
    
    def get_leader_signals(self, 
                          leader: str, 
                          followers: List[str]) -> Dict[str, float]:
        """
        Get current signals from a leader for its followers.
        
        Args:
            leader: Leader symbol
            followers: List of follower symbols
            
        Returns:
            Dictionary of follower: signal_strength
        """
        if leader not in self.price_data:
            return {}
        
        # Get latest return for leader
        leader_data = self.price_data[leader]
        if len(leader_data) < 2:
            return {}
        
        latest_return = leader_data.iloc[-1]['returns']
        
        signals = {}
        for follower in followers:
            # Check cached lead-lag relationship
            result = self.analyze_lead_lag(leader, follower)
            if result and result.leader == leader:
                # Signal strength proportional to correlation and return magnitude
                signal_strength = result.correlation * latest_return
                signals[follower] = signal_strength
        
        return signals
    
    def select_daily_pairs(self, symbols: Optional[List[str]] = None) -> List[AttentionWeights]:
        """
        Select top leader-lagger pairs for daily trading using SugaFormer logic.
        
        Args:
            symbols: List of symbols to consider (default: all available)
            
        Returns:
            List of top pairs sorted by activation score
        """
        if symbols is None:
            symbols = list(self.price_data.keys())
            
        # Analyze all pairs
        results = self.analyze_all_pairs(symbols)
        
        # Sort by activation score
        sorted_results = sorted(results, key=lambda x: x.activation_score, reverse=True)
        
        # Extract top pairs
        top_pairs = []
        for result in sorted_results[:10]:  # Top 10 pairs
            key = f"{result.leader}->{result.follower}"
            if key in self.attention_weights:
                top_pairs.append(self.attention_weights[key])
                
        # Save updated weights
        self.save_attention_weights()
        
        logger.info(f"Selected {len(top_pairs)} daily pairs")
        return top_pairs
        
    def get_pair_recommendation(self, leader: str, follower: str) -> Optional[Dict]:
        """
        Get trading recommendation for a specific pair.
        
        Args:
            leader: Leader symbol
            follower: Follower symbol
            
        Returns:
            Trading recommendation or None
        """
        key = f"{leader}->{follower}"
        if key not in self.attention_weights:
            return None
            
        weights = self.attention_weights[key]
        
        # Check if still valid (updated within last 24 hours)
        if datetime.utcnow() - weights.last_updated > timedelta(days=1):
            return None
            
        # Generate recommendation
        if weights.attention_score > 0.7 and weights.ic > 0.05:
            direction = "BUY" if weights.ic > 0 else "SELL"
            confidence = min(weights.attention_score, 0.95)
            
            return {
                'pair': f"{leader}/{follower}",
                'direction': direction,
                'confidence': confidence,
                'lag_hours': weights.time_delta,
                'ic': weights.ic,
                'hurst': weights.hurst,
                'attention_score': weights.attention_score,
                'reasoning': f"{leader} leads {follower} by {weights.time_delta}h with IC={weights.ic:.3f}"
            }
            
        return None
    
    def get_network_graph(self) -> Dict:
        """
        Get lead-lag relationships as a network graph structure.
        
        Returns:
            Dictionary with nodes and edges for visualization
        """
        results = self.analyze_all_pairs()
        
        nodes = set()
        edges = []
        
        for result in results:
            nodes.add(result.leader)
            nodes.add(result.follower)
            
            edges.append({
                'source': result.leader,
                'target': result.follower,
                'lag_hours': result.lag_hours,
                'correlation': result.correlation,
                'confidence': result.confidence
            })
        
        return {
            'nodes': list(nodes),
            'edges': edges,
            'timestamp': datetime.utcnow().isoformat()
        }
    
    def clear_cache(self):
        """Clear correlation cache."""
        self.correlation_cache.clear()
        logger.info("Correlation cache cleared")
