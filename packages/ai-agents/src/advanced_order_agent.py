"""
Advanced Order Agent (Iceberg/TWAP/VWAP)
AI-powered execution of complex order types using intelligent algorithms.
"""

import asyncio
import json
import math
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
import anthropic
import numpy as np
import structlog

logger = structlog.get_logger(__name__)


class OrderType(Enum):
    """Advanced order types."""
    ICEBERG = "iceberg"
    TWAP = "twap"
    VWAP = "vwap"
    POV = "pov"  # Percentage of Volume
    IMPLEMENTATION_SHORTFALL = "implementation_shortfall"


class OrderStatus(Enum):
    """Order execution status."""
    PENDING = "pending"
    ACTIVE = "active"
    PAUSED = "paused"
    COMPLETED = "completed"
    CANCELED = "canceled"
    PARTIAL = "partial"


@dataclass
class AdvancedOrderParams:
    """Parameters for advanced order types."""
    order_type: OrderType
    total_quantity: float
    price_limit: Optional[float]
    time_limit: Optional[datetime]
    display_quantity: Optional[float]  # For Iceberg
    participation_rate: Optional[float]  # For POV
    urgency: Optional[str]  # LOW/MEDIUM/HIGH
    max_slippage: Optional[float]


@dataclass
class ChildOrder:
    """Individual child order execution."""
    order_id: str
    parent_order_id: str
    quantity: float
    price: Optional[float]
    status: OrderStatus
    filled_quantity: float
    fill_price: Optional[float]
    created_at: datetime
    filled_at: Optional[datetime]


@dataclass
class AdvancedOrder:
    """Advanced order with AI-managed execution."""
    order_id: str
    symbol: str
    side: str  # BUY/SELL
    params: AdvancedOrderParams
    child_orders: List[ChildOrder]
    total_filled: float
    avg_fill_price: float
    status: OrderStatus
    ai_strategy: str
    execution_insights: List[str]
    created_at: datetime
    updated_at: datetime


class AdvancedOrderAgent:
    """
    AI-powered advanced order execution agent that handles
    complex order types with intelligent algorithm selection.
    """
    
    def __init__(self, anthropic_api_key: str):
        self.client = anthropic.AsyncAnthropic(api_key=anthropic_api_key)
        self.active_orders: Dict[str, AdvancedOrder] = {}
        self.execution_history: List[Dict] = []
        
        # AI prompts
        self.strategy_prompt = """
        You are an AI Execution Algorithm Specialist. Design execution strategy:
        
        Order: {order}
        Market Data: {market}
        Volume Profile: {volume_profile}
        
        Recommend:
        1. Optimal algorithm (ICEBERG/TWAP/VWAP/POV/IMPLEMENTATION_SHORTFALL)
        2. Execution parameters
        3. Timing strategy
        4. Venue selection
        5. Risk controls
        
        Consider market impact, timing risk, and liquidity.
        
        Format as JSON.
        """
        
        self.child_order_prompt = """
        Determine next child order parameters:
        
        Parent Order: {parent_order}
        Progress: {progress}
        Market State: {market}
        Recent Fills: {fills}
        
        Provide:
        1. Child order quantity
        2. Price limit (if any)
        3. Timing for next order
        4. Adjustments needed
        
        Format as JSON.
        """
    
    async def create_advanced_order(self,
                                   symbol: str,
                                   side: str,
                                   params: AdvancedOrderParams,
                                   market_context: Dict[str, Any]) -> AdvancedOrder:
        """
        Create and execute advanced order with AI strategy.
        """
        order_id = f"ADV-{int(datetime.utcnow().timestamp() * 1000)}"
        
        # Get AI strategy recommendation
        strategy = await self._get_ai_strategy(symbol, side, params, market_context)
        
        # Create advanced order
        order = AdvancedOrder(
            order_id=order_id,
            symbol=symbol,
            side=side,
            params=params,
            child_orders=[],
            total_filled=0.0,
            avg_fill_price=0.0,
            status=OrderStatus.PENDING,
            ai_strategy=strategy["strategy"],
            execution_insights=strategy["insights"],
            created_at=datetime.utcnow(),
            updated_at=datetime.utcnow()
        )
        
        self.active_orders[order_id] = order
        
        # Start execution
        await self._start_execution(order, market_context)
        
        logger.info(
            "Advanced order created",
            order_id=order_id,
            symbol=symbol,
            order_type=params.order_type.value,
            strategy=strategy["strategy"]
        )
        
        return order
    
    async def _get_ai_strategy(self,
                              symbol: str,
                              side: str,
                              params: AdvancedOrderParams,
                              market_context: Dict[str, Any]) -> Dict[str, Any]:
        """
        Get AI-recommended execution strategy.
        """
        # If order type specified, use it
        if params.order_type:
            order_type = params.order_type.value
        else:
            # Let AI decide
            prompt = self.strategy_prompt.format(
                order=asdict(params),
                market=market_context,
                volume_profile=market_context.get("volume_profile", {})
            )
            
            try:
                response = await self.client.messages.create(
                    model="claude-3-sonnet-20240229",
                    max_tokens=1000,
                    messages=[{
                        "role": "user",
                        "content": prompt
                    }]
                )
                
                strategy_data = json.loads(response.content[0].text)
                order_type = strategy_data.get("algorithm", "TWAP")
                
                return {
                    "strategy": order_type,
                    "parameters": strategy_data.get("parameters", {}),
                    "insights": strategy_data.get("insights", [])
                }
                
            except Exception as e:
                logger.error("AI strategy failed", error=str(e))
                order_type = "TWAP"
        
        return {
            "strategy": order_type,
            "parameters": {},
            "insights": [f"Using {order_type} algorithm"]
        }
    
    async def _start_execution(self, order: AdvancedOrder, market_context: Dict[str, Any]):
        """
        Start executing the advanced order.
        """
        order.status = OrderStatus.ACTIVE
        
        if order.params.order_type == OrderType.ICEBERG:
            await self._execute_iceberg(order, market_context)
        elif order.params.order_type == OrderType.TWAP:
            await self._execute_twap(order, market_context)
        elif order.params.order_type == OrderType.VWAP:
            await self._execute_vwap(order, market_context)
        elif order.params.order_type == OrderType.POV:
            await self._execute_pov(order, market_context)
        else:
            await self._execute_implementation_shortfall(order, market_context)
    
    async def _execute_iceberg(self, order: AdvancedOrder, market_context: Dict[str, Any]):
        """
        Execute Iceberg order - hidden large quantity with small visible orders.
        """
        display_qty = order.params.display_quantity or min(1000, order.params.total_quantity / 10)
        remaining = order.params.total_quantity - order.total_filled
        
        while remaining > 0 and order.status == OrderStatus.ACTIVE:
            # Create child order for display quantity
            child_qty = min(display_qty, remaining)
            
            child_order = await self._create_child_order(
                order,
                child_qty,
                order.params.price_limit,
                market_context
            )
            
            if child_order:
                order.child_orders.append(child_order)
                
                # Wait for fill or timeout
                await self._wait_for_fill(child_order, timeout=30)
                
                # Update remaining
                remaining = order.params.total_quantity - order.total_filled
                
                # Check if time limit reached
                if order.params.time_limit and datetime.utcnow() > order.params.time_limit:
                    break
                
                # Small delay between orders
                await asyncio.sleep(1)
        
        # Check completion
        if order.total_filled >= order.params.total_quantity:
            order.status = OrderStatus.COMPLETED
        elif remaining > 0:
            order.status = OrderStatus.PARTIAL
    
    async def _execute_twap(self, order: AdvancedOrder, market_context: Dict[str, Any]):
        """
        Execute TWAP (Time-Weighted Average Price) order.
        """
        # Calculate time slices
        time_limit = order.params.time_limit or datetime.utcnow() + timedelta(hours=1)
        time_remaining = (time_limit - datetime.utcnow()).total_seconds()
        
        if time_remaining <= 0:
            return
        
        num_slices = min(20, max(5, int(time_remaining / 60)))  # 5-20 slices
        slice_interval = time_remaining / num_slices
        slice_quantity = (order.params.total_quantity - order.total_filled) / num_slices
        
        slice_count = 0
        
        while slice_count < num_slices and order.status == OrderStatus.ACTIVE:
            # Create slice order
            child_order = await self._create_child_order(
                order,
                slice_quantity,
                None,  # Market order
                market_context
            )
            
            if child_order:
                order.child_orders.append(child_order)
                await self._wait_for_fill(child_order, timeout=slice_interval * 0.8)
            
            slice_count += 1
            
            # Wait for next slice
            if slice_count < num_slices:
                await asyncio.sleep(slice_interval)
        
        # Update status
        if order.total_filled >= order.params.total_quantity:
            order.status = OrderStatus.COMPLETED
        else:
            order.status = OrderStatus.PARTIAL
    
    async def _execute_vwap(self, order: AdvancedOrder, market_context: Dict[str, Any]):
        """
        Execute VWAP (Volume-Weighted Average Price) order.
        """
        # Get volume profile
        volume_profile = market_context.get("volume_profile", {})
        total_volume = sum(volume_profile.values())
        
        if not volume_profile or total_volume == 0:
            # Fallback to TWAP
            await self._execute_twap(order, market_context)
            return
        
        # Calculate participation rate
        participation = order.params.participation_rate or 0.1  # 10% default
        
        # Execute based on volume profile
        for time_bucket, bucket_volume in volume_profile.items():
            if order.status != OrderStatus.ACTIVE:
                break
            
            # Calculate order size for this bucket
            target_volume = bucket_volume * participation
            remaining = order.params.total_quantity - order.total_filled
            
            if remaining <= 0:
                break
            
            order_size = min(target_volume, remaining)
            
            # Create child order
            child_order = await self._create_child_order(
                order,
                order_size,
                None,
                market_context
            )
            
            if child_order:
                order.child_orders.append(child_order)
                await self._wait_for_fill(child_order, timeout=60)
            
            # Wait for next time bucket
            await asyncio.sleep(60)  # 1 minute buckets
    
    async def _execute_pov(self, order: AdvancedOrder, market_context: Dict[str, Any]):
        """
        Execute POV (Percentage of Volume) order.
        """
        participation_rate = order.params.participation_rate or 0.05  # 5% default
        time_limit = order.params.time_limit or datetime.utcnow() + timedelta(hours=1)
        
        while datetime.utcnow() < time_limit and order.status == OrderStatus.ACTIVE:
            # Get current market volume
            current_volume = market_context.get("current_volume", 0)
            
            if current_volume > 0:
                # Calculate allowed order size
                max_size = current_volume * participation_rate
                remaining = order.params.total_quantity - order.total_filled
                
                if remaining > 0:
                    order_size = min(max_size, remaining)
                    
                    child_order = await self._create_child_order(
                        order,
                        order_size,
                        None,
                        market_context
                    )
                    
                    if child_order:
                        order.child_orders.append(child_order)
                        await self._wait_for_fill(child_order, timeout=30)
            
            # Check completion
            if order.total_filled >= order.params.total_quantity:
                break
            
            # Wait before next check
            await asyncio.sleep(10)
    
    async def _execute_implementation_shortfall(self, order: AdvancedOrder, market_context: Dict[str, Any]):
        """
        Execute Implementation Shortfall algorithm.
        """
        # Calculate optimal execution path minimizing market impact
        urgency = order.params.urgency or "MEDIUM"
        
        # Adjust execution speed based on urgency
        if urgency == "HIGH":
            # Execute faster, accept more impact
            participation = 0.2
            slice_time = 30
        elif urgency == "LOW":
            # Execute slower, minimize impact
            participation = 0.05
            slice_time = 300
        else:
            # Balanced
            participation = 0.1
            slice_time = 120
        
        remaining = order.params.total_quantity - order.total_filled
        
        while remaining > 0 and order.status == OrderStatus.ACTIVE:
            # Get AI recommendation for next slice
            prompt = self.child_order_prompt.format(
                parent_order=asdict(order),
                progress={
                    "filled": order.total_filled,
                    "remaining": remaining,
                    "progress_pct": (order.total_filled / order.params.total_quantity) * 100
                },
                market=market_context,
                fills=[asdict(f) for f in order.child_orders[-5:] if f.status == OrderStatus.COMPLETED]
            )
            
            try:
                response = await self.client.messages.create(
                    model="claude-3-sonnet-20240229",
                    max_tokens=500,
                    messages=[{
                        "role": "user",
                        "content": prompt
                    }]
                )
                
                child_params = json.loads(response.content[0].text)
                slice_quantity = child_params.get("quantity", remaining * participation)
                slice_price = child_params.get("price_limit", order.params.price_limit)
                
            except Exception as e:
                logger.error("Child order AI failed", error=str(e))
                slice_quantity = remaining * participation
                slice_price = order.params.price_limit
            
            # Execute slice
            child_order = await self._create_child_order(
                order,
                slice_quantity,
                slice_price,
                market_context
            )
            
            if child_order:
                order.child_orders.append(child_order)
                await self._wait_for_fill(child_order, timeout=slice_time)
            
            remaining = order.params.total_quantity - order.total_filled
            
            # Wait for next slice
            if remaining > 0:
                await asyncio.sleep(slice_time)
    
    async def _create_child_order(self,
                                 parent_order: AdvancedOrder,
                                 quantity: float,
                                 price: Optional[float],
                                 market_context: Dict[str, Any]) -> Optional[ChildOrder]:
        """
        Create a child order.
        """
        child_id = f"CHILD-{int(datetime.utcnow().timestamp() * 1000)}"
        
        child_order = ChildOrder(
            order_id=child_id,
            parent_order_id=parent_order.order_id,
            quantity=quantity,
            price=price,
            status=OrderStatus.PENDING,
            filled_quantity=0.0,
            fill_price=None,
            created_at=datetime.utcnow(),
            filled_at=None
        )
        
        # Submit to exchange (simulated)
        # In production, this would call the exchange API
        logger.info(
            "Child order created",
            child_id=child_id,
            parent_id=parent_order.order_id,
            quantity=quantity,
            price=price
        )
        
        # Simulate fill
        await asyncio.sleep(0.1)
        fill_price = price or market_context.get("current_price", 50000)
        
        child_order.status = OrderStatus.COMPLETED
        child_order.filled_quantity = quantity
        child_order.fill_price = fill_price
        child_order.filled_at = datetime.utcnow()
        
        # Update parent order
        await self._update_parent_order(parent_order, child_order)
        
        return child_order
    
    async def _update_parent_order(self, parent_order: AdvancedOrder, child_order: ChildOrder):
        """
        Update parent order with child order fill.
        """
        parent_order.total_filled += child_order.filled_quantity
        
        # Calculate average fill price
        total_value = 0
        total_qty = 0
        
        for child in parent_order.child_orders:
            if child.fill_price:
                total_value += child.fill_price * child.filled_quantity
                total_qty += child.filled_quantity
        
        if total_qty > 0:
            parent_order.avg_fill_price = total_value / total_qty
        
        parent_order.updated_at = datetime.utcnow()
        
        # Check completion
        if parent_order.total_filled >= parent_order.params.total_quantity:
            parent_order.status = OrderStatus.COMPLETED
            logger.info(
                "Advanced order completed",
                order_id=parent_order.order_id,
                total_filled=parent_order.total_filled,
                avg_price=parent_order.avg_fill_price
            )
    
    async def _wait_for_fill(self, child_order: ChildOrder, timeout: float):
        """
        Wait for child order to fill.
        """
        # In production, this would monitor the order status
        # For now, child orders fill immediately in _create_child_order
        pass
    
    async def cancel_advanced_order(self, order_id: str, reason: str = "") -> bool:
        """
        Cancel an advanced order.
        """
        if order_id not in self.active_orders:
            return False
        
        order = self.active_orders[order_id]
        
        # Cancel all pending child orders
        for child in order.child_orders:
            if child.status in [OrderStatus.PENDING, OrderStatus.ACTIVE]:
                child.status = OrderStatus.CANCELED
        
        order.status = OrderStatus.CANCELED
        order.updated_at = datetime.utcnow()
        
        logger.info(
            "Advanced order canceled",
            order_id=order_id,
            reason=reason,
            filled_pct=(order.total_filled / order.params.total_quantity) * 100
        )
        
        return True
    
    def get_order(self, order_id: str) -> Optional[AdvancedOrder]:
        """Get advanced order by ID."""
        return self.active_orders.get(order_id)
    
    def get_active_orders(self) -> List[AdvancedOrder]:
        """Get all active advanced orders."""
        return [o for o in self.active_orders.values() if o.status == OrderStatus.ACTIVE]
