"""
AI Order Management Agent
Replaces traditional OMS with LLM-powered order reasoning and management.
"""

import asyncio
import json
from datetime import datetime
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, asdict
from enum import Enum
import anthropic
import structlog

logger = structlog.get_logger(__name__)


class OrderState(Enum):
    """Order states managed by AI reasoning."""
    CONCEIVED = "conceived"
    VALIDATED = "validated"
    STRATEGIZING = "strategizing"
    EXECUTING = "executing"
    PARTIAL = "partial"
    COMPLETED = "completed"
    CANCELED = "canceled"
    REJECTED = "rejected"


@dataclass
class AIOrder:
    """AI-managed order with reasoning context."""
    order_id: str
    agent_id: str
    symbol: str
    side: str
    quantity: float
    price: Optional[float]
    strategy: str
    reasoning: str
    confidence: float
    state: OrderState
    created_at: datetime
    updated_at: datetime
    metadata: Dict[str, Any]


class OrderManagementAgent:
    """
    AI Order Management Agent that uses LLM reasoning for order lifecycle.
    Replaces traditional OMS with intelligent decision-making.
    """
    
    def __init__(self, anthropic_api_key: str):
        self.client = anthropic.AsyncAnthropic(api_key=anthropic_api_key)
        self.active_orders: Dict[str, AIOrder] = {}
        self.order_history: List[AIOrder] = []
        
        # AI reasoning prompts
        self.order_validation_prompt = """
        You are an AI Order Management Agent for a trading system.
        Analyze this order request and provide reasoning:
        
        Order: {order}
        Market Context: {context}
        Portfolio State: {portfolio}
        
        Provide:
        1. Validation (PASS/FAIL)
        2. Risk assessment (LOW/MEDIUM/HIGH)
        3. Strategic recommendation
        4. Confidence score (0-1)
        5. Reasoning explanation
        
        Format as JSON.
        """
        
        self.execution_strategy_prompt = """
        You are determining execution strategy for an order.
        
        Order: {order}
        Market Data: {market_data}
        Liquidity Analysis: {liquidity}
        
        Recommend:
        1. Execution algorithm (MKT/LIMIT/Iceberg/TWAP/VWAP)
        2. Timing strategy
        3. Venue selection
        4. Splitting logic if applicable
        
        Format as JSON.
        """
    
    async def create_order(self, 
                          agent_id: str,
                          symbol: str,
                          side: str,
                          quantity: float,
                          price: Optional[float] = None,
                          strategy: str = "default",
                          context: Optional[Dict] = None) -> AIOrder:
        """
        Create and validate order using AI reasoning.
        """
        order = AIOrder(
            order_id=f"ORD-{int(datetime.utcnow().timestamp() * 1000)}",
            agent_id=agent_id,
            symbol=symbol,
            side=side,
            quantity=quantity,
            price=price,
            strategy=strategy,
            reasoning="",
            confidence=0.0,
            state=OrderState.CONCEIVED,
            created_at=datetime.utcnow(),
            updated_at=datetime.utcnow(),
            metadata=context or {}
        )
        
        # Validate order with AI
        validation = await self._validate_order_with_ai(order)
        
        if validation["validation"] == "PASS":
            order.state = OrderState.VALIDATED
            order.reasoning = validation["reasoning"]
            order.confidence = validation["confidence"]
            
            # Determine execution strategy
            strategy = await self._determine_execution_strategy(order)
            order.metadata["execution_strategy"] = strategy
            
            self.active_orders[order.order_id] = order
            
            logger.info(
                "Order created and validated",
                order_id=order.order_id,
                symbol=symbol,
                confidence=order.confidence
            )
        else:
            order.state = OrderState.REJECTED
            order.reasoning = validation["reasoning"]
            
            logger.warning(
                "Order rejected by AI",
                order_id=order.order_id,
                reason=order.reasoning
            )
        
        return order
    
    async def _validate_order_with_ai(self, order: AIOrder) -> Dict[str, Any]:
        """
        Use Claude to validate order and assess risk.
        """
        # Gather context
        context = await self._gather_market_context(order.symbol)
        portfolio = await self._get_portfolio_state(order.agent_id)
        
        # Prepare prompt
        prompt = self.order_validation_prompt.format(
            order=asdict(order),
            context=context,
            portfolio=portfolio
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
            
            # Parse AI response
            ai_response = json.loads(response.content[0].text)
            
            return {
                "validation": ai_response.get("validation"),
                "risk_assessment": ai_response.get("risk_assessment"),
                "strategy": ai_response.get("strategic_recommendation"),
                "confidence": float(ai_response.get("confidence", 0)),
                "reasoning": ai_response.get("reasoning", "")
            }
            
        except Exception as e:
            logger.error(
                "AI validation failed",
                order_id=order.order_id,
                error=str(e)
            )
            
            # Fallback to basic validation
            return {
                "validation": "PASS",
                "risk_assessment": "MEDIUM",
                "strategy": "default",
                "confidence": 0.5,
                "reasoning": "AI validation failed, using fallback"
            }
    
    async def _determine_execution_strategy(self, order: AIOrder) -> Dict[str, Any]:
        """
        Use AI to determine optimal execution strategy.
        """
        # Gather market data
        market_data = await self._get_market_data(order.symbol)
        liquidity = await self._analyze_liquidity(order.symbol, order.quantity)
        
        # Prepare prompt
        prompt = self.execution_strategy_prompt.format(
            order=asdict(order),
            market_data=market_data,
            liquidity=liquidity
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
                "AI strategy determination failed",
                order_id=order.order_id,
                error=str(e)
            )
            
            # Fallback strategy
            return {
                "algorithm": "MARKET",
                "timing": "IMMEDIATE",
                "venue": "DEFAULT",
                "splitting": "NONE"
            }
    
    async def modify_order(self, 
                          order_id: str,
                          new_quantity: Optional[float] = None,
                          new_price: Optional[float] = None,
                          reason: str = "") -> bool:
        """
        Modify order using AI reasoning to validate changes.
        """
        if order_id not in self.active_orders:
            return False
        
        order = self.active_orders[order_id]
        
        # Only allow modifications in certain states
        if order.state not in [OrderState.VALIDATED, OrderState.STRATEGIZING]:
            logger.warning(
                "Cannot modify order in current state",
                order_id=order_id,
                state=order.state.value
            )
            return False
        
        # Use AI to validate modification
        modification_prompt = f"""
        Order modification request:
        Original: {asdict(order)}
        Changes: Quantity={new_quantity}, Price={new_price}
        Reason: {reason}
        
        Analyze if this modification is beneficial and safe.
        Respond with ALLOW/DENY and reasoning.
        """
        
        try:
            response = await self.client.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=500,
                messages=[{
                    "role": "user",
                    "content": modification_prompt
                }]
            )
            
            ai_response = response.content[0].text
            
            if "ALLOW" in ai_response:
                if new_quantity:
                    order.quantity = new_quantity
                if new_price:
                    order.price = new_price
                
                order.updated_at = datetime.utcnow()
                order.metadata["modification_reason"] = reason
                
                logger.info(
                    "Order modified",
                    order_id=order_id,
                    new_quantity=new_quantity,
                    new_price=new_price
                )
                
                return True
            else:
                logger.warning(
                    "Order modification denied by AI",
                    order_id=order_id,
                    reasoning=ai_response
                )
                return False
                
        except Exception as e:
            logger.error(
                "AI modification check failed",
                order_id=order_id,
                error=str(e)
            )
            return False
    
    async def cancel_order(self, order_id: str, reason: str = "") -> bool:
        """
        Cancel order with AI validation.
        """
        if order_id not in self.active_orders:
            return False
        
        order = self.active_orders[order_id]
        
        # Use AI to assess cancellation impact
        cancellation_prompt = f"""
        Order cancellation request:
        Order: {asdict(order)}
        Reason: {reason}
        
        Assess the impact of cancellation and recommend PROCEED/ABORT.
        Consider market impact, portfolio effects, and strategic implications.
        """
        
        try:
            response = await self.client.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=500,
                messages=[{
                    "role": "user",
                    "content": cancellation_prompt
                }]
            )
            
            ai_response = response.content[0].text
            
            if "PROCEED" in ai_response:
                order.state = OrderState.CANCELED
                order.updated_at = datetime.utcnow()
                order.metadata["cancellation_reason"] = reason
                
                # Move to history
                self.order_history.append(order)
                del self.active_orders[order_id]
                
                logger.info(
                    "Order canceled",
                    order_id=order_id,
                    reason=reason
                )
                
                return True
            else:
                logger.warning(
                    "Order cancellation denied by AI",
                    order_id=order_id,
                    reasoning=ai_response
                )
                return False
                
        except Exception as e:
            logger.error(
                "AI cancellation check failed",
                order_id=order_id,
                error=str(e)
            )
            return False
    
    async def update_order_state(self, 
                                order_id: str, 
                                new_state: OrderState,
                                execution_details: Optional[Dict] = None):
        """
        Update order state with AI reasoning about state transitions.
        """
        if order_id not in self.active_orders:
            return
        
        order = self.active_orders[order_id]
        old_state = order.state
        
        # AI validates state transition
        transition_prompt = f"""
        State transition request:
        Order: {asdict(order)}
        Transition: {old_state.value} -> {new_state.value}
        Details: {execution_details}
        
        Validate if this transition is logical and safe.
        Respond with VALID/INVALID and reasoning.
        """
        
        try:
            response = await self.client.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=500,
                messages=[{
                    "role": "user",
                    "content": transition_prompt
                }]
            )
            
            ai_response = response.content[0].text
            
            if "VALID" in ai_response:
                order.state = new_state
                order.updated_at = datetime.utcnow()
                
                if execution_details:
                    order.metadata.update(execution_details)
                
                # Move completed orders to history
                if new_state in [OrderState.COMPLETED, OrderState.CANCELED, OrderState.REJECTED]:
                    self.order_history.append(order)
                    del self.active_orders[order_id]
                
                logger.info(
                    "Order state updated",
                    order_id=order_id,
                    old_state=old_state.value,
                    new_state=new_state.value
                )
            else:
                logger.warning(
                    "State transition denied by AI",
                    order_id=order_id,
                    reasoning=ai_response
                )
                
        except Exception as e:
            logger.error(
                "AI state transition check failed",
                order_id=order_id,
                error=str(e)
            )
    
    async def _gather_market_context(self, symbol: str) -> Dict[str, Any]:
        """Gather market context for AI reasoning."""
        # This would connect to market data feeds
        return {
            "symbol": symbol,
            "price": 50000.0,
            "volume": 1000000,
            "volatility": 0.02,
            "trend": "NEUTRAL"
        }
    
    async def _get_portfolio_state(self, agent_id: str) -> Dict[str, Any]:
        """Get portfolio state for AI reasoning."""
        # This would connect to position management
        return {
            "agent_id": agent_id,
            "cash": 100000.0,
            "positions": {},
            "leverage": 1.0,
            "risk_score": 0.3
        }
    
    async def _get_market_data(self, symbol: str) -> Dict[str, Any]:
        """Get detailed market data."""
        return {
            "bid": 49999.0,
            "ask": 50001.0,
            "spread": 2.0,
            "depth": {"bid": [[49999, 100], [49998, 200]], "ask": [[50001, 100], [50002, 200]]}
        }
    
    async def _analyze_liquidity(self, symbol: str, quantity: float) -> Dict[str, Any]:
        """Analyze liquidity for order."""
        return {
            "sufficient": True,
            "impact_estimate": 0.001,
            "recommended_venues": ["EXCHANGE1", "EXCHANGE2"]
        }
    
    def get_order(self, order_id: str) -> Optional[AIOrder]:
        """Get order by ID."""
        return self.active_orders.get(order_id)
    
    def get_active_orders(self, agent_id: Optional[str] = None) -> List[AIOrder]:
        """Get all active orders, optionally filtered by agent."""
        orders = list(self.active_orders.values())
        if agent_id:
            orders = [o for o in orders if o.agent_id == agent_id]
        return orders
    
    def get_order_history(self, agent_id: Optional[str] = None, limit: int = 100) -> List[AIOrder]:
        """Get order history."""
        history = self.order_history
        if agent_id:
            history = [o for o in history if o.agent_id == agent_id]
        return history[-limit:]
