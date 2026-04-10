"""
Multi-Agent Risk Consensus System
Replaces traditional rule-based risk management with collaborative AI decision-making.
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


class RiskLevel(Enum):
    """Risk levels determined by consensus."""
    MINIMAL = "minimal"
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"


class ConsensusDecision(Enum):
    """Consensus decision outcomes."""
    APPROVE = "approve"
    REJECT = "reject"
    MODIFY = "modify"
    ESCALATE = "escalate"


@dataclass
class RiskAssessment:
    """Individual agent's risk assessment."""
    agent_id: str
    agent_type: str  # RISK_ANALYST, MARKET_MAKER, COMPLIANCE, PORTFOLIO_MANAGER
    risk_level: RiskLevel
    confidence: float
    reasoning: str
    factors: List[str]
    recommendations: List[str]
    timestamp: datetime


@dataclass
class ConsensusResult:
    """Result of multi-agent risk consensus."""
    request_id: str
    decision: ConsensusDecision
    consensus_strength: float  # 0-1, how strong the consensus is
    risk_level: RiskLevel
    majority_assessment: str
    dissenting_opinions: List[RiskAssessment]
    final_recommendations: List[str]
    required_actions: List[str]
    timestamp: datetime


class RiskConsensusAgent:
    """
    Multi-Agent Risk Consensus System that uses multiple AI agents
    to collaboratively assess and make risk decisions.
    """
    
    def __init__(self, anthropic_api_key: str):
        self.client = anthropic.AsyncAnthropic(api_key=anthropic_api_key)
        self.active_consensus: Dict[str, ConsensusResult] = {}
        self.consensus_history: List[ConsensusResult] = []
        
        # Define agent personas for risk assessment
        self.agent_personas = {
            "RISK_ANALYST": {
                "focus": "Quantitative risk metrics, VaR, stress testing",
                "perspective": "Conservative, data-driven"
            },
            "MARKET_MAKER": {
                "focus": "Market liquidity, execution risk, timing",
                "perspective": "Market-aware, opportunistic"
            },
            "COMPLIANCE": {
                "focus": "Regulatory compliance, legal risk",
                "perspective": "Risk-averse, rule-bound"
            },
            "PORTFOLIO_MANAGER": {
                "focus": "Portfolio impact, correlation, diversification",
                "perspective": "Strategic, balanced"
            }
        }
        
        # Consensus prompts for each agent type
        self.risk_assessment_prompt = """
        You are a {agent_type} AI agent assessing trading risk.
        
        Your focus: {focus}
        Your perspective: {perspective}
        
        Trading Request: {request}
        Market Context: {market}
        Portfolio State: {portfolio}
        
        Provide your assessment:
        1. Risk level (MINIMAL/LOW/MEDIUM/HIGH/CRITICAL)
        2. Confidence (0-1)
        3. Key risk factors identified
        4. Reasoning from your perspective
        5. Specific recommendations
        
        Format as JSON.
        """
        
        self.consensus_prompt = """
        You are the Consensus Coordinator analyzing multiple risk assessments:
        
        Assessments: {assessments}
        Original Request: {request}
        
        Determine:
        1. Final decision (APPROVE/REJECT/MODIFY/ESCALATE)
        2. Consensus strength (0-1)
        3. Overall risk level
        4. Majority reasoning
        5. Required actions or modifications
        6. Final recommendations
        
        Consider:
- Agreement/disagreement between agents
- Most critical risk factors
- Balance of perspectives
- Risk vs reward assessment
        
        Format as JSON.
        """
    
    async def assess_risk_consensus(self,
                                   request_id: str,
                                   trading_request: Dict[str, Any],
                                   market_context: Dict[str, Any],
                                   portfolio_state: Dict[str, Any]) -> ConsensusResult:
        """
        Run multi-agent risk consensus assessment.
        """
        logger.info(
            "Starting risk consensus assessment",
            request_id=request_id,
            request_type=trading_request.get("type")
        )
        
        # Collect assessments from all agent types
        assessments = []
        
        for agent_type, persona in self.agent_personas.items():
            assessment = await self._get_agent_assessment(
                agent_type,
                persona,
                trading_request,
                market_context,
                portfolio_state
            )
            assessments.append(assessment)
            
            # Add small delay between agents to simulate real collaboration
            await asyncio.sleep(0.1)
        
        # Calculate consensus
        consensus = await self._calculate_consensus(request_id, assessments, trading_request)
        
        # Store result
        self.active_consensus[request_id] = consensus
        self.consensus_history.append(consensus)
        
        # Log consensus result
        logger.info(
            "Risk consensus completed",
            request_id=request_id,
            decision=consensus.decision.value,
            consensus_strength=consensus.consensus_strength,
            risk_level=consensus.risk_level.value
        )
        
        return consensus
    
    async def _get_agent_assessment(self,
                                   agent_type: str,
                                   persona: Dict[str, str],
                                   trading_request: Dict[str, Any],
                                   market_context: Dict[str, Any],
                                   portfolio_state: Dict[str, Any]) -> RiskAssessment:
        """
        Get risk assessment from a specific AI agent.
        """
        prompt = self.risk_assessment_prompt.format(
            agent_type=agent_type,
            focus=persona["focus"],
            perspective=persona["perspective"],
            request=trading_request,
            market=market_context,
            portfolio=portfolio_state
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
            
            ai_response = json.loads(response.content[0].text)
            
            return RiskAssessment(
                agent_id=f"{agent_type.lower()}_{int(datetime.utcnow().timestamp())}",
                agent_type=agent_type,
                risk_level=RiskLevel(ai_response.get("risk_level", "MEDIUM")),
                confidence=float(ai_response.get("confidence", 0.5)),
                reasoning=ai_response.get("reasoning", ""),
                factors=ai_response.get("factors", []),
                recommendations=ai_response.get("recommendations", []),
                timestamp=datetime.utcnow()
            )
            
        except Exception as e:
            logger.error(
                "Agent assessment failed",
                agent_type=agent_type,
                error=str(e)
            )
            
            # Fallback assessment
            return RiskAssessment(
                agent_id=f"{agent_type.lower()}_fallback",
                agent_type=agent_type,
                risk_level=RiskLevel.MEDIUM,
                confidence=0.3,
                reason="AI assessment failed",
                factors=["Assessment unavailable"],
                recommendations=["Manual review required"],
                timestamp=datetime.utcnow()
            )
    
    async def _calculate_consensus(self,
                                  request_id: str,
                                  assessments: List[RiskAssessment],
                                  trading_request: Dict[str, Any]) -> ConsensusResult:
        """
        Calculate consensus from multiple agent assessments.
        """
        # Prepare consensus prompt
        assessments_data = [asdict(a) for a in assessments]
        
        prompt = self.consensus_prompt.format(
            assessments=assessments_data,
            request=trading_request
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
            
            consensus_data = json.loads(response.content[0].text)
            
            # Identify dissenting opinions
            majority_risk = RiskLevel(consensus_data.get("overall_risk_level", "MEDIUM"))
            dissenting = [a for a in assessments if a.risk_level != majority_risk]
            
            return ConsensusResult(
                request_id=request_id,
                decision=ConsensusDecision(consensus_data.get("final_decision", "APPROVE")),
                consensus_strength=float(consensus_data.get("consensus_strength", 0.5)),
                risk_level=majority_risk,
                majority_assessment=consensus_data.get("majority_reasoning", ""),
                dissenting_opinions=dissenting,
                final_recommendations=consensus_data.get("final_recommendations", []),
                required_actions=consensus_data.get("required_actions", []),
                timestamp=datetime.utcnow()
            )
            
        except Exception as e:
            logger.error(
                "Consensus calculation failed",
                request_id=request_id,
                error=str(e)
            )
            
            # Fallback consensus
            risk_levels = [a.risk_level for a in assessments]
            most_common = max(set(risk_levels), key=risk_levels.count)
            
            return ConsensusResult(
                request_id=request_id,
                decision=ConsensusDecision.ESCALATE,
                consensus_strength=0.0,
                risk_level=most_common,
                majority_assessment="Consensus failed, manual review required",
                dissenting_opinions=assessments,
                final_recommendations=["Manual review required"],
                required_actions=["Escalate to human"],
                timestamp=datetime.utcnow()
            )
    
    async def continuous_risk_monitoring(self,
                                        agent_id: str,
                                        positions: List[Dict[str, Any]],
                                        market_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Continuous risk monitoring with consensus alerts.
        """
        monitoring_request = {
            "type": "CONTINUOUS_MONITORING",
            "agent_id": agent_id,
            "positions": positions,
            "trigger": "AUTOMATIC"
        }
        
        portfolio_state = {
            "agent_id": agent_id,
            "positions": positions,
            "total_exposure": sum(p.get("exposure", 0) for p in positions)
        }
        
        # Run consensus assessment
        consensus = await self.assess_risk_consensus(
            f"monitor_{agent_id}_{int(datetime.utcnow().timestamp())}",
            monitoring_request,
            market_data,
            portfolio_state
        )
        
        # Determine if alert is needed
        alert_needed = (
            consensus.risk_level in [RiskLevel.HIGH, RiskLevel.CRITICAL] or
            consensus.decision in [ConsensusDecision.REJECT, ConsensusDecision.ESCALATE]
        )
        
        return {
            "consensus": asdict(consensus),
            "alert_needed": alert_needed,
            "alert_level": consensus.risk_level.value,
            "recommended_actions": consensus.required_actions
        }
    
    async def emergency_risk_assessment(self,
                                       incident_type: str,
                                       incident_data: Dict[str, Any]) -> ConsensusResult:
        """
        Emergency risk assessment for critical incidents.
        """
        emergency_request = {
            "type": "EMERGENCY",
            "incident_type": incident_type,
            "incident_data": incident_data,
            "priority": "CRITICAL"
        }
        
        # Use emergency context
        emergency_context = {
            "market_status": "ACTIVE",
            "volatility": "ELEVATED",
            "liquidity": "UNKNOWN"
        }
        
        portfolio_state = {
            "emergency_mode": True,
            "all_positions_at_risk": True
        }
        
        # Run accelerated consensus
        consensus = await self.assess_risk_consensus(
            f"emergency_{incident_type}_{int(datetime.utcnow().timestamp())}",
            emergency_request,
            emergency_context,
            portfolio_state
        )
        
        # Log emergency assessment
        logger.critical(
            "Emergency risk assessment completed",
            incident_type=incident_type,
            decision=consensus.decision.value,
            risk_level=consensus.risk_level.value
        )
        
        return consensus
    
    async def get_consensus_history(self,
                                   agent_id: Optional[str] = None,
                                   hours: int = 24) -> List[ConsensusResult]:
        """
        Get consensus history for analysis.
        """
        cutoff_time = datetime.utcnow() - timedelta(hours=hours)
        
        history = [
            c for c in self.consensus_history
            if c.timestamp > cutoff_time
        ]
        
        # Filter by agent if specified
        if agent_id:
            # This would need request_id to agent mapping
            pass
        
        return history
    
    async def analyze_consensus_patterns(self) -> Dict[str, Any]:
        """
        Analyze patterns in consensus decisions.
        """
        if not self.consensus_history:
            return {"error": "No consensus history available"}
        
        # Calculate statistics
        total_consensus = len(self.consensus_history)
        decisions = [c.decision for c in self.consensus_history]
        risk_levels = [c.risk_level for c in self.consensus_history]
        consensus_strengths = [c.consensus_strength for c in self.consensus_history]
        
        decision_counts = {}
        for decision in decisions:
            decision_counts[decision.value] = decision_counts.get(decision.value, 0) + 1
        
        risk_counts = {}
        for risk in risk_levels:
            risk_counts[risk.value] = risk_counts.get(risk.value, 0) + 1
        
        return {
            "total_assessments": total_consensus,
            "decision_distribution": decision_counts,
            "risk_level_distribution": risk_counts,
            "average_consensus_strength": np.mean(consensus_strengths),
            "strong_consensus_rate": len([s for s in consensus_strengths if s > 0.8]) / total_consensus,
            "escalation_rate": decision_counts.get("ESCALATE", 0) / total_consensus
        }
    
    def get_active_consensus(self, request_id: str) -> Optional[ConsensusResult]:
        """Get active consensus by request ID."""
        return self.active_consensus.get(request_id)
    
    def update_consensus_decision(self,
                                 request_id: str,
                                 new_decision: ConsensusDecision,
                                 reason: str) -> bool:
        """
        Update consensus decision (e.g., after human review).
        """
        if request_id in self.active_consensus:
            consensus = self.active_consensus[request_id]
            consensus.decision = new_decision
            consensus.timestamp = datetime.utcnow()
            consensus.required_actions.append(f"Decision updated: {reason}")
            
            logger.info(
                "Consensus decision updated",
                request_id=request_id,
                new_decision=new_decision.value,
                reason=reason
            )
            
            return True
        
        return False
