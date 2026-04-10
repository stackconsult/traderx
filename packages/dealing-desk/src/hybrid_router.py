"""
Hybrid Risk Router (Task 4.1)
Routes orders to B-Book (internalization) or A-Book (STP) based on risk metrics.
Simulates eBPF/XDP routing logic in userspace for development.
"""

import asyncio
import json
import os
import time
from dataclasses import dataclass, asdict
from datetime import datetime
from typing import Dict, Optional, Tuple
import redis.asyncio as redis
import numpy as np
import structlog

logger = structlog.get_logger(__name__)


@dataclass
class OrderFrame:
    """Order frame for routing decision."""
    order_id: str
    tenant_id: str
    symbol: str
    side: str  # BUY/SELL
    quantity: float
    price: Optional[float]
    confidence: float  # AI confidence score
    timestamp: datetime
    metadata: Dict = None


@dataclass
class RoutingDecision:
    """Routing decision result."""
    order_id: str
    route: str  # "B_BOOK" or "A_BOOK"
    reason: str
    sharpe_ratio: float
    confidence: float
    latency_ns: int
    timestamp: datetime


class HSTRStateTensor:
    """HSTR (High-Speed Trading Risk) State Tensor stored in Redis."""
    
    def __init__(self, redis_url: str):
        self.redis_url = redis_url
        self.redis_client: Optional[redis.Redis] = None
        
    async def get_sharpe_ratio(self, symbol: str, tenant_id: str) -> float:
        """
        Get Sharpe ratio from HSTR state tensor.
        
        Returns:
            Sharpe ratio for the symbol/tenant pair
        """
        if not self.redis_client:
            self.redis_client = redis.from_url(self.redis_url)
        
        try:
            # Get state from Redis
            key = f"hstr:sharpe:{tenant_id}:{symbol}"
            value = await self.redis_client.get(key)
            
            if value:
                state = json.loads(value)
                return state.get("sharpe_ratio", 0.0)
            
            # Default Sharpe ratio if not found
            return 0.5
            
        except Exception as e:
            logger.error(
                "Failed to get Sharpe ratio from HSTR",
                symbol=symbol,
                tenant_id=tenant_id,
                error=str(e)
            )
            return 0.5
    
    async def update_sharpe_ratio(self, symbol: str, tenant_id: str, sharpe: float):
        """Update Sharpe ratio in HSTR state tensor."""
        if not self.redis_client:
            self.redis_client = redis.from_url(self.redis_url)
        
        try:
            key = f"hstr:sharpe:{tenant_id}:{symbol}"
            value = json.dumps({
                "sharpe_ratio": sharpe,
                "timestamp": datetime.utcnow().isoformat()
            })
            
            await self.redis_client.setex(key, 3600, value)  # 1 hour TTL
            
        except Exception as e:
            logger.error(
                "Failed to update Sharpe ratio in HSTR",
                symbol=symbol,
                tenant_id=tenant_id,
                sharpe=sharpe,
                error=str(e)
            )


class HybridRiskRouter:
    """
    Routes orders based on risk metrics.
    Simulates eBPF/XDP routing logic in userspace.
    """
    
    def __init__(self, 
                 redis_url: str,
                 confidence_threshold: float = 0.6,
                 sharpe_threshold: float = 1.5):
        self.redis_url = redis_url
        self.confidence_threshold = confidence_threshold
        self.sharpe_threshold = sharpe_threshold
        
        # Initialize HSTR state tensor
        self.hstr = HSTRStateTensor(redis_url)
        
        # Statistics
        self.stats = {
            "total_orders": 0,
            "b_book_routes": 0,
            "a_book_routes": 0,
            "avg_latency_ns": 0,
            "max_latency_ns": 0
        }
        
        # A-Book (STP) handler simulation
        self.a_book_handler = None
        
        # B-Book internalization handler
        self.b_book_handler = None
        
        logger.info(
            "Hybrid risk router initialized",
            confidence_threshold=confidence_threshold,
            sharpe_threshold=sharpe_threshold
        )
    
    async def route_order(self, order: OrderFrame) -> RoutingDecision:
        """
        Route order based on risk metrics.
        
        Args:
            order: Order frame to route
            
        Returns:
            Routing decision with latency measurement
        """
        start_time = time.time_ns()
        
        # Get Sharpe ratio from HSTR state tensor
        sharpe = await self.hstr.get_sharpe_ratio(order.symbol, order.tenant_id)
        
        # Make routing decision
        if order.confidence < self.confidence_threshold or sharpe > self.sharpe_threshold:
            # Route to A-Book (STP)
            route = "A_BOOK"
            reason = f"Low confidence ({order.confidence:.3f} < {self.confidence_threshold}) or high Sharpe ({sharpe:.3f} > {self.sharpe_threshold})"
            
            # Send to A-Book handler (ferrumfix)
            if self.a_book_handler:
                await self.a_book_handler(order)
                
        else:
            # Route to B-Book (internalization)
            route = "B_BOOK"
            reason = f"Adequate confidence ({order.confidence:.3f} >= {self.confidence_threshold}) and acceptable Sharpe ({sharpe:.3f} <= {self.sharpe_threshold})"
            
            # Send to B-Book handler
            if self.b_book_handler:
                await self.b_book_handler(order)
        
        # Calculate latency
        latency_ns = time.time_ns() - start_time
        
        # Update statistics
        self.stats["total_orders"] += 1
        if route == "B_BOOK":
            self.stats["b_book_routes"] += 1
        else:
            self.stats["a_book_routes"] += 1
        
        self.stats["max_latency_ns"] = max(self.stats["max_latency_ns"], latency_ns)
        self.stats["avg_latency_ns"] = (
            (self.stats["avg_latency_ns"] * (self.stats["total_orders"] - 1) + latency_ns) /
            self.stats["total_orders"]
        )
        
        # Create decision
        decision = RoutingDecision(
            order_id=order.order_id,
            route=route,
            reason=reason,
            sharpe_ratio=sharpe,
            confidence=order.confidence,
            latency_ns=latency_ns,
            timestamp=datetime.utcnow()
        )
        
        # Log routing decision
        logger.info(
            "Order routed",
            order_id=order.order_id,
            route=route,
            symbol=order.symbol,
            confidence=order.confidence,
            sharpe=sharpe,
            latency_ns=latency_ns
        )
        
        return decision
    
    def set_handlers(self, a_book_handler=None, b_book_handler=None):
        """Set handlers for A-Book and B-Book routing."""
        self.a_book_handler = a_book_handler
        self.b_book_handler = b_book_handler
    
    async def update_sharpe_ratio(self, symbol: str, tenant_id: str, sharpe: float):
        """Update Sharpe ratio in HSTR state tensor."""
        await self.hstr.update_sharpe_ratio(symbol, tenant_id, sharpe)
    
    def get_statistics(self) -> Dict:
        """Get routing statistics."""
        total = self.stats["total_orders"]
        if total == 0:
            return self.stats.copy()
        
        return {
            **self.stats,
            "b_book_percentage": (self.stats["b_book_routes"] / total) * 100,
            "a_book_percentage": (self.stats["a_book_routes"] / total) * 100
        }
    
    def save_benchmark(self, filepath: str):
        """Save routing benchmark for validation."""
        benchmark = {
            "timestamp": datetime.utcnow().isoformat(),
            "thresholds": {
                "confidence": self.confidence_threshold,
                "sharpe": self.sharpe_threshold
            },
            "statistics": self.get_statistics(),
            "target_latency_ns": 100,  # Target: <100ns per packet
            "meets_target": self.stats["avg_latency_ns"] < 100
        }
        
        with open(filepath, 'w') as f:
            json.dump(benchmark, f, indent=2)
        
        logger.info(
            "Routing benchmark saved",
            filepath=filepath,
            avg_latency_ns=self.stats["avg_latency_ns"],
            meets_target=benchmark["meets_target"]
        )


# Mock eBPF/XDP interface for development
class MockXDPProgram:
    """
    Mock XDP program that simulates kernel-level packet processing.
    In production, this would be actual eBPF code loaded via aya-rs.
    """
    
    def __init__(self, router: HybridRiskRouter):
        self.router = router
        self.packet_count = 0
        
    async def process_packet(self, packet_data: bytes) -> str:
        """
        Simulate XDP packet processing.
        
        Returns:
            XDP action: "PASS" (A-Book) or "REDIRECT" (B-Book)
        """
        self.packet_count += 1
        
        # Parse packet (simplified)
        # In real XDP, this would be raw packet parsing
        try:
            order_data = json.loads(packet_data.decode())
            order = OrderFrame(**order_data)
            
            # Route order
            decision = await self.router.route_order(order)
            
            # Return XDP action
            if decision.route == "A_BOOK":
                return "XDP_PASS"  # Route to ferrumfix A-Book
            else:
                return "XDP_REDIRECT"  # Redirect to B-Book socket
                
        except Exception as e:
            logger.error(
                "Failed to process packet",
                packet_count=self.packet_count,
                error=str(e)
            )
            return "XDP_DROP"


# Global router instance
hybrid_router = None


async def get_hybrid_router() -> HybridRiskRouter:
    """Get the global hybrid router instance."""
    global hybrid_router
    if not hybrid_router:
        redis_url = os.getenv("REDIS_URL", "redis://localhost:6379")
        hybrid_router = HybridRiskRouter(redis_url)
    return hybrid_router
