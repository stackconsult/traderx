"""
Base risk agent with common risk management functionality.
"""

from abc import ABC, abstractmethod
from typing import Dict, Any, List, Optional
from dataclasses import dataclass
from enum import Enum
import asyncio
import logging
from datetime import datetime, timedelta

logger = logging.getLogger(__name__)


class RiskLevel(Enum):
    """Risk classification levels"""
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"


@dataclass
class RiskAssessment:
    """Standardized risk assessment result"""
    agent_name: str
    risk_level: RiskLevel
    risk_score: float  # 0.0 to 1.0
    factors: List[str]
    mitigation: List[str]
    confidence: float
    timestamp: datetime


class BaseRiskAgent(ABC):
    """
    Base class for risk management agents.
    
    Provides common risk assessment functionality and interfaces.
    """
    
    def __init__(self, name: str):
        self.name = name
        
    @abstractmethod
    async def assess_risk(self, input_data: Dict[str, Any]) -> RiskAssessment:
        """Assess risk for given input data"""
        pass
        
    def calculate_risk_score(self, factors: Dict[str, float]) -> float:
        """
        Calculate overall risk score from individual factors.
        
        Args:
            factors: Dictionary of risk factors with weights (0.0 to 1.0)
            
        Returns:
            Overall risk score (0.0 to 1.0)
        """
        if not factors:
            return 0.0
            
        # Weighted average of factors
        total_weight = sum(factors.values())
        if total_weight == 0:
            return 0.0
            
        weighted_sum = sum(factor * weight for factor, weight in factors.items())
        return weighted_sum / total_weight
        
    def determine_risk_level(self, risk_score: float) -> RiskLevel:
        """Convert risk score to risk level"""
        if risk_score >= 0.8:
            return RiskLevel.CRITICAL
        elif risk_score >= 0.6:
            return RiskLevel.HIGH
        elif risk_score >= 0.3:
            return RiskLevel.MEDIUM
        else:
            return RiskLevel.LOW
