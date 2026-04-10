"""
Adapter Manager - Orchestrates multi-venue liquidity execution.
Implements smart order routing and concurrent execution.
"""

import asyncio
import logging
from typing import List, Dict, Any, Optional, Tuple
from datetime import datetime
import time
from dataclasses import dataclass

from .ports.liquidity_port import LiquidityPort
from .domain.models import (
    UnifiedOrder, ExecutionResult, LiquidityRequest, 
    LiquidityResponse, VenueInfo, OrderStatus
)

logger = logging.getLogger(__name__)


@dataclass
class VenuePerformance:
    """Venue performance metrics."""
    venue_name: str
    latency_ms: float
    success_rate: float
    last_updated: datetime
    error_count: int = 0
    total_requests: int = 0


class AdapterManager:
    """
    Manages multiple liquidity adapters and orchestrates execution.
    Implements smart order routing and performance monitoring.
    """
    
    def __init__(self):
        self.adapters: Dict[str, LiquidityPort] = {}
        self.venue_performance: Dict[str, VenuePerformance] = {}
        self._performance_lock = asyncio.Lock()
        self._execution_stats = {
            "total_orders": 0,
            "successful_orders": 0,
            "failed_orders": 0,
            "average_latency_ms": 0.0
        }
        
    async def register_adapter(self, adapter: LiquidityPort) -> bool:
        """
        Register a liquidity adapter.
        
        Args:
            adapter: Liquidity adapter to register
            
        Returns:
            True if registration successful
        """
        try:
            # Connect adapter
            if not await adapter.connect():
                logger.error(f"Failed to connect adapter for {adapter.venue_name}")
                return False
                
            # Register adapter
            self.adapters[adapter.venue_name] = adapter
            
            # Initialize performance tracking
            venue_info = await adapter.get_venue_info()
            self.venue_performance[adapter.venue_name] = VenuePerformance(
                venue_name=adapter.venue_name,
                latency_ms=0.0,
                success_rate=1.0,
                last_updated=datetime.utcnow()
            )
            
            logger.info(f"Registered adapter for {adapter.venue_name}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to register adapter: {e}")
            return False
            
    async def unregister_adapter(self, venue_name: str) -> None:
        """Unregister and disconnect an adapter."""
        if venue_name in self.adapters:
            await self.adapters[venue_name].disconnect()
            del self.adapters[venue_name]
            del self.venue_performance[venue_name]
            logger.info(f"Unregistered adapter for {venue_name}")
            
    async def execute_order(self, request: LiquidityRequest) -> LiquidityResponse:
        """
        Execute order across multiple venues.
        
        Args:
            request: Liquidity execution request
            
        Returns:
            Execution response
        """
        start_time = time.time()
        request_id = str(int(start_time * 1000000))
        
        try:
            # Select venues for execution
            selected_venues = await self._select_venues(request)
            
            if not selected_venues:
                return LiquidityResponse(
                    request_id=request_id,
                    status="FAILED",
                    error_message="No suitable venues available"
                )
                
            # Execute order based on routing strategy
            if request.routing_strategy == "BEST_PRICE":
                response = await self._execute_best_price(request, selected_venues)
            elif request.routing_strategy == "FASTEST":
                response = await self._execute_fastest(request, selected_venues)
            elif request.routing_strategy == "SPLIT":
                response = await self._execute_split(request, selected_venues)
            else:
                response = await self._execute_best_price(request, selected_venues)
                
            # Update performance metrics
            await self._update_performance_metrics(selected_venues, response)
            
            # Calculate overall latency
            response.latency_ms = (time.time() - start_time) * 1000
            response.request_id = request_id
            
            # Update global stats
            self._execution_stats["total_orders"] += 1
            if response.status == "SUCCESS":
                self._execution_stats["successful_orders"] += 1
            else:
                self._execution_stats["failed_orders"] += 1
                
            return response
            
        except Exception as e:
            logger.error(f"Order execution failed: {e}")
            return LiquidityResponse(
                request_id=request_id,
                status="FAILED",
                error_message=str(e),
                latency_ms=(time.time() - start_time) * 1000
            )
            
    async def _select_venues(self, request: LiquidityRequest) -> List[str]:
        """Select venues for order execution."""
        if request.venues:
            # Use specified venues
            return [v for v in request.venues if v in self.adapters]
            
        # Select based on performance and capabilities
        suitable_venues = []
        
        for venue_name, adapter in self.adapters.items():
            # Check if venue supports the symbol
            venue_info = await adapter.get_venue_info()
            if venue_info.supported_symbols and request.order.symbol not in venue_info.supported_symbols:
                continue
                
            # Check venue performance
            perf = self.venue_performance.get(venue_name)
            if perf and perf.success_rate < 0.8:  # Skip venues with low success rate
                continue
                
            suitable_venues.append(venue_name)
            
        # Sort by latency (fastest first)
        suitable_venues.sort(key=lambda v: self.venue_performance[v].latency_ms)
        
        return suitable_venues[:5]  # Limit to top 5 venues
        
    async def _execute_best_price(self, request: LiquidityRequest, venues: List[str]) -> LiquidityResponse:
        """Execute on venue with best price."""
        executions = []
        
        # Get quotes from all venues
        quotes = []
        for venue_name in venues:
            adapter = self.adapters[venue_name]
            try:
                # Get market data for price
                async for tick in adapter.get_market_data([request.order.symbol]):
                    if tick.bid and tick.ask:
                        quotes.append((venue_name, tick.bid, tick.ask))
                        break
            except Exception as e:
                logger.warning(f"Failed to get quote from {venue_name}: {e}")
                
        if not quotes:
            return LiquidityResponse(
                request_id="",
                status="FAILED",
                error_message="No quotes available"
            )
            
        # Select best venue
        if request.order.side.value == "BUY":
            best_venue = min(quotes, key=lambda x: x[2])  # Lowest ask
        else:
            best_venue = max(quotes, key=lambda x: x[1])  # Highest bid
            
        # Execute on best venue
        adapter = self.adapters[best_venue[0]]
        result = await adapter.submit_order(request.order)
        
        if result.status == OrderStatus.FILLED:
            return LiquidityResponse(
                request_id="",
                status="SUCCESS",
                executions=[result],
                total_filled=result.filled_quantity,
                average_price=result.average_price,
                total_fees=sum(result.fees.values())
            )
        else:
            return LiquidityResponse(
                request_id="",
                status="FAILED",
                executions=[result],
                error_message=result.error_message
            )
            
    async def _execute_fastest(self, request: LiquidityRequest, venues: List[str]) -> LiquidityResponse:
        """Execute on fastest venue."""
        # Use first venue (sorted by latency)
        if not venues:
            return LiquidityResponse(
                request_id="",
                status="FAILED",
                error_message="No venues available"
            )
            
        adapter = self.adapters[venues[0]]
        result = await adapter.submit_order(request.order)
        
        if result.status == OrderStatus.FILLED:
            return LiquidityResponse(
                request_id="",
                status="SUCCESS",
                executions=[result],
                total_filled=result.filled_quantity,
                average_price=result.average_price,
                total_fees=sum(result.fees.values())
            )
        else:
            return LiquidityResponse(
                request_id="",
                status="FAILED",
                executions=[result],
                error_message=result.error_message
            )
            
    async def _execute_split(self, request: LiquidityRequest, venues: List[str]) -> LiquidityResponse:
        """Split order across multiple venues."""
        if len(venues) < 2:
            return await self._execute_best_price(request, venues)
            
        # Split quantity equally
        quantity_per_venue = request.order.quantity / len(venues)
        executions = []
        total_filled = 0.0
        total_fees = 0.0
        prices = []
        
        # Submit orders to all venues
        tasks = []
        for venue_name in venues:
            adapter = self.adapters[venue_name]
            split_order = UnifiedOrder(
                tenant_id=request.order.tenant_id,
                symbol=request.order.symbol,
                side=request.order.side,
                quantity=quantity_per_venue,
                order_type=request.order.order_type,
                price=request.order.price,
                time_in_force=request.order.time_in_force
            )
            tasks.append(adapter.submit_order(split_order))
            
        # Wait for all executions
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        for result in results:
            if isinstance(result, Exception):
                logger.error(f"Split order failed: {result}")
                continue
                
            executions.append(result)
            
            if result.status == OrderStatus.FILLED:
                total_filled += result.filled_quantity
                total_fees += sum(result.fees.values())
                if result.average_price:
                    prices.append(result.average_price)
                    
        if total_filled > 0:
            return LiquidityResponse(
                request_id="",
                status="SUCCESS" if total_filled == request.order.quantity else "PARTIAL",
                executions=executions,
                total_filled=total_filled,
                average_price=sum(prices) / len(prices) if prices else None,
                total_fees=total_fees
            )
        else:
            return LiquidityResponse(
                request_id="",
                status="FAILED",
                executions=executions,
                error_message="All split orders failed"
            )
            
    async def _update_performance_metrics(self, venues: List[str], response: LiquidityResponse):
        """Update performance metrics for venues."""
        async with self._performance_lock:
            for venue_name in venues:
                perf = self.venue_performance.get(venue_name)
                if not perf:
                    continue
                    
                perf.total_requests += 1
                
                # Check if venue had successful execution
                venue_success = any(
                    exec.execution_venue == venue_name and 
                    exec.status == OrderStatus.FILLED 
                    for exec in response.executions
                )
                
                if venue_success:
                    perf.success_rate = (perf.success_rate * (perf.total_requests - 1) + 1.0) / perf.total_requests
                else:
                    perf.success_rate = (perf.success_rate * (perf.total_requests - 1)) / perf.total_requests
                    perf.error_count += 1
                    
                perf.last_updated = datetime.utcnow()
                
    async def get_venue_status(self) -> Dict[str, Dict[str, Any]]:
        """Get status of all registered venues."""
        status = {}
        
        for venue_name, adapter in self.adapters.items():
            perf = self.venue_performance.get(venue_name)
            health = await adapter.health_check()
            
            status[venue_name] = {
                "connected": adapter.is_connected,
                "latency_ms": perf.latency_ms if perf else None,
                "success_rate": perf.success_rate if perf else None,
                "error_count": perf.error_count if perf else 0,
                "health": health
            }
            
        return status
        
    async def run_performance_test(self, num_orders: int = 100, concurrent_venues: int = 5) -> Dict[str, float]:
        """
        Run performance test for concurrent order submission.
        
        Args:
            num_orders: Number of orders to submit
            concurrent_venues: Number of venues to use
            
        Returns:
            Performance metrics
        """
        logger.info(f"Starting performance test: {num_orders} orders across {concurrent_venues} venues")
        
        # Create test orders
        test_orders = []
        for i in range(num_orders):
            order = UnifiedOrder(
                tenant_id="test-tenant",
                symbol="BTCUSDT",
                side="BUY" if i % 2 == 0 else "SELL",
                quantity=0.01,
                order_type="MARKET"
            )
            
            request = LiquidityRequest(
                order=order,
                routing_strategy="FASTEST"
            )
            test_orders.append(request)
            
        # Execute orders concurrently
        start_time = time.time()
        tasks = [self.execute_order(req) for req in test_orders]
        results = await asyncio.gather(*tasks, return_exceptions=True)
        end_time = time.time()
        
        # Calculate metrics
        total_time = end_time - start_time
        successful_orders = sum(
            1 for r in results 
            if not isinstance(r, Exception) and r.status in ["SUCCESS", "PARTIAL"]
        )
        
        avg_latency = sum(
            r.latency_ms for r in results 
            if not isinstance(r, Exception)
        ) / len(results)
        
        throughput = num_orders / total_time
        
        metrics = {
            "total_orders": num_orders,
            "successful_orders": successful_orders,
            "total_time_seconds": total_time,
            "average_latency_ms": avg_latency,
            "orders_per_second": throughput
        }
        
        logger.info(f"Performance test completed: {metrics}")
        return metrics
        
    async def shutdown(self):
        """Shutdown all adapters."""
        for adapter in self.adapters.values():
            await adapter.disconnect()
        self.adapters.clear()
        self.venue_performance.clear()
        logger.info("Adapter manager shutdown complete")
