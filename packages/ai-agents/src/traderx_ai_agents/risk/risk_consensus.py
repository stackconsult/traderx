"""
Risk consensus agent for building consensus among multiple risk perspectives.
"""

from typing import Dict, Any, List, Optional, Tuple
from dataclasses import dataclass
import asyncio
import logging
from datetime import datetime, timedelta
from enum import Enum

from .base import BaseRiskAgent, RiskAssessment, RiskLevel
from ..analysis.base import AnalysisResult

logger = logging.getLogger(__name__)


class ConsensusMethod(Enum):
    """Methods for building consensus"""
    WEIGHTED_AVERAGE = "weighted_average"
    MAJORITY_VOTE = "majority_vote"
    EXPERT_WEIGHTED = "expert_weighted"
    RISK_AVERSE = "risk_averse"
    RISK_TOLERANT = "risk_tolerant"


@dataclass
class RiskAgent:
    """Individual risk agent configuration"""
    name: str
    weight: float  # 0.0 to 1.0
    expertise: str  # market, credit, operational, etc.
    risk_tolerance: float  # 0.0 to 1.0


class RiskConsensusAgent(BaseRiskAgent):
    """
    Builds consensus among multiple risk agents to determine overall risk assessment.
    
    Features:
    - Multiple consensus methods
    - Configurable risk agents
    - Disagreement detection
    - Confidence weighting
    - Historical performance tracking
    """
    
    def __init__(
        self,
        consensus_method: ConsensusMethod = ConsensusMethod.WEIGHTED_AVERAGE,
        disagreement_threshold: float = 0.3,
        min_confidence: float = 0.6
    ):
        super().__init__("risk_consensus_agent")
        self.consensus_method = consensus_method
        self.disagreement_threshold = disagreement_threshold
        self.min_confidence = min_confidence
        
        # Initialize default risk agents
        self.risk_agents = [
            RiskAgent("market_risk", 0.3, "market", 0.5),
            RiskAgent("credit_risk", 0.2, "credit", 0.4),
            RiskAgent("operational_risk", 0.2, "operational", 0.6),
            RiskAgent("liquidity_risk", 0.15, "liquidity", 0.5),
            RiskAgent("regulatory_risk", 0.15, "regulatory", 0.3)
        ]
        
        # Track historical performance
        self.agent_performance: Dict[str, float] = {}
        
    async def assess_risk(self, input_data: Dict[str, Any]) -> RiskAssessment:
        """
        Assess risk by building consensus among multiple risk agents.
        
        Args:
            input_data: Contains symbol, market data, and risk factors
            
        Returns:
            Consensus risk assessment
        """
        symbol = input_data.get("symbol", "")
        
        # Collect risk assessments from all agents
        individual_assessments = []
        for agent in self.risk_agents:
            assessment = await self._assess_individual_risk(agent, input_data)
            individual_assessments.append((agent, assessment))
            
        # Build consensus based on method
        consensus_assessment = await self._build_consensus(individual_assessments)
        
        # Check for disagreement
        disagreement_score = self._calculate_disagreement(individual_assessments)
        
        # Adjust confidence based on disagreement
        adjusted_confidence = self._adjust_confidence(
            consensus_assessment.confidence,
            disagreement_score
        )
        
        # Generate consensus factors and mitigation
        consensus_factors = self._aggregate_factors(individual_assessments)
        consensus_mitigation = self._aggregate_mitigation(individual_assessments)
        
        return RiskAssessment(
            agent_name=self.name,
            risk_level=consensus_assessment.risk_level,
            risk_score=consensus_assessment.risk_score,
            factors=consensus_factors,
            mitigation=consensus_mitigation,
            confidence=adjusted_confidence,
            timestamp=datetime.now()
        )
        
    async def _assess_individual_risk(
        self,
        agent: RiskAgent,
        input_data: Dict[str, Any]
    ) -> RiskAssessment:
        """Assess risk from individual agent perspective"""
        symbol = input_data.get("symbol", "")
        market_data = input_data.get("market_data", {})
        
        # Simulate individual agent risk assessment
        # In production, each agent would have its own assessment logic
        
        risk_factors = {}
        
        # Market risk agent
        if agent.expertise == "market":
            volatility = market_data.get("volatility", 0.2)
            beta = market_data.get("beta", 1.0)
            risk_factors["volatility"] = min(volatility * 2, 1.0)
            risk_factors["beta_risk"] = min(abs(beta - 1.0) * 0.5, 1.0)
            
        # Credit risk agent
        elif agent.expertise == "credit":
            debt_ratio = market_data.get("debt_to_equity", 0.5)
            interest_coverage = market_data.get("interest_coverage", 5.0)
            risk_factors["debt_risk"] = min(debt_ratio, 1.0)
            risk_factors["coverage_risk"] = max(0, 1.0 - interest_coverage / 10.0)
            
        # Operational risk agent
        elif agent.expertise == "operational":
            complexity_score = market_data.get("complexity_score", 0.3)
            geographic_risk = market_data.get("geographic_risk", 0.2)
            risk_factors["complexity"] = complexity_score
            risk_factors["geographic"] = geographic_risk
            
        # Liquidity risk agent
        elif agent.expertise == "liquidity":
            volume_ratio = market_data.get("volume_ratio", 1.0)
            bid_ask_spread = market_data.get("bid_ask_spread", 0.01)
            risk_factors["volume_risk"] = max(0, 1.0 - volume_ratio)
            risk_factors["spread_risk"] = min(bid_ask_spread * 50, 1.0)
            
        # Regulatory risk agent
        elif agent.expertise == "regulatory":
            regulatory_score = market_data.get("regulatory_risk", 0.2)
            compliance_risk = market_data.get("compliance_risk", 0.1)
            risk_factors["regulatory"] = regulatory_score
            risk_factors["compliance"] = compliance_risk
            
        # Calculate risk score
        risk_score = self.calculate_risk_score(risk_factors)
        
        # Adjust based on agent's risk tolerance
        adjusted_score = risk_score * (1.0 + (agent.risk_tolerance - 0.5) * 0.4)
        adjusted_score = max(0.0, min(1.0, adjusted_score))
        
        # Generate factors list
        factors = [f"{name}: {value:.2f}" for name, value in risk_factors.items() if value > 0.3]
        
        # Generate mitigation strategies
        mitigation = []
        if adjusted_score > 0.6:
            mitigation.append(f"Monitor {agent.expertise} risk closely")
            mitigation.append(f"Implement {agent.expertise} hedges")
        elif adjusted_score > 0.4:
            mitigation.append(f"Regular {agent.expertise} reviews")
            
        return RiskAssessment(
            agent_name=agent.name,
            risk_level=self.determine_risk_level(adjusted_score),
            risk_score=adjusted_score,
            factors=factors,
            mitigation=mitigation,
            confidence=0.8,  # Individual agent confidence
            timestamp=datetime.now()
        )
        
    async def _build_consensus(
        self,
        individual_assessments: List[Tuple[RiskAgent, RiskAssessment]]
    ) -> RiskAssessment:
        """Build consensus based on selected method"""
        if self.consensus_method == ConsensusMethod.WEIGHTED_AVERAGE:
            return self._weighted_average_consensus(individual_assessments)
        elif self.consensus_method == ConsensusMethod.MAJORITY_VOTE:
            return self._majority_vote_consensus(individual_assessments)
        elif self.consensus_method == ConsensusMethod.EXPERT_WEIGHTED:
            return self._expert_weighted_consensus(individual_assessments)
        elif self.consensus_method == ConsensusMethod.RISK_AVERSE:
            return self._risk_averse_consensus(individual_assessments)
        elif self.consensus_method == ConsensusMethod.RISK_TOLERANT:
            return self._risk_tolerant_consensus(individual_assessments)
        else:
            return self._weighted_average_consensus(individual_assessments)
            
    def _weighted_average_consensus(
        self,
        individual_assessments: List[Tuple[RiskAgent, RiskAssessment]]
    ) -> RiskAssessment:
        """Calculate weighted average of risk scores"""
        total_weight = sum(agent.weight for agent, _ in individual_assessments)
        
        if total_weight == 0:
            return individual_assessments[0][1]  # Fallback
            
        weighted_score = sum(
            assessment.risk_score * agent.weight
            for agent, assessment in individual_assessments
        ) / total_weight
        
        # Determine risk level
        risk_level = self.determine_risk_level(weighted_score)
        
        # Average confidence
        avg_confidence = sum(
            assessment.confidence * agent.weight
            for agent, assessment in individual_assessments
        ) / total_weight
        
        return RiskAssessment(
            agent_name="consensus",
            risk_level=risk_level,
            risk_score=weighted_score,
            factors=[],  # Will be aggregated separately
            mitigation=[],
            confidence=avg_confidence,
            timestamp=datetime.now()
        )
        
    def _majority_vote_consensus(
        self,
        individual_assessments: List[Tuple[RiskAgent, RiskAssessment]]
    ) -> RiskAssessment:
        """Use majority vote for risk level"""
        risk_level_counts = {}
        
        for _, assessment in individual_assessments:
            level = assessment.risk_level.value
            risk_level_counts[level] = risk_level_counts.get(level, 0) + 1
            
        # Find majority
        majority_level = max(risk_level_counts, key=risk_level_counts.get)
        
        # Average score for majority level
        majority_assessments = [
            assessment for _, assessment in individual_assessments
            if assessment.risk_level.value == majority_level
        ]
        
        avg_score = sum(a.risk_score for a in majority_assessments) / len(majority_assessments)
        avg_confidence = sum(a.confidence for a in majority_assessments) / len(majority_assessments)
        
        return RiskAssessment(
            agent_name="consensus",
            risk_level=RiskLevel(majority_level),
            risk_score=avg_score,
            factors=[],
            mitigation=[],
            confidence=avg_confidence,
            timestamp=datetime.now()
        )
        
    def _expert_weighted_consensus(
        self,
        individual_assessments: List[Tuple[RiskAgent, RiskAssessment]]
    ) -> RiskAssessment:
        """Weight by expertise and historical performance"""
        total_weight = 0
        weighted_score = 0
        weighted_confidence = 0
        
        for agent, assessment in individual_assessments:
            # Get performance multiplier (default to 1.0)
            performance_multiplier = self.agent_performance.get(agent.name, 1.0)
            
            # Calculate effective weight
            effective_weight = agent.weight * performance_multiplier
            total_weight += effective_weight
            
            weighted_score += assessment.risk_score * effective_weight
            weighted_confidence += assessment.confidence * effective_weight
            
        if total_weight > 0:
            weighted_score /= total_weight
            weighted_confidence /= total_weight
            
        return RiskAssessment(
            agent_name="consensus",
            risk_level=self.determine_risk_level(weighted_score),
            risk_score=weighted_score,
            factors=[],
            mitigation=[],
            confidence=weighted_confidence,
            timestamp=datetime.now()
        )
        
    def _risk_averse_consensus(
        self,
        individual_assessments: List[Tuple[RiskAgent, RiskAssessment]]
    ) -> RiskAssessment:
        """Take more conservative (higher risk) view"""
        # Use 75th percentile of risk scores
        scores = [assessment.risk_score for _, assessment in individual_assessments]
        scores.sort()
        
        conservative_score = scores[int(len(scores) * 0.75)] if scores else 0.5
        
        # Lower confidence for risk-averse approach
        avg_confidence = sum(assessment.confidence for _, assessment in individual_assessments) / len(individual_assessments)
        adjusted_confidence = avg_confidence * 0.9
        
        return RiskAssessment(
            agent_name="consensus",
            risk_level=self.determine_risk_level(conservative_score),
            risk_score=conservative_score,
            factors=[],
            mitigation=[],
            confidence=adjusted_confidence,
            timestamp=datetime.now()
        )
        
    def _risk_tolerant_consensus(
        self,
        individual_assessments: List[Tuple[RiskAgent, RiskAssessment]]
    ) -> RiskAssessment:
        """Take more tolerant (lower risk) view"""
        # Use 25th percentile of risk scores
        scores = [assessment.risk_score for _, assessment in individual_assessments]
        scores.sort()
        
        tolerant_score = scores[int(len(scores) * 0.25)] if scores else 0.5
        
        # Higher confidence for risk-tolerant approach
        avg_confidence = sum(assessment.confidence for _, assessment in individual_assessments) / len(individual_assessments)
        adjusted_confidence = min(1.0, avg_confidence * 1.1)
        
        return RiskAssessment(
            agent_name="consensus",
            risk_level=self.determine_risk_level(tolerant_score),
            risk_score=tolerant_score,
            factors=[],
            mitigation=[],
            confidence=adjusted_confidence,
            timestamp=datetime.now()
        )
        
    def _calculate_disagreement(
        self,
        individual_assessments: List[Tuple[RiskAgent, RiskAssessment]]
    ) -> float:
        """Calculate disagreement score among agents"""
        if len(individual_assessments) < 2:
            return 0.0
            
        scores = [assessment.risk_score for _, assessment in individual_assessments]
        
        # Calculate standard deviation as disagreement metric
        mean_score = sum(scores) / len(scores)
        variance = sum((score - mean_score) ** 2 for score in scores) / len(scores)
        disagreement = variance ** 0.5
        
        return min(disagreement, 1.0)
        
    def _adjust_confidence(self, base_confidence: float, disagreement: float) -> float:
        """Adjust confidence based on disagreement level"""
        if disagreement > self.disagreement_threshold:
            # Reduce confidence when there's high disagreement
            confidence_penalty = (disagreement - self.disagreement_threshold) * 0.5
            adjusted_confidence = base_confidence * (1.0 - confidence_penalty)
        else:
            # Slightly increase confidence when there's consensus
            confidence_bonus = (self.disagreement_threshold - disagreement) * 0.1
            adjusted_confidence = min(1.0, base_confidence * (1.0 + confidence_bonus))
            
        return max(self.min_confidence, adjusted_confidence)
        
    def _aggregate_factors(
        self,
        individual_assessments: List[Tuple[RiskAgent, RiskAssessment]]
    ) -> List[str]:
        """Aggregate risk factors from all agents"""
        all_factors = []
        
        for agent, assessment in individual_assessments:
            for factor in assessment.factors:
                all_factors.append(f"{agent.name}: {factor}")
                
        # Remove duplicates and limit to top factors
        unique_factors = list(set(all_factors))
        return unique_factors[:10]  # Limit to top 10
        
    def _aggregate_mitigation(
        self,
        individual_assessments: List[Tuple[RiskAgent, RiskAssessment]]
    ) -> List[str]:
        """Aggregate mitigation strategies from all agents"""
        all_mitigation = []
        
        for _, assessment in individual_assessments:
            all_mitigation.extend(assessment.mitigation)
            
        # Remove duplicates
        unique_mitigation = list(set(all_mitigation))
        return unique_mitigation[:8]  # Limit to top 8
        
    def update_agent_performance(self, agent_name: str, performance_score: float):
        """Update historical performance of an agent"""
        self.agent_performance[agent_name] = performance_score
        
    def add_risk_agent(self, agent: RiskAgent):
        """Add a new risk agent to the consensus"""
        self.risk_agents.append(agent)
        
    def remove_risk_agent(self, agent_name: str):
        """Remove a risk agent from the consensus"""
        self.risk_agents = [agent for agent in self.risk_agents if agent.name != agent_name]
