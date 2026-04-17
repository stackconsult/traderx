"""
TraderX AI Agents - Sentinel-Nexus Architecture

Integrates multi-agent analysis hierarchy with execution layer.
"""

__version__ = "0.1.0"

# Import available components
try:
    from .enhanced_agent_hierarchy import EnhancedAgentOrchestrator, AgentSignal, SignalRouter
    _HAS_ORCHESTRATOR = True
except ImportError:
    _HAS_ORCHESTRATOR = False

try:
    from .analysis import (
        FundamentalsAnalyst,
        MarketAnalyst,
        NewsAnalyst,
        SocialMediaAnalyst,
        BullResearcher,
        BearResearcher,
    )
    _HAS_ANALYSIS = True
except ImportError:
    _HAS_ANALYSIS = False

try:
    from .risk import PortfolioManager, RiskConsensusAgent
    _HAS_RISK = True
except ImportError:
    _HAS_RISK = False

# Build __all__ based on available components
__all__ = []

if _HAS_ORCHESTRATOR:
    __all__.extend([
        "EnhancedAgentOrchestrator",
        "AgentSignal",
        "SignalRouter",
    ])

if _HAS_ANALYSIS:
    __all__.extend([
        "FundamentalsAnalyst",
        "MarketAnalyst",
        "NewsAnalyst",
        "SocialMediaAnalyst",
        "BullResearcher",
        "BearResearcher",
    ])

if _HAS_RISK:
    __all__.extend([
        "PortfolioManager",
        "RiskConsensusAgent",
    ])
