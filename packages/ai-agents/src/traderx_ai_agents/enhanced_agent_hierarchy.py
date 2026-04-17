"""
Enhanced Agent Hierarchy for TraderX Sentinel-Nexus Architecture

Integrates TradingAgents analysis hierarchy with TraderX execution agents.
Analysis layer generates signals → Execution layer acts on signals.
"""

from typing import Dict, List, Optional, Any
from dataclasses import dataclass
from enum import Enum
import asyncio
import logging

logger = logging.getLogger(__name__)


class AgentType(Enum):
    """Agent classification in the hierarchy"""
    ANALYSIS = "analysis"  # TradingAgents-derived
    RISK = "risk"  # Both systems
    EXECUTION = "execution"  # TraderX-derived
    COORDINATION = "coordination"  # New orchestration layer


@dataclass
class AgentSignal:
    """Standardized signal format for inter-agent communication"""
    source_agent: str
    target_agent: str
    signal_type: str
    data: Dict[str, Any]
    timestamp: float
    confidence: float
    priority: int  # 1=highest, 5=lowest


class EnhancedAgentOrchestrator:
    """
    Orchestrates the merged agent hierarchy.
    
    Architecture:
    ┌─────────────────────────────────────────────────────────┐
    │                ANALYSIS LAYER                           │
    │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐ │
    │  │ Fundamentals│ │   Market    │ │    News/Sentiment   │ │
    │  │   Analyst   │ │   Analyst   │ │      Analysts       │ │
    │  └─────────────┘ └─────────────┘ └─────────────────────┘ │
    │           │               │                 │           │
    │           └───────────────┼─────────────────┘           │
    │                           │                             │
    │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐ │
    │  │   Bull      │ │    Bear     │ │   Research Manager  │ │
    │  │ Researcher  │ │  Researcher │ │                     │ │
    │  └─────────────┘ └─────────────┘ └─────────────────────┘ │
    └─────────────────────────────────────────────────────────┘
                                │
    ┌─────────────────────────────────────────────────────────┐
    │                 RISK LAYER                              │
    │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐ │
    │  │Aggressive   │ │Conservative │ │   Risk Consensus    │ │
    │  │  Debator    │ │  Debator    │ │      Agent          │ │
    │  └─────────────┘ └─────────────┘ └─────────────────────┘ │
    │                           │                             │
    │               ┌─────────────┴─────────────┐             │
    │               │   Portfolio Manager      │             │
    │               └─────────────────────────┘             │
    └─────────────────────────────────────────────────────────┘
                                │
    ┌─────────────────────────────────────────────────────────┐
    │              EXECUTION LAYER                            │
    │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐ │
    │  │Smart Order  │ │   Position  │ │   Reconciliation    │ │
    │  │  Routing    │ │   Tracker   │ │      Agent          │ │
    │  └─────────────┘ └─────────────┘ └─────────────────────┘ │
    │           │               │                 │           │
    │           └───────────────┼─────────────────┘           │
    │                           │                             │
    │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐ │
    │  │  Kill Switch│ │Market Data  │ │   Order Management  │ │
    │  │   Agent     │ │ Interpreter │ │      Agent          │ │
    │  └─────────────┘ └─────────────┘ └─────────────────────┘ │
    └─────────────────────────────────────────────────────────┘
    """
    
    def __init__(self):
        self.analysis_agents: Dict[str, Any] = {}
        self.risk_agents: Dict[str, Any] = {}
        self.execution_agents: Dict[str, Any] = {}
        self.signal_queue: asyncio.Queue = asyncio.Queue()
        self.running = False
        
    async def initialize(self):
        """Initialize all agents and establish communication channels"""
        # Import existing TraderX agents
        from .kill_switch_agent import KillSwitchAgent
        from .market_data_interpreter import MarketDataInterpreter
        from .position_tracker_agent import PositionTrackerAgent
        from .reconciliation_agent import ReconciliationAgent
        from .risk_consensus_agent import RiskConsensusAgent
        from .smart_order_routing_agent import SmartOrderRoutingAgent
        from .order_management_agent import OrderManagementAgent
        
        # Register execution agents (existing)
        self.execution_agents["kill_switch"] = KillSwitchAgent()
        self.execution_agents["market_data"] = MarketDataInterpreter()
        self.execution_agents["position_tracker"] = PositionTrackerAgent()
        self.execution_agents["reconciliation"] = ReconciliationAgent()
        self.execution_agents["smart_routing"] = SmartOrderRoutingAgent()
        self.execution_agents["order_mgmt"] = OrderManagementAgent()
        
        # Register risk agents (existing + enhanced)
        self.risk_agents["risk_consensus"] = RiskConsensusAgent()
        
        # TODO: Add TradingAgents analysis agents
        # self.analysis_agents["fundamentals"] = FundamentalsAnalyst()
        # self.analysis_agents["market"] = MarketAnalyst()
        # self.analysis_agents["news"] = NewsAnalyst()
        # self.analysis_agents["social"] = SocialMediaAnalyst()
        
        logger.info("Enhanced Agent Orchestrator initialized with hierarchy")
        
    async def start(self):
        """Start the orchestration loop"""
        self.running = True
        
        # Start signal processing task
        asyncio.create_task(self._process_signals())
        
        # Start individual agents
        for agent_type, agents in [
            (AgentType.ANALYSIS, self.analysis_agents),
            (AgentType.RISK, self.risk_agents),
            (AgentType.EXECUTION, self.execution_agents)
        ]:
            for name, agent in agents.items():
                if hasattr(agent, 'start'):
                    await agent.start()
                    logger.info(f"Started {agent_type.value} agent: {name}")
                    
    async def stop(self):
        """Stop all agents"""
        self.running = False
        
        for agent_type, agents in [
            (AgentType.ANALYSIS, self.analysis_agents),
            (AgentType.RISK, self.risk_agents),
            (AgentType.EXECUTION, self.execution_agents)
        ]:
            for name, agent in agents.items():
                if hasattr(agent, 'stop'):
                    await agent.stop()
                    logger.info(f"Stopped {agent_type.value} agent: {name}")
                    
    async def send_signal(self, signal: AgentSignal):
        """Send a signal between agents"""
        await self.signal_queue.put(signal)
        
    async def _process_signals(self):
        """Process signals in the queue"""
        while self.running:
            try:
                signal = await asyncio.wait_for(self.signal_queue.get(), timeout=1.0)
                await self._route_signal(signal)
            except asyncio.TimeoutError:
                continue
            except Exception as e:
                logger.error(f"Error processing signal: {e}")
                
    async def _route_signal(self, signal: AgentSignal):
        """Route signal to appropriate agent"""
        target = signal.target_agent
        
        # Check execution agents
        if target in self.execution_agents:
            agent = self.execution_agents[target]
            if hasattr(agent, 'handle_signal'):
                await agent.handle_signal(signal)
                
        # Check risk agents
        elif target in self.risk_agents:
            agent = self.risk_agents[target]
            if hasattr(agent, 'handle_signal'):
                await agent.handle_signal(signal)
                
        # Check analysis agents
        elif target in self.analysis_agents:
            agent = self.analysis_agents[target]
            if hasattr(agent, 'handle_signal'):
                await agent.handle_signal(signal)
        else:
            logger.warning(f"Unknown target agent: {target}")


# Signal routing constants
SIGNAL_ANALYSIS_TO_RISK = "analysis_to_risk"
SIGNAL_RISK_TO_EXECUTION = "risk_to_execution"
SIGNAL_EXECUTION_TO_ANALYSIS = "execution_to_analysis"


class SignalRouter:
    """Routes signals between agent layers based on predefined rules"""
    
    @staticmethod
    def create_portfolio_signal(portfolio_decision: Dict[str, Any]) -> AgentSignal:
        """Create a portfolio decision signal for execution layer"""
        return AgentSignal(
            source_agent="portfolio_manager",
            target_agent="smart_routing",
            signal_type="portfolio_decision",
            data=portfolio_decision,
            timestamp=asyncio.get_event_loop().time(),
            confidence=portfolio_decision.get("confidence", 0.5),
            priority=2
        )
        
    @staticmethod
    def create_risk_signal(risk_assessment: Dict[str, Any]) -> AgentSignal:
        """Create a risk assessment signal"""
        return AgentSignal(
            source_agent="risk_consensus",
            target_agent="portfolio_manager",
            signal_type="risk_assessment",
            data=risk_assessment,
            timestamp=asyncio.get_event_loop().time(),
            confidence=risk_assessment.get("confidence", 0.5),
            priority=1  # High priority for risk signals
        )
        
    @staticmethod
    def create_market_signal(market_data: Dict[str, Any]) -> AgentSignal:
        """Create market data signal for analysis layer"""
        return AgentSignal(
            source_agent="market_data",
            target_agent="market_analyst",
            signal_type="market_update",
            data=market_data,
            timestamp=asyncio.get_event_loop().time(),
            confidence=1.0,
            priority=3
        )
