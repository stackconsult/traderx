"""
Integration layer for Sentinel-Nexus architecture.

Provides bridges and interfaces for connecting with TraderX execution system.
"""

from .traderx_bridge import TraderXBridge, TraderXSignal, ExecutionAgentInterface

__all__ = [
    "TraderXBridge",
    "TraderXSignal", 
    "ExecutionAgentInterface",
]
