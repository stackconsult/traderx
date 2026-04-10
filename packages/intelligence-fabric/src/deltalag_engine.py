"""DeltaLag Lead-Lag Detection Engine
Implements cross-correlation based lead-lag discovery for financial time series.
Based on: "DeltaLag: Learning Dynamic Lead-Lag Patterns in Financial Markets"
"""

import numpy as np
import pandas as pd
from scipy import signal
from scipy.stats import pearsonr
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
from datetime import datetime
import structlog

logger = structlog.get_logger(__name__)


@dataclass
class LeadLagRelationship:
    """Detected lead-lag relationship between two assets."""
    leader: str
    follower: str
    lag: int  # periods
    correlation: float
    confidence: float
    direction: str  # "positive" or "negative"
    timestamp: datetime


class DeltaLagEngine:
    """
    Lead-Lag Detection using cross-correlation analysis.
    
    Detects temporal relationships where one asset (leader)
    predicts movements in another (follower).
    """
    
    def __init__(self, max_lag: int = 20, min_correlation: float = 0.3):
        self.max_lag = max_lag
        self.min_correlation = min_correlation
        self.relationships: Dict[str, LeadLagRelationship] = {}
        
    def detect_lead_lag(self, 
                       leader_returns: np.ndarray,
                       follower_returns: np.ndarray,
                       leader_symbol: str,
                       follower_symbol: str) -> Optional[LeadLagRelationship]:
        """
        Detect lead-lag relationship using cross-correlation.
        
        Args:
            leader_returns: Returns series of potential leader
            follower_returns: Returns series of potential follower
            leader_symbol: Symbol name of leader
            follower_symbol: Symbol name of follower
            
        Returns:
            LeadLagRelationship if significant correlation found
        """
        # Normalize series
        leader = (leader_returns - np.mean(leader_returns)) / (np.std(leader_returns) + 1e-10)
        follower = (follower_returns - np.mean(follower_returns)) / (np.std(follower_returns) + 1e-10)
        
        # Compute cross-correlation
        correlation = signal.correlate(leader, follower, mode='full')
        lags = signal.correlation_lags(len(leader), len(follower), mode='full')
        
        # Find max correlation within lag window
        mask = (lags >= -self.max_lag) & (lags <= self.max_lag)
        valid_corr = correlation[mask]
        valid_lags = lags[mask]
        
        if len(valid_corr) == 0:
            return None
            
        max_idx = np.argmax(np.abs(valid_corr))
        max_corr = valid_corr[max_idx] / (len(leader) * np.std(leader) * np.std(follower) + 1e-10)
        best_lag = valid_lags[max_idx]
        
        # Check significance
        if abs(max_corr) < self.min_correlation:
            return None
            
        # Determine direction
        direction = "positive" if max_corr > 0 else "negative"
        
        # Calculate confidence using t-statistic
        n = len(leader)
        t_stat = max_corr * np.sqrt((n-2) / (1-max_corr**2 + 1e-10))
        confidence = min(1.0, abs(t_stat) / 2.0)  # Normalize to [0,1]
        
        relationship = LeadLagRelationship(
            leader=leader_symbol,
            follower=follower_symbol,
            lag=int(best_lag),
            correlation=float(max_corr),
            confidence=float(confidence),
            direction=direction,
            timestamp=datetime.utcnow()
        )
        
        # Store relationship
        key = f"{leader_symbol}->{follower_symbol}"
        self.relationships[key] = relationship
        
        logger.info(
            "Lead-lag detected",
            leader=leader_symbol,
            follower=follower_symbol,
            lag=best_lag,
            correlation=max_corr,
            confidence=confidence
        )
        
        return relationship
    
    def analyze_market_pairs(self, 
                            returns_df: pd.DataFrame) -> List[LeadLagRelationship]:
        """
        Analyze all pairs in returns DataFrame for lead-lag relationships.
        
        Args:
            returns_df: DataFrame with columns=symbols, rows=time periods
            
        Returns:
            List of detected relationships
        """
        relationships = []
        symbols = returns_df.columns
        
        for i, leader_sym in enumerate(symbols):
            for j, follower_sym in enumerate(symbols):
                if i == j:
                    continue
                    
                leader_returns = returns_df[leader_sym].dropna().values
                follower_returns = returns_df[follower_sym].dropna().values
                
                # Ensure equal length
                min_len = min(len(leader_returns), len(follower_returns))
                if min_len < self.max_lag * 2:
                    continue
                    
                leader_returns = leader_returns[-min_len:]
                follower_returns = follower_returns[-min_len:]
                
                rel = self.detect_lead_lag(
                    leader_returns,
                    follower_returns,
                    leader_sym,
                    follower_sym
                )
                
                if rel:
                    relationships.append(rel)
        
        # Sort by confidence
        relationships.sort(key=lambda x: x.confidence, reverse=True)
        
        return relationships
    
    def get_leading_signals(self, 
                           symbol: str, 
                           min_confidence: float = 0.5) -> List[LeadLagRelationship]:
        """Get all assets that lead the given symbol."""
        return [
            rel for rel in self.relationships.values()
            if rel.follower == symbol and rel.confidence >= min_confidence
        ]
    
    def generate_trading_signals(self,
                                symbol: str,
                                returns_df: pd.DataFrame) -> Dict:
        """Generate trading signals based on lead-lag relationships."""
        leaders = self.get_leading_signals(symbol)
        
        if not leaders:
            return {"signal": "neutral", "confidence": 0}
        
        # Aggregate signals from leaders
        weighted_signal = 0
        total_weight = 0
        
        for rel in leaders:
            leader_sym = rel.leader
            if leader_sym not in returns_df.columns:
                continue
                
            # Get recent returns of leader
            leader_returns = returns_df[leader_sym].dropna().values[-rel.lag:]
            if len(leader_returns) == 0:
                continue
                
            # Predict direction
            leader_direction = np.sign(np.mean(leader_returns))
            
            if rel.direction == "positive":
                predicted_direction = leader_direction
            else:
                predicted_direction = -leader_direction
                
            weight = rel.confidence * abs(rel.correlation)
            weighted_signal += predicted_direction * weight
            total_weight += weight
        
        if total_weight == 0:
            return {"signal": "neutral", "confidence": 0}
            
        final_signal = weighted_signal / total_weight
        confidence = min(1.0, total_weight / len(leaders))
        
        return {
            "signal": "buy" if final_signal > 0.3 else "sell" if final_signal < -0.3 else "neutral",
            "strength": abs(final_signal),
            "confidence": confidence,
            "leaders_used": len(leaders)
        }
