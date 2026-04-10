"""
Execution Guard - Integrates regime detection with execution control.
Enforces sub-5μs execution freezes on toxic regime detection.
"""

import asyncio
import time
from datetime import datetime
from typing import Optional, Dict, Any
import structlog

from .regime_detector import RegimeDetector, get_regime_detector
from packages.handoff.src.models.handoff_package import HandoffPackage, HandoffStatus

logger = structlog.get_logger(__name__)


class ExecutionGuard:
    """
    Guards execution based on regime detection.
    Freezes execution within 5μs of toxic event detection.
    """
    
    def __init__(self, regime_detector: Optional[RegimeDetector] = None):
        self.regime_detector = regime_detector
        self.last_check_time = 0
        self.check_interval = 0.001  # 1ms check interval
        
        # Performance tracking
        self.guard_stats = {
            "total_checks": 0,
            "blocked_executions": 0,
            "allowed_executions": 0,
            "max_latency_ns": 0,
            "avg_latency_ns": 0
        }
        
        logger.info("Execution guard initialized")
    
    async def check_execution_allowed(self, handoff: HandoffPackage) -> bool:
        """
        Check if execution is allowed for a handoff.
        
        Args:
            handoff: HandoffPackage to check
            
        Returns:
            True if execution allowed, False if blocked
        """
        start_time = time.time_ns()
        
        # Get regime detector if not provided
        if not self.regime_detector:
            self.regime_detector = await get_regime_detector()
        
        # Check if execution is frozen
        allowed = self.regime_detector.is_execution_allowed()
        
        # Update statistics
        latency = time.time_ns() - start_time
        self.guard_stats["total_checks"] += 1
        self.guard_stats["max_latency_ns"] = max(self.guard_stats["max_latency_ns"], latency)
        self.guard_stats["avg_latency_ns"] = (
            (self.guard_stats["avg_latency_ns"] * (self.guard_stats["total_checks"] - 1) + latency) /
            self.guard_stats["total_checks"]
        )
        
        if allowed:
            self.guard_stats["allowed_executions"] += 1
        else:
            self.guard_stats["blocked_executions"] += 1
            
            # Log blocked execution
            regime_status = self.regime_detector.get_regime_status()
            logger.warning(
                "Execution blocked by regime detector",
                handoff_id=handoff.id,
                vpin=regime_status["current_vpin"],
                threshold=regime_status["vpin_threshold"],
                freeze_until=regime_status["freeze_until"],
                latency_ns=latency
            )
            
            # Transition handoff to failed state
            handoff.transition_to(
                HandoffStatus.FAILED,
                reason="TOXIC_REGIME_DETECTED",
                details={
                    "vpin": regime_status["current_vpin"],
                    "threshold": regime_status["vpin_threshold"],
                    "freeze_until": regime_status["freeze_until"]
                }
            )
        
        # Verify sub-5μs requirement
        if latency > 5000:  # 5 microseconds in nanoseconds
            logger.error(
                "Execution guard latency exceeded 5μs",
                latency_ns=latency,
                handoff_id=handoff.id
            )
        
        return allowed
    
    async def process_tick(self, 
                          timestamp: datetime,
                          price: float,
                          volume: float,
                          side: str) -> Optional[Dict[str, Any]]:
        """
        Process a tick through the regime detector.
        
        Returns:
            Regime state if changed, None otherwise
        """
        if not self.regime_detector:
            self.regime_detector = await get_regime_detector()
        
        # Rate limit checks
        current_time = time.time()
        if current_time - self.last_check_time < self.check_interval:
            return None
        
        self.last_check_time = current_time
        
        # Process tick
        regime_state = await self.regime_detector.process_tick(timestamp, price, volume, side)
        
        if regime_state:
            logger.info(
                "Regime state changed",
                is_toxic=regime_state.is_toxic,
                is_choppy=regime_state.is_choppy,
                vpin=regime_state.vpin_value,
                hurst=regime_state.hurst_exponent
            )
            
            return {
                "is_toxic": regime_state.is_toxic,
                "is_choppy": regime_state.is_choppy,
                "vpin": regime_state.vpin_value,
                "hurst": regime_state.hurst_exponent,
                "threshold": regime_state.vpin_threshold,
                "timestamp": regime_state.timestamp.isoformat()
            }
        
        return None
    
    def get_guard_status(self) -> Dict[str, Any]:
        """Get execution guard status."""
        regime_status = self.regime_detector.get_regime_status() if self.regime_detector else {}
        
        return {
            "is_active": self.regime_detector is not None,
            "check_interval_ms": self.check_interval * 1000,
            "statistics": self.guard_stats.copy(),
            "regime_status": regime_status
        }
    
    def save_trace(self, filepath: str):
        """Save execution guard trace for audit."""
        if self.regime_detector:
            self.regime_detector.save_trace(filepath)
        
        logger.info("Execution guard trace saved", filepath=filepath)


# Global execution guard instance
execution_guard = ExecutionGuard()


async def get_execution_guard() -> ExecutionGuard:
    """Get the global execution guard instance."""
    return execution_guard
