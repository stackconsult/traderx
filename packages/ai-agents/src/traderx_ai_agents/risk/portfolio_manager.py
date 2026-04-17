"""
Portfolio manager agent for making investment decisions based on research and risk analysis.
"""

from typing import Dict, Any, List, Optional, Tuple
from dataclasses import dataclass
from enum import Enum
import asyncio
import logging
from datetime import datetime, timedelta

from .base import BaseRiskAgent, RiskAssessment, RiskLevel
from ..analysis.base import AnalysisResult

logger = logging.getLogger(__name__)


class Recommendation(Enum):
    """Investment recommendation levels"""
    STRONG_BUY = "strong_buy"
    BUY = "buy"
    OVERWEIGHT = "overweight"
    HOLD = "hold"
    UNDERWEIGHT = "underweight"
    SELL = "sell"
    STRONG_SELL = "strong_sell"


@dataclass
class PortfolioDecision:
    """Portfolio management decision"""
    symbol: str
    recommendation: Recommendation
    position_size: float  # 0.0 to 1.0 (percentage of portfolio)
    entry_price: Optional[float]
    target_price: Optional[float]
    stop_loss: Optional[float]
    time_horizon: str  # short/medium/long
    conviction: float  # 0.0 to 1.0
    rationale: str
    key_risks: List[str]
    catalysts: List[str]
    timestamp: datetime


class PortfolioManager(BaseRiskAgent):
    """
    Portfolio manager that synthesizes research and risk analysis to make investment decisions.
    
    Responsibilities:
    - Evaluate research team conclusions
    - Assess risk-adjusted returns
    - Determine position sizing
    - Set entry/exit parameters
    - Manage portfolio allocation
    """
    
    def __init__(
        self,
        max_position_size: float = 0.10,  # 10% max per position
        risk_tolerance: float = 0.15,  # 15% portfolio risk tolerance
        rebalance_threshold: float = 0.05  # 5% deviation triggers rebalance
    ):
        super().__init__("portfolio_manager")
        self.max_position_size = max_position_size
        self.risk_tolerance = risk_tolerance
        self.rebalance_threshold = rebalance_threshold
        
        # Portfolio state
        self.current_positions: Dict[str, float] = {}
        self.portfolio_value: float = 0.0
        self.cash_available: float = 0.0
        
    async def assess_risk(self, input_data: Dict[str, Any]) -> RiskAssessment:
        """Assess portfolio risk for a potential investment"""
        symbol = input_data.get("symbol", "")
        research_data = input_data.get("research_data", {})
        risk_data = input_data.get("risk_data", {})
        
        # Extract risk factors
        risk_factors = {}
        
        # Research confidence risk
        moderated_conclusion = research_data.get("moderated_conclusion", {})
        research_confidence = moderated_conclusion.get("confidence", 0.5)
        risk_factors["research_uncertainty"] = 1.0 - research_confidence
        
        # Bull/bear disagreement risk
        bull_confidence = moderated_conclusion.get("bull_confidence", 0.5)
        bear_confidence = moderated_conclusion.get("bear_confidence", 0.5)
        disagreement = abs(bull_confidence - bear_confidence)
        risk_factors["analyst_disagreement"] = 1.0 - disagreement
        
        # Market risk
        market_volatility = risk_data.get("volatility", 0.2)
        risk_factors["market_volatility"] = min(market_volatility * 2, 1.0)
        
        # Liquidity risk
        liquidity_score = risk_data.get("liquidity_score", 0.8)
        risk_factors["liquidity_risk"] = 1.0 - liquidity_score
        
        # Calculate overall risk score
        risk_score = self.calculate_risk_score(risk_factors)
        risk_level = self.determine_risk_level(risk_score)
        
        # Generate risk factors list
        factors = []
        if risk_factors["research_uncertainty"] > 0.5:
            factors.append("Low research confidence")
        if risk_factors["analyst_disagreement"] > 0.5:
            factors.append("High analyst disagreement")
        if risk_factors["market_volatility"] > 0.5:
            factors.append("High market volatility")
        if risk_factors["liquidity_risk"] > 0.5:
            factors.append("Low liquidity")
            
        # Generate mitigation strategies
        mitigation = []
        if risk_score > 0.6:
            mitigation.append("Reduce position size")
            mitigation.append("Set tighter stop-loss")
        if risk_factors["market_volatility"] > 0.5:
            mitigation.append("Use dollar-cost averaging")
        if risk_factors["liquidity_risk"] > 0.5:
            mitigation.append("Monitor trading volume")
            
        return RiskAssessment(
            agent_name=self.name,
            risk_level=risk_level,
            risk_score=risk_score,
            factors=factors,
            mitigation=mitigation,
            confidence=research_confidence,
            timestamp=datetime.now()
        )
        
    async def make_portfolio_decision(
        self,
        symbol: str,
        research_data: Dict[str, Any],
        risk_data: Dict[str, Any],
        current_price: float
    ) -> PortfolioDecision:
        """
        Make portfolio decision based on research and risk analysis.
        
        Args:
            symbol: Stock symbol
            research_data: Research team analysis
            risk_data: Risk assessment data
            current_price: Current market price
            
        Returns:
            Portfolio decision with all parameters
        """
        # Get recommendation from moderated conclusion
        moderated_conclusion = research_data.get("moderated_conclusion", {})
        recommendation_str = moderated_conclusion.get("recommendation", "HOLD")
        
        # Map string recommendation to enum
        recommendation_map = {
            "STRONG BUY": Recommendation.STRONG_BUY,
            "BUY": Recommendation.BUY,
            "OVERWEIGHT": Recommendation.OVERWEIGHT,
            "HOLD": Recommendation.HOLD,
            "UNDERWEIGHT": Recommendation.UNDERWEIGHT,
            "SELL": Recommendation.SELL,
            "STRONG SELL": Recommendation.STRONG_SELL
        }
        
        recommendation = recommendation_map.get(recommendation_str, Recommendation.HOLD)
        
        # Calculate conviction based on confidence and risk
        confidence = moderated_conclusion.get("confidence", 0.5)
        risk_assessment = await self.assess_risk({
            "symbol": symbol,
            "research_data": research_data,
            "risk_data": risk_data
        })
        
        # Adjust conviction based on risk
        risk_adjusted_confidence = confidence * (1.0 - risk_assessment.risk_score * 0.5)
        conviction = max(0.1, min(1.0, risk_adjusted_confidence))
        
        # Determine position size based on conviction and risk
        base_position = conviction * self.max_position_size
        risk_adjusted_position = base_position * (1.0 - risk_assessment.risk_score)
        position_size = max(0.0, min(self.max_position_size, risk_adjusted_position))
        
        # Set price targets
        entry_price = current_price
        target_price = None
        stop_loss = None
        
        if recommendation in [Recommendation.BUY, Recommendation.STRONG_BUY, Recommendation.OVERWEIGHT]:
            # Set upside target (10-30% based on conviction)
            upside_target = 0.10 + (conviction * 0.20)
            target_price = current_price * (1.0 + upside_target)
            
            # Set stop loss (5-15% based on risk)
            stop_loss_pct = 0.05 + (risk_assessment.risk_score * 0.10)
            stop_loss = current_price * (1.0 - stop_loss_pct)
            
        elif recommendation in [Recommendation.SELL, Recommendation.STRONG_SELL, Recommendation.UNDERWEIGHT]:
            # Set downside target for short positions
            downside_target = 0.10 + (conviction * 0.20)
            target_price = current_price * (1.0 - downside_target)
            
            # Set stop loss for shorts
            stop_loss_pct = 0.05 + (risk_assessment.risk_score * 0.10)
            stop_loss = current_price * (1.0 + stop_loss_pct)
            
        # Determine time horizon
        time_horizon = "medium"  # Default
        if conviction > 0.8 and risk_assessment.risk_score < 0.3:
            time_horizon = "long"
        elif conviction < 0.5 or risk_assessment.risk_score > 0.6:
            time_horizon = "short"
            
        # Extract rationale and key points
        rationale = moderated_conclusion.get("moderated_thesis", "")
        
        # Extract key risks and catalysts
        key_risks = risk_assessment.factors
        catalysts = self._extract_catalysts(research_data)
        
        return PortfolioDecision(
            symbol=symbol,
            recommendation=recommendation,
            position_size=position_size,
            entry_price=entry_price,
            target_price=target_price,
            stop_loss=stop_loss,
            time_horizon=time_horizon,
            conviction=conviction,
            rationale=rationale,
            key_risks=key_risks,
            catalysts=catalysts,
            timestamp=datetime.now()
        )
        
    def _extract_catalysts(self, research_data: Dict[str, Any]) -> List[str]:
        """Extract key catalysts from research data"""
        catalysts = []
        
        # From bull thesis
        bull_thesis = research_data.get("bull_thesis", {}).get("thesis", "")
        if "growth" in bull_thesis.lower():
            catalysts.append("Growth acceleration")
        if "launch" in bull_thesis.lower():
            catalysts.append("Product launch")
        if "expansion" in bull_thesis.lower():
            catalysts.append("Market expansion")
            
        # From bear thesis (as potential positive catalysts if risks pass)
        bear_thesis = research_data.get("bear_thesis", {}).get("thesis", "")
        if "regulation" in bear_thesis.lower():
            catalysts.append("Regulatory clarity")
        if "competition" in bear_thesis.lower():
            catalysts.append("Competitive advantage confirmation")
            
        return catalysts[:5]  # Limit to top 5
        
    async def update_portfolio_state(
        self,
        positions: Dict[str, float],
        portfolio_value: float,
        cash_available: float
    ):
        """Update current portfolio state"""
        self.current_positions = positions
        self.portfolio_value = portfolio_value
        self.cash_available = cash_available
        
    async def check_rebalance_needed(self, symbol: str, target_weight: float) -> bool:
        """Check if portfolio rebalancing is needed"""
        current_weight = self.current_positions.get(symbol, 0.0)
        deviation = abs(target_weight - current_weight)
        
        return deviation > self.rebalance_threshold
        
    async def calculate_portfolio_risk(self) -> Dict[str, Any]:
        """Calculate overall portfolio risk metrics"""
        if not self.current_positions:
            return {"total_risk": 0.0, "concentration_risk": 0.0}
            
        # Calculate concentration risk (largest position)
        max_position = max(self.current_positions.values())
        concentration_risk = max_position
        
        # Calculate portfolio beta (simplified)
        portfolio_beta = 1.0  # Would need individual betas in production
        
        # Calculate total portfolio risk
        total_risk = (concentration_risk + portfolio_beta) / 2.0
        
        return {
            "total_risk": total_risk,
            "concentration_risk": concentration_risk,
            "portfolio_beta": portfolio_beta,
            "position_count": len(self.current_positions),
            "timestamp": datetime.now()
        }
