"""
Smart Order Routing Agent
AI-powered intelligent order routing across multiple venues for optimal execution.
"""

import asyncio
import json
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
import anthropic
import numpy as np
import structlog

logger = structlog.get_logger(__name__)


class VenueType(Enum):
    """Types of trading venues."""
    EXCHANGE = "exchange"
    ECN = "ecn"
    DARK_POOL = "dark_pool"
    INTERNALIZER = "internalizer"
    RETAIL_MARKET_MAKER = "retail_mm"


class RoutingStrategy(Enum):
    """Routing strategies."""
    BEST_PRICE = "best_price"
    LOWEST_COST = "lowest_cost"
    FASTEST_EXECUTION = "fastest_execution"
    LIQUIDITY_SEEKING = "liquidity_seeking"
    IMPACT_MINIMIZATION = "impact_minimization"


@dataclass
class Venue:
    """Trading venue with AI-assessed characteristics."""
    venue_id: str
    name: str
    venue_type: VenueType
    fees: Dict[str, float]  # maker, taker, per_share
    latency_ms: float
    liquidity_score: float  # 0-1, AI assessed
    reliability_score: float  # 0-1, AI assessed
    supported_symbols: List[str]
    min_order_size: float
    max_order_size: float
    last_updated: datetime


@dataclass
class RoutingDecision:
    """AI routing decision for an order."""
    order_id: str
    venue_id: str
    venue_name: str
    strategy: RoutingStrategy
    confidence: float
    expected_fill_time_ms: float
    total_cost_estimate: float
    price_improvement: float
    reasoning: str
    alternative_venues: List[Dict[str, Any]]
    timestamp: datetime


@dataclass
class OrderRouteRequest:
    """Request for order routing."""
    order_id: str
    symbol: str
    side: str  # BUY/SELL
    quantity: float
    order_type: str  # MARKET/LIMIT
    price_limit: Optional[float]
    urgency: str  # LOW/MEDIUM/HIGH
    max_venues: Optional[int]
    excluded_venues: List[str]
    preferences: Dict[str, Any]


class SmartOrderRoutingAgent:
    """
    AI-powered smart order routing agent that intelligently routes orders
    to optimal venues based on multiple factors.
    """
    
    def __init__(self, anthropic_api_key: str):
        self.client = anthropic.AsyncAnthropic(api_key=anthropic_api_key)
        self.venues: Dict[str, Venue] = {}
        self.routing_history: List[RoutingDecision] = []
        self.performance_metrics: Dict[str, Dict] = {}
        
        # AI prompts
        self.venue_assessment_prompt = """
        You are an AI Venue Analyst. Assess this trading venue:
        
        Venue Data: {venue_data}
        Recent Performance: {performance}
        Market Conditions: {market}
        
        Provide:
        1. Liquidity score (0-1)
        2. Reliability score (0-1)
        3. Cost efficiency assessment
        4. Execution quality rating
        5. Recommended order types
        6. Risk factors
        
        Consider fees, latency, fill rates, and market impact.
        
        Format as JSON.
        """
        
        self.routing_decision_prompt = """
        Make routing decision for this order:
        
        Order: {order}
        Available Venues: {venues}
        Market Data: {market}
        Historical Performance: {performance}
        
        Determine:
        1. Optimal venue (venue_id)
        2. Routing strategy
        3. Confidence (0-1)
        4. Expected fill time (ms)
        5. Total cost estimate
        6. Price improvement potential
        7. Reasoning
        8. Top 3 alternative venues with scores
        
        Consider:
        - Best execution obligation
        - Cost vs speed tradeoffs
        - Liquidity depth
        - Venue reliability
        
        Format as JSON.
        """
    
    async def register_venue(self, venue_data: Dict[str, Any]) -> Venue:
        """
        Register a new trading venue with AI assessment.
        """
        venue_id = venue_data["venue_id"]
        
        # Get AI assessment
        assessment = await self._assess_venue(venue_data)
        
        venue = Venue(
            venue_id=venue_id,
            name=venue_data["name"],
            venue_type=VenueType(venue_data["type"]),
            fees=venue_data.get("fees", {}),
            latency_ms=venue_data.get("latency_ms", 0),
            liquidity_score=assessment.get("liquidity_score", 0.5),
            reliability_score=assessment.get("reliability_score", 0.5),
            supported_symbols=venue_data.get("supported_symbols", []),
            min_order_size=venue_data.get("min_order_size", 0.01),
            max_order_size=venue_data.get("max_order_size", float('inf')),
            last_updated=datetime.utcnow()
        )
        
        self.venues[venue_id] = venue
        
        logger.info(
            "Venue registered",
            venue_id=venue_id,
            name=venue.name,
            liquidity_score=venue.liquidity_score,
            reliability_score=venue.reliability_score
        )
        
        return venue
    
    async def _assess_venue(self, venue_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Use AI to assess venue quality.
        """
        prompt = self.venue_assessment_prompt.format(
            venue_data=venue_data,
            performance=self.performance_metrics.get(venue_data["venue_id"], {}),
            market={}  # Would include current market data
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
            
            return json.loads(response.content[0].text)
            
        except Exception as e:
            logger.error(
                "Venue assessment failed",
                venue_id=venue_data["venue_id"],
                error=str(e)
            )
            
            # Default assessment
            return {
                "liquidity_score": 0.5,
                "reliability_score": 0.5,
                "cost_efficiency": "MEDIUM",
                "execution_quality": "AVERAGE"
            }
    
    async def route_order(self, request: OrderRouteRequest, market_data: Dict[str, Any]) -> RoutingDecision:
        """
        Route order to optimal venue using AI decision-making.
        """
        logger.info(
            "Routing order",
            order_id=request.order_id,
            symbol=request.symbol,
            quantity=request.quantity,
            side=request.side
        )
        
        # Filter eligible venues
        eligible_venues = await self._filter_venues(request)
        
        if not eligible_venues:
            # No eligible venues
            raise ValueError(f"No eligible venues for order {request.order_id}")
        
        # Get AI routing decision
        decision = await self._make_routing_decision(request, eligible_venues, market_data)
        
        # Store decision
        self.routing_history.append(decision)
        
        # Update venue performance (would be done after execution)
        self.performance_metrics[decision.venue_id] = {
            "last_used": decision.timestamp,
            "orders_routed": self.performance_metrics.get(decision.venue_id, {}).get("orders_routed", 0) + 1
        }
        
        logger.info(
            "Order routed",
            order_id=request.order_id,
            venue=decision.venue_name,
            strategy=decision.strategy.value,
            confidence=decision.confidence
        )
        
        return decision
    
    async def _filter_venues(self, request: OrderRouteRequest) -> List[Venue]:
        """
        Filter venues based on order requirements.
        """
        eligible = []
        
        for venue in self.venues.values():
            # Check symbol support
            if request.symbol not in venue.supported_symbols:
                continue
            
            # Check excluded venues
            if venue.venue_id in request.excluded_venues:
                continue
            
            # Check order size limits
            if request.quantity < venue.min_order_size or request.quantity > venue.max_order_size:
                continue
            
            # Check reliability threshold
            if venue.reliability_score < 0.3:  # Minimum reliability
                continue
            
            eligible.append(venue)
        
        # Sort by liquidity score (primary) and reliability (secondary)
        eligible.sort(key=lambda v: (v.liquidity_score, v.reliability_score), reverse=True)
        
        # Apply max venues limit
        if request.max_venues:
            eligible = eligible[:request.max_venues]
        
        return eligible
    
    async def _make_routing_decision(self,
                                    request: OrderRouteRequest,
                                    venues: List[Venue],
                                    market_data: Dict[str, Any]) -> RoutingDecision:
        """
        Use AI to make optimal routing decision.
        """
        # Prepare venue data for AI
        venues_data = []
        for venue in venues:
            venues_data.append({
                "venue_id": venue.venue_id,
                "name": venue.name,
                "type": venue.venue_type.value,
                "fees": venue.fees,
                "latency_ms": venue.latency_ms,
                "liquidity_score": venue.liquidity_score,
                "reliability_score": venue.reliability_score,
                "performance": self.performance_metrics.get(venue.venue_id, {})
            })
        
        prompt = self.routing_decision_prompt.format(
            order=asdict(request),
            venues=venues_data,
            market=market_data,
            performance=self.performance_metrics
        )
        
        try:
            response = await self.client.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=1500,
                messages=[{
                    "role": "user",
                    "content": prompt
                }]
            )
            
            decision_data = json.loads(response.content[0].text)
            
            # Find selected venue
            selected_venue = next(
                (v for v in venues if v.venue_id == decision_data["optimal_venue"]),
                venues[0]  # Fallback to first venue
            )
            
            # Create routing decision
            decision = RoutingDecision(
                order_id=request.order_id,
                venue_id=selected_venue.venue_id,
                venue_name=selected_venue.name,
                strategy=RoutingStrategy(decision_data.get("strategy", "BEST_PRICE")),
                confidence=float(decision_data.get("confidence", 0.5)),
                expected_fill_time_ms=float(decision_data.get("expected_fill_time", 100)),
                total_cost_estimate=float(decision_data.get("total_cost", 0)),
                price_improvement=float(decision_data.get("price_improvement", 0)),
                reasoning=decision_data.get("reasoning", ""),
                alternative_venues=decision_data.get("alternative_venues", []),
                timestamp=datetime.utcnow()
            )
            
            return decision
            
        except Exception as e:
            logger.error(
                "AI routing decision failed",
                order_id=request.order_id,
                error=str(e)
            )
            
            # Fallback to best venue by liquidity
            best_venue = venues[0]
            
            return RoutingDecision(
                order_id=request.order_id,
                venue_id=best_venue.venue_id,
                venue_name=best_venue.name,
                strategy=RoutingStrategy.BEST_PRICE,
                confidence=0.3,
                expected_fill_time_ms=best_venue.latency_ms,
                total_cost_estimate=0,
                price_improvement=0,
                reasoning="AI decision failed, using fallback",
                alternative_venues=[],
                timestamp=datetime.utcnow()
            )
    
    async def multi_venue_routing(self,
                                 request: OrderRouteRequest,
                                 market_data: Dict[str, Any],
                                 split_ratio: Dict[str, float]) -> List[RoutingDecision]:
        """
        Route order across multiple venues with specified split ratios.
        """
        decisions = []
        remaining_quantity = request.quantity
        
        for venue_id, ratio in split_ratio.items():
            if remaining_quantity <= 0:
                break
            
            if venue_id not in self.venues:
                continue
            
            venue_quantity = min(request.quantity * ratio, remaining_quantity)
            
            # Create sub-request
            sub_request = OrderRouteRequest(
                order_id=f"{request.order_id}_{venue_id}",
                symbol=request.symbol,
                side=request.side,
                quantity=venue_quantity,
                order_type=request.order_type,
                price_limit=request.price_limit,
                urgency=request.urgency,
                max_venues=1,
                excluded_venues=[],
                preferences=request.preferences
            )
            
            # Route sub-order
            try:
                decision = await self.route_order(sub_request, market_data)
                decisions.append(decision)
                remaining_quantity -= venue_quantity
            except Exception as e:
                logger.error(
                    "Multi-venue routing failed",
                    venue_id=venue_id,
                    error=str(e)
                )
        
        return decisions
    
    async def update_venue_performance(self,
                                      venue_id: str,
                                      execution_data: Dict[str, Any]):
        """
        Update venue performance metrics for learning.
        """
        if venue_id not in self.performance_metrics:
            self.performance_metrics[venue_id] = {}
        
        metrics = self.performance_metrics[venue_id]
        
        # Update metrics
        metrics["avg_fill_time_ms"] = self._update_average(
            metrics.get("avg_fill_time_ms", 0),
            execution_data.get("fill_time_ms", 0),
            metrics.get("executions", 0)
        )
        
        metrics["fill_rate"] = self._update_average(
            metrics.get("fill_rate", 1.0),
            1.0 if execution_data.get("filled", False) else 0.0,
            metrics.get("executions", 0)
        )
        
        metrics["price_improvement"] = self._update_average(
            metrics.get("price_improvement", 0),
            execution_data.get("price_improvement", 0),
            metrics.get("executions", 0)
        )
        
        metrics["executions"] = metrics.get("executions", 0) + 1
        metrics["last_updated"] = datetime.utcnow()
        
        # Reassess venue if significant performance change
        if metrics["executions"] % 100 == 0:
            venue = self.venues.get(venue_id)
            if venue:
                await self._reassess_venue(venue)
    
    async def _reassess_venue(self, venue: Venue):
        """
        Reassess venue based on updated performance.
        """
        venue_data = {
            "venue_id": venue.venue_id,
            "name": venue.name,
            "type": venue.venue_type.value,
            "fees": venue.fees,
            "latency_ms": venue.latency_ms,
            "supported_symbols": venue.supported_symbols,
            "performance": self.performance_metrics.get(venue.venue_id, {})
        }
        
        assessment = await self._assess_venue(venue_data)
        
        # Update venue scores
        venue.liquidity_score = assessment.get("liquidity_score", venue.liquidity_score)
        venue.reliability_score = assessment.get("reliability_score", venue.reliability_score)
        venue.last_updated = datetime.utcnow()
        
        logger.info(
            "Venue reassessed",
            venue_id=venue.venue_id,
            new_liquidity=venue.liquidity_score,
            new_reliability=venue.reliability_score
        )
    
    def _update_average(self, old_avg: float, new_value: float, count: int) -> float:
        """Update running average."""
        if count == 0:
            return new_value
        return (old_avg * count + new_value) / (count + 1)
    
    async def get_routing_analytics(self, hours: int = 24) -> Dict[str, Any]:
        """
        Get AI-generated routing analytics.
        """
        cutoff_time = datetime.utcnow() - timedelta(hours=hours)
        recent_routes = [r for r in self.routing_history if r.timestamp > cutoff_time]
        
        if not recent_routes:
            return {"message": "No recent routing data"}
        
        # Calculate statistics
        venue_usage = {}
        strategy_usage = {}
        avg_confidence = np.mean([r.confidence for r in recent_routes])
        
        for route in recent_routes:
            venue_usage[route.venue_id] = venue_usage.get(route.venue_id, 0) + 1
            strategy_usage[route.strategy.value] = strategy_usage.get(route.strategy.value, 0) + 1
        
        # Generate AI insights
        analytics_prompt = f"""
        Analyze routing performance:
        
        Recent Routes: {[asdict(r) for r in recent_routes[-10:]]}
        Venue Usage: {venue_usage}
        Strategy Usage: {strategy_usage}
        Average Confidence: {avg_confidence}
        
        Provide:
        1. Performance insights
        2. Venue recommendations
        3. Strategy effectiveness
        4. Optimization opportunities
        
        Format as JSON.
        """
        
        try:
            response = await self.client.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=1000,
                messages=[{
                    "role": "user",
                    "content": analytics_prompt
                }]
            )
            
            return json.loads(response.content[0].text)
            
        except Exception as e:
            logger.error("Analytics generation failed", error=str(e))
            return {"error": "Analytics unavailable"}
    
    def get_venue_rankings(self, symbol: Optional[str] = None) -> List[Dict[str, Any]]:
        """
        Get ranked list of venues.
        """
        venues = list(self.venues.values())
        
        if symbol:
            venues = [v for v in venues if symbol in v.supported_symbols]
        
        # Calculate composite score
        ranked = []
        for venue in venues:
            performance = self.performance_metrics.get(venue.venue_id, {})
            
            score = (
                venue.liquidity_score * 0.4 +
                venue.reliability_score * 0.3 +
                (1 - min(venue.latency_ms / 100, 1)) * 0.2 +
                performance.get("fill_rate", 0.5) * 0.1
            )
            
            ranked.append({
                "venue_id": venue.venue_id,
                "name": venue.name,
                "type": venue.venue_type.value,
                "score": score,
                "liquidity_score": venue.liquidity_score,
                "reliability_score": venue.reliability_score,
                "latency_ms": venue.latency_ms,
                "fill_rate": performance.get("fill_rate", 0)
            })
        
        # Sort by score
        ranked.sort(key=lambda x: x["score"], reverse=True)
        
        return ranked
