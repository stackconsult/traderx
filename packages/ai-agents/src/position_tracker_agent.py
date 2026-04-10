"""
AI Position Tracker with Natural Language P&L
Replaces traditional position management with AI-powered analysis and reporting.
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


class PositionStatus(Enum):
    """Position status determined by AI."""
    ACTIVE = "active"
    CLOSING = "closing"
    CLOSED = "closed"
    HEDGED = "hedged"
    AT_RISK = "at_risk"
    OPTIMAL = "optimal"


@dataclass
class AIPosition:
    """AI-managed position with intelligent tracking."""
    symbol: str
    agent_id: str
    quantity: float
    avg_price: float
    current_price: float
    unrealized_pnl: float
    realized_pnl: float
    status: PositionStatus
    risk_score: float
    ai_analysis: str
    recommendations: List[str]
    created_at: datetime
    updated_at: datetime
    trades_count: int
    metadata: Dict[str, Any]


class PositionTrackerAgent:
    """
    AI Position Tracker that provides intelligent position management
    and natural language P&L analysis.
    """
    
    def __init__(self, anthropic_api_key: str):
        self.client = anthropic.AsyncAnthropic(api_key=anthropic_api_key)
        self.positions: Dict[str, AIPosition] = {}  # key: agent_id:symbol
        self.pnl_history: List[Dict] = []
        
        # AI analysis prompts
        self.position_analysis_prompt = """
        You are an AI Position Analyst. Analyze this position:
        
        Position: {position}
        Market Context: {market}
        Recent Trades: {trades}
        
        Provide:
        1. Position status (ACTIVE/CLOSING/CLOSED/HEDGED/AT_RISK/OPTIMAL)
        2. Risk score (0-1)
        3. P&L analysis in natural language
        4. Strategic recommendations (list)
        5. Key insights and warnings
        
        Format as JSON.
        """
        
        self.pnl_narrative_prompt = """
        Create a natural language P&L report for this portfolio:
        
        Positions: {positions}
        Time Period: {period}
        Performance Metrics: {metrics}
        
        Generate:
        1. Executive summary
        2. Position-by-position analysis
        3. Market impact assessment
        4. Risk observations
        5. Forward-looking outlook
        
        Format as JSON with narrative text.
        """
    
    async def update_position(self,
                             agent_id: str,
                             symbol: str,
                             side: str,
                             quantity: float,
                             price: float,
                             trade_id: str) -> AIPosition:
        """
        Update position with new trade and AI analysis.
        """
        position_key = f"{agent_id}:{symbol}"
        
        if position_key not in self.positions:
            # Create new position
            position = AIPosition(
                symbol=symbol,
                agent_id=agent_id,
                quantity=quantity if side == "BUY" else -quantity,
                avg_price=price,
                current_price=price,
                unrealized_pnl=0.0,
                realized_pnl=0.0,
                status=PositionStatus.ACTIVE,
                risk_score=0.5,
                ai_analysis="New position opened",
                recommendations=["Monitor market conditions"],
                created_at=datetime.utcnow(),
                updated_at=datetime.utcnow(),
                trades_count=1,
                metadata={"last_trade_id": trade_id}
            )
        else:
            # Update existing position
            position = self.positions[position_key]
            
            if side == "BUY":
                # Calculate new average price
                old_qty = position.quantity
                new_qty = old_qty + quantity
                if new_qty != 0:
                    position.avg_price = (position.avg_price * abs(old_qty) + price * quantity) / abs(new_qty)
                position.quantity = new_qty
            else:  # SELL
                if abs(quantity) >= abs(position.quantity):
                    # Closing position
                    realized = (price - position.avg_price) * abs(position.quantity)
                    position.realized_pnl += realized
                    position.quantity = position.quantity + quantity  # quantity is negative for SELL
                else:
                    # Partial close
                    realized = (price - position.avg_price) * quantity
                    position.realized_pnl += realized
                    position.quantity += quantity
            
            position.current_price = price
            position.trades_count += 1
            position.metadata["last_trade_id"] = trade_id
        
        # AI analysis of position
        analysis = await self._analyze_position_with_ai(position)
        
        position.status = PositionStatus(analysis["status"])
        position.risk_score = analysis["risk_score"]
        position.ai_analysis = analysis["analysis"]
        position.recommendations = analysis["recommendations"]
        position.updated_at = datetime.utcnow()
        
        # Calculate unrealized P&L
        if position.quantity != 0:
            position.unrealized_pnl = (position.current_price - position.avg_price) * position.quantity
        else:
            position.unrealized_pnl = 0.0
            if position.status != PositionStatus.CLOSED:
                position.status = PositionStatus.CLOSED
        
        self.positions[position_key] = position
        
        # Log position update
        logger.info(
            "Position updated",
            agent_id=agent_id,
            symbol=symbol,
            quantity=position.quantity,
            unrealized_pnl=position.unrealized_pnl,
            realized_pnl=position.realized_pnl,
            status=position.status.value
        )
        
        return position
    
    async def _analyze_position_with_ai(self, position: AIPosition) -> Dict[str, Any]:
        """
        Use AI to analyze position and provide insights.
        """
        # Gather context
        market_context = await self._get_market_context(position.symbol)
        recent_trades = await self._get_recent_trades(position.agent_id, position.symbol)
        
        # Prepare prompt
        prompt = self.position_analysis_prompt.format(
            position=asdict(position),
            market=market_context,
            trades=recent_trades
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
            
            ai_response = json.loads(response.content[0].text)
            
            return {
                "status": ai_response.get("status", "ACTIVE"),
                "risk_score": float(ai_response.get("risk_score", 0.5)),
                "analysis": ai_response.get("analysis", ""),
                "recommendations": ai_response.get("recommendations", [])
            }
            
        except Exception as e:
            logger.error(
                "AI position analysis failed",
                symbol=position.symbol,
                error=str(e)
            )
            
            # Fallback analysis
            return {
                "status": "ACTIVE",
                "risk_score": 0.5,
                "analysis": "AI analysis unavailable",
                "recommendations": ["Monitor position"]
            }
    
    async def generate_portfolio_report(self, 
                                       agent_id: str,
                                       period: str = "daily") -> Dict[str, Any]:
        """
        Generate natural language portfolio P&L report.
        """
        # Get agent's positions
        agent_positions = [p for p in self.positions.values() if p.agent_id == agent_id]
        
        if not agent_positions:
            return {"error": "No positions found"}
        
        # Calculate metrics
        total_unrealized = sum(p.unrealized_pnl for p in agent_positions)
        total_realized = sum(p.realized_pnl for p in agent_positions)
        total_pnl = total_unrealized + total_realized
        
        metrics = {
            "total_unrealized_pnl": total_unrealized,
            "total_realized_pnl": total_realized,
            "total_pnl": total_pnl,
            "position_count": len(agent_positions),
            "risk_score_avg": np.mean([p.risk_score for p in agent_positions])
        }
        
        # Generate narrative with AI
        narrative = await self._generate_pnl_narrative(agent_positions, period, metrics)
        
        return {
            "agent_id": agent_id,
            "period": period,
            "generated_at": datetime.utcnow().isoformat(),
            "metrics": metrics,
            "narrative": narrative,
            "positions": [asdict(p) for p in agent_positions]
        }
    
    async def _generate_pnl_narrative(self, 
                                     positions: List[AIPosition],
                                     period: str,
                                     metrics: Dict) -> Dict[str, str]:
        """
        Use AI to generate natural language P&L narrative.
        """
        prompt = self.pnl_narrative_prompt.format(
            positions=[asdict(p) for p in positions],
            period=period,
            metrics=metrics
        )
        
        try:
            response = await self.client.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=2000,
                messages=[{
                    "role": "user",
                    "content": prompt
                }]
            )
            
            return json.loads(response.content[0].text)
            
        except Exception as e:
            logger.error(
                "AI narrative generation failed",
                error=str(e)
            )
            
            return {
                "executive_summary": f"Portfolio P&L: ${metrics['total_pnl']:.2f}",
                "analysis": "AI narrative unavailable",
                "outlook": "Monitor positions"
            }
    
    async def get_position_recommendations(self, 
                                          agent_id: str,
                                          symbol: Optional[str] = None) -> List[Dict[str, Any]]:
        """
        Get AI-generated recommendations for positions.
        """
        positions = [p for p in self.positions.values() if p.agent_id == agent_id]
        if symbol:
            positions = [p for p in positions if p.symbol == symbol]
        
        recommendations = []
        
        for position in positions:
            if position.recommendations:
                recommendations.append({
                    "symbol": position.symbol,
                    "quantity": position.quantity,
                    "recommendations": position.recommendations,
                    "risk_score": position.risk_score,
                    "reasoning": position.ai_analysis
                })
        
        return recommendations
    
    async def assess_portfolio_risk(self, agent_id: str) -> Dict[str, Any]:
        """
        AI-powered portfolio risk assessment.
        """
        positions = [p for p in self.positions.values() if p.agent_id == agent_id]
        
        if not positions:
            return {"risk_score": 0.0, "status": "NO_POSITIONS"}
        
        # Calculate risk metrics
        risk_scores = [p.risk_score for p in positions]
        total_exposure = sum(abs(p.quantity * p.current_price) for p in positions)
        
        # AI risk assessment
        risk_prompt = f"""
        Assess portfolio risk:
        Positions: {[asdict(p) for p in positions]}
        Total Exposure: ${total_exposure:,.2f}
        Average Risk Score: {np.mean(risk_scores):.3f}
        
        Provide:
        1. Overall risk score (0-1)
        2. Risk level (LOW/MEDIUM/HIGH/CRITICAL)
        3. Main risk factors
        4. Mitigation strategies
        
        Format as JSON.
        """
        
        try:
            response = await self.client.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=1000,
                messages=[{
                    "role": "user",
                    "content": risk_prompt
                }]
            )
            
            ai_risk = json.loads(response.content[0].text)
            
            return {
                "risk_score": float(ai_risk.get("overall_risk_score", np.mean(risk_scores))),
                "risk_level": ai_risk.get("risk_level", "MEDIUM"),
                "risk_factors": ai_risk.get("risk_factors", []),
                "mitigation": ai_risk.get("mitigation_strategies", []),
                "total_exposure": total_exposure,
                "position_count": len(positions)
            }
            
        except Exception as e:
            logger.error("AI risk assessment failed", error=str(e))
            
            return {
                "risk_score": np.mean(risk_scores),
                "risk_level": "MEDIUM",
                "risk_factors": ["AI assessment unavailable"],
                "mitigation": ["Monitor positions"],
                "total_exposure": total_exposure,
                "position_count": len(positions)
            }
    
    async def close_position(self, 
                            agent_id: str,
                            symbol: str,
                            close_price: float,
                            reason: str = "") -> Optional[AIPosition]:
        """
        Close position with AI validation.
        """
        position_key = f"{agent_id}:{symbol}"
        
        if position_key not in self.positions:
            return None
        
        position = self.positions[position_key]
        
        if position.quantity == 0:
            return position
        
        # Calculate final P&L
        final_pnl = (close_price - position.avg_price) * position.quantity
        position.realized_pnl += final_pnl
        position.quantity = 0
        position.current_price = close_price
        position.unrealized_pnl = 0.0
        position.status = PositionStatus.CLOSED
        position.updated_at = datetime.utcnow()
        position.metadata["close_reason"] = reason
        
        logger.info(
            "Position closed",
            agent_id=agent_id,
            symbol=symbol,
            final_pnl=final_pnl,
            reason=reason
        )
        
        return position
    
    async def _get_market_context(self, symbol: str) -> Dict[str, Any]:
        """Get market context for AI analysis."""
        return {
            "symbol": symbol,
            "price": 50000.0,
            "change": "+0.5%",
            "volume": "10M",
            "volatility": 0.02,
            "trend": "BULLISH"
        }
    
    async def _get_recent_trades(self, agent_id: str, symbol: str) -> List[Dict]:
        """Get recent trades for position analysis."""
        return []  # Would fetch from trade history
    
    def get_position(self, agent_id: str, symbol: str) -> Optional[AIPosition]:
        """Get specific position."""
        position_key = f"{agent_id}:{symbol}"
        return self.positions.get(position_key)
    
    def get_all_positions(self, agent_id: Optional[str] = None) -> List[AIPosition]:
        """Get all positions, optionally filtered by agent."""
        positions = list(self.positions.values())
        if agent_id:
            positions = [p for p in positions if p.agent_id == agent_id]
        return positions
