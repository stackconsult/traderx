"""
Regime-Conditioned Execution Veto (Task 4.3)
Detects toxic flow regimes using VPIN and Hurst exponent to prevent capital allocation.
"""

import asyncio
import json
import logging
import time
import numpy as np
import pandas as pd
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
from collections import deque
import nolds
import structlog

logger = structlog.get_logger(__name__)


@dataclass
class VolumeBucket:
    """Volume bucket for VPIN calculation."""
    timestamp: datetime
    buy_volume: float
    sell_volume: float
    total_volume: float
    trades: int
    vwap: float
    price_range: float  # high - low


@dataclass
class RegimeState:
    """Current market regime state."""
    is_toxic: bool
    is_choppy: bool
    vpin_value: float
    hurst_exponent: float
    vpin_threshold: float
    timestamp: datetime
    freeze_until: Optional[datetime] = None


class RegimeDetector:
    """
    Detects toxic flow regimes using VPIN and Hurst exponent.
    Freezes execution when toxic conditions are detected.
    """
    
    def __init__(self, 
                 bucket_size: int = 100,  # Volume bucket size
                 vpin_window: int = 50,    # Number of buckets for VPIN
                 vpin_percentile: float = 0.95,  # Toxic threshold percentile
                 hurst_window: int = 60,   # Minutes for Hurst calculation
                 freeze_duration: int = 300):  # Freeze duration in seconds
        
        self.bucket_size = bucket_size
        self.vpin_window = vpin_window
        self.vpin_percentile = vpin_percentile
        self.hurst_window = hurst_window
        self.freeze_duration = freeze_duration
        
        # State
        self.current_bucket = None
        self.volume_buckets: deque[VolumeBucket] = deque(maxlen=vpin_window * 2)
        self.vpin_history: deque[float] = deque(maxlen=vpin_window * 10)
        self.price_history: deque[float] = deque(maxlen=hurst_window)
        self.volume_history: deque[float] = deque(maxlen=hurst_window)
        
        # Thresholds (will be updated dynamically)
        self.vpin_threshold = 0.0
        self.is_frozen = False
        self.freeze_until = None
        
        # Statistics
        self.stats = {
            "toxic_events": 0,
            "choppy_events": 0,
            "freezes_triggered": 0,
            "total_checks": 0
        }
        
        logger.info(
            "Regime detector initialized",
            bucket_size=bucket_size,
            vpin_window=vpin_window,
            vpin_percentile=vpin_percentile,
            hurst_window=hurst_window,
            freeze_duration=freeze_duration
        )
    
    async def process_tick(self, 
                          timestamp: datetime,
                          price: float,
                          volume: float,
                          side: str) -> Optional[RegimeState]:
        """
        Process a tick and update regime detection.
        
        Args:
            timestamp: Tick timestamp
            price: Tick price
            volume: Tick volume
            side: "BUY" or "SELL"
            
        Returns:
            Current regime state if regime changed, None otherwise
        """
        # Update price and volume history
        self.price_history.append(price)
        self.volume_history.append(volume)
        
        # Initialize or update current bucket
        if self.current_bucket is None:
            self.current_bucket = VolumeBucket(
                timestamp=timestamp,
                buy_volume=0.0,
                sell_volume=0.0,
                total_volume=0.0,
                trades=0,
                vwap=price,
                price_range=0.0
            )
        
        # Update current bucket
        self.current_bucket.trades += 1
        self.current_bucket.total_volume += volume
        
        if side.upper() == "BUY":
            self.current_bucket.buy_volume += volume
        else:
            self.current_bucket.sell_volume += volume
        
        # Update VWAP
        total_value = self.current_bucket.vwap * (self.current_bucket.total_volume - volume)
        self.current_bucket.vwap = (total_value + price * volume) / self.current_bucket.total_volume
        
        # Update price range
        if self.current_bucket.trades == 1:
            self.current_bucket.price_range = 0.0
        else:
            # Simple approximation - in production, track high/low properly
            self.current_bucket.price_range = abs(price - self.current_bucket.vwap) * 2
        
        # Check if bucket is complete
        if self.current_bucket.total_volume >= self.bucket_size:
            return await self._complete_bucket()
        
        return None
    
    async def _complete_bucket(self) -> Optional[RegimeState]:
        """Complete current bucket and analyze regime."""
        # Store completed bucket
        self.volume_buckets.append(self.current_bucket)
        
        # Calculate VPIN
        vpin = self._calculate_vpin()
        self.vpin_history.append(vpin)
        
        # Update threshold if we have enough history
        if len(self.vpin_history) >= self.vpin_window:
            self.vpin_threshold = np.percentile(list(self.vpin_history), self.vpin_percentile * 100)
        
        # Calculate Hurst exponent
        hurst = self._calculate_hurst_exponent()
        
        # Determine regime
        is_toxic = vpin > self.vpin_threshold
        is_choppy = hurst < 0.5
        
        # Handle toxic regime
        if is_toxic:
            self.stats["toxic_events"] += 1
            if not self.is_frozen:
                self._trigger_freeze()
                logger.warning(
                    "Toxic regime detected",
                    vpin=vpin,
                    threshold=self.vpin_threshold,
                    timestamp=self.current_bucket.timestamp
                )
        
        # Handle choppy regime
        if is_choppy:
            self.stats["choppy_events"] += 1
            logger.info(
                "Choppy regime detected",
                hurst=hurst,
                timestamp=self.current_bucket.timestamp
            )
        
        # Create regime state
        regime_state = RegimeState(
            is_toxic=is_toxic,
            is_choppy=is_choppy,
            vpin_value=vpin,
            hurst_exponent=hurst,
            vpin_threshold=self.vpin_threshold,
            timestamp=self.current_bucket.timestamp,
            freeze_until=self.freeze_until
        )
        
        # Reset current bucket
        self.current_bucket = None
        
        self.stats["total_checks"] += 1
        
        return regime_state
    
    def _calculate_vpin(self) -> float:
        """
        Calculate Volume-Synchronized Probability of Informed Trading.
        Simplified implementation using volume imbalance.
        """
        if len(self.volume_buckets) < self.vpin_window:
            return 0.0
        
        # Get recent buckets
        recent_buckets = list(self.volume_buckets)[-self.vpin_window:]
        
        # Calculate absolute volume imbalance for each bucket
        imbalances = []
        for bucket in recent_buckets:
            imbalance = abs(bucket.buy_volume - bucket.sell_volume)
            imbalances.append(imbalance)
        
        # Normalize by total volume
        total_volume = sum(b.total_volume for b in recent_buckets)
        if total_volume == 0:
            return 0.0
        
        vpin = sum(imbalances) / (total_volume * self.vpin_window)
        
        return vpin
    
    def _calculate_hurst_exponent(self) -> float:
        """
        Calculate Hurst exponent using R/S analysis.
        H < 0.5: Mean-reverting (choppy)
        H = 0.5: Random walk
        H > 0.5: Trending
        """
        if len(self.price_history) < 30:  # Minimum for Hurst calculation
            return 0.5  # Default to random walk
        
        try:
            # Use price returns
            prices = np.array(list(self.price_history))
            returns = np.diff(np.log(prices))
            
            # Calculate Hurst using R/S analysis
            hurst = nolds.hurst_rs(returns)
            
            # Clamp to reasonable range
            hurst = max(0.0, min(1.0, hurst))
            
            return hurst
            
        except Exception as e:
            logger.error(
                "Failed to calculate Hurst exponent",
                error=str(e),
                data_points=len(self.price_history)
            )
            return 0.5  # Default to random walk
    
    def _trigger_freeze(self):
        """Trigger execution freeze for toxic regime."""
        self.is_frozen = True
        self.freeze_until = datetime.utcnow() + timedelta(seconds=self.freeze_duration)
        self.stats["freezes_triggered"] += 1
        
        logger.warning(
            "Execution freeze triggered",
            freeze_duration=self.freeze_duration,
            freeze_until=self.freeze_until.isoformat()
        )
    
    def is_execution_allowed(self) -> bool:
        """
        Check if execution is currently allowed.
        
        Returns:
            True if execution is allowed, False if frozen
        """
        # Check if freeze has expired
        if self.is_frozen and self.freeze_until and datetime.utcnow() > self.freeze_until:
            self.is_frozen = False
            self.freeze_until = None
            logger.info("Execution freeze lifted")
        
        return not self.is_frozen
    
    def get_regime_status(self) -> Dict[str, any]:
        """Get current regime status and statistics."""
        return {
            "is_frozen": self.is_frozen,
            "freeze_until": self.freeze_until.isoformat() if self.freeze_until else None,
            "current_vpin": self.vpin_history[-1] if self.vpin_history else 0.0,
            "vpin_threshold": self.vpin_threshold,
            "current_hurst": self._calculate_hurst_exponent() if len(self.price_history) >= 30 else 0.5,
            "bucket_count": len(self.volume_buckets),
            "stats": self.stats.copy()
        }
    
    def save_trace(self, filepath: str):
        """Save regime detection trace for audit."""
        trace_data = {
            "timestamp": datetime.utcnow().isoformat(),
            "vpin_history": list(self.vpin_history),
            "vpin_threshold": self.vpin_threshold,
            "volume_buckets": [
                {
                    "timestamp": b.timestamp.isoformat(),
                    "buy_volume": b.buy_volume,
                    "sell_volume": b.sell_volume,
                    "total_volume": b.total_volume,
                    "trades": b.trades,
                    "vwap": b.vwap
                }
                for b in self.volume_buckets
            ],
            "statistics": self.stats,
            "freeze_events": []
        }
        
        with open(filepath, 'w') as f:
            json.dump(trace_data, f, indent=2)
        
        logger.info(
            "Regime trace saved",
            filepath=filepath,
            bucket_count=len(self.volume_buckets)
        )


# Global regime detector instance
regime_detector = RegimeDetector()


async def get_regime_detector() -> RegimeDetector:
    """Get the global regime detector instance."""
    return regime_detector
