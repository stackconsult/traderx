"""
AI Kill Switch with Autonomous Triggers
Emergency position liquidation and system protection using AI decision-making.
"""

import asyncio
import json
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Callable
from dataclasses import dataclass, asdict
from enum import Enum
import anthropic
import structlog

logger = structlog.get_logger(__name__)


class KillSwitchTrigger(Enum):
    """Types of kill switch triggers."""
    LOSS_THRESHOLD = "loss_threshold"
    DRAWDOWN_LIMIT = "drawdown_limit"
    POSITION_LIMIT = "position_limit"
    CORRELATION_SPIKE = "correlation_spike"
    VOLATILITY_CRISIS = "volatility_crisis"
    LIQUIDITY_CRUNCH = "liquidity_crunch"
    SYSTEM_ANOMALY = "system_anomaly"
    MANUAL_OVERRIDE = "manual_override"


class KillSwitchScope(Enum):
    """Scope of kill switch activation."""
    GLOBAL = "global"  # All trading
    SYMBOL = "symbol"  # Specific symbol
    ACCOUNT = "account"  # Specific account
    STRATEGY = "strategy"  # Specific strategy


class KillSwitchAction(Enum):
    """Actions taken by kill switch."""
    STOP_NEW_ORDERS = "stop_new_orders"
    CANCEL_PENDING = "cancel_pending"
    LIQUIDATE_POSITIONS = "liquidate_positions"
    REDUCE_EXPOSURE = "reduce_exposure"
    SWITCH_TO_CASH = "switch_to_cash"
    ESCALATE_TO_HUMAN = "escalate_to_human"


@dataclass
class KillSwitchEvent:
    """Kill switch activation event."""
    event_id: str
    trigger: KillSwitchTrigger
    scope: KillSwitchScope
    scope_value: Optional[str]  # symbol/account/strategy if applicable
    action: KillSwitchAction
    confidence: float
    reasoning: str
    metrics: Dict[str, float]
    triggered_at: datetime
    resolved_at: Optional[datetime]
    resolution: Optional[str]


class KillSwitchAgent:
    """
    AI-powered kill switch that monitors trading systems and
    autonomously triggers emergency actions when risks exceed thresholds.
    """
    
    def __init__(self, anthropic_api_key: str):
        self.client = anthropic.AsyncAnthropic(api_key=anthropic_api_key)
        self.active_kill_switches: Dict[str, KillSwitchEvent] = {}
        self.kill_switch_history: List[KillSwitchEvent] = []
        
        # Configuration
        self.thresholds = {
            "max_daily_loss": 100000.0,
            "max_drawdown": 0.20,
            "max_position_size": 1000000.0,
            "max_correlation": 0.95,
            "volatility_threshold": 0.05,
            "liquidity_ratio_min": 0.1
        }
        
        # Callbacks for actions
        self.order_manager: Optional[Callable] = None
        self.position_manager: Optional[Callable] = None
        self.alert_system: Optional[Callable] = None
        
        # AI prompts
        self.trigger_analysis_prompt = """
        You are an AI Kill Switch Analyst. Analyze this situation:
        
        Current Metrics: {metrics}
        Thresholds: {thresholds}
        Recent Activity: {activity}
        
        Determine:
        1. Is emergency action needed? (YES/NO)
        2. Trigger type if yes
        3. Scope of action (GLOBAL/SYMBOL/ACCOUNT/STRATEGY)
        4. Recommended action
        5. Confidence in decision (0-1)
        6. Reasoning
        7. Critical metrics driving decision
        
        Consider market conditions, system health, and risk factors.
        
        Format as JSON.
        """
        
        self.action_execution_prompt = """
        Execute kill switch action:
        
        Event: {event}
        Available Actions: {available_actions}
        System State: {system_state}
        
        Plan:
        1. Execution steps
        2. Order of operations
        3. Verification checks
        4. Rollback conditions if needed
        5. Communication requirements
        
        Format as JSON.
        """
    
    def register_callbacks(self,
                          order_manager: Callable,
                          position_manager: Callable,
                          alert_system: Callable):
        """Register system callbacks for kill switch actions."""
        self.order_manager = order_manager
        self.position_manager = position_manager
        self.alert_system = alert_system
    
    async def monitor_and_evaluate(self,
                                  current_metrics: Dict[str, float],
                                  recent_activity: List[Dict]) -> Optional[KillSwitchEvent]:
        """
        Continuously monitor and evaluate if kill switch should be triggered.
        """
        # Check all threshold conditions
        violations = []
        
        # Loss threshold check
        daily_pnl = current_metrics.get("daily_pnl", 0)
        if daily_pnl < -self.thresholds["max_daily_loss"]:
            violations.append({
                "type": KillSwitchTrigger.LOSS_THRESHOLD,
                "value": daily_pnl,
                "threshold": self.thresholds["max_daily_loss"],
                "severity": "HIGH"
            })
        
        # Drawdown check
        drawdown = current_metrics.get("max_drawdown", 0)
        if drawdown > self.thresholds["max_drawdown"]:
            violations.append({
                "type": KillSwitchTrigger.DRAWDOWN_LIMIT,
                "value": drawdown,
                "threshold": self.thresholds["max_drawdown"],
                "severity": "CRITICAL"
            })
        
        # Position size check
        max_position = current_metrics.get("max_position_size", 0)
        if max_position > self.thresholds["max_position_size"]:
            violations.append({
                "type": KillSwitchTrigger.POSITION_LIMIT,
                "value": max_position,
                "threshold": self.thresholds["max_position_size"],
                "severity": "MEDIUM"
            })
        
        # Volatility check
        volatility = current_metrics.get("volatility", 0)
        if volatility > self.thresholds["volatility_threshold"]:
            violations.append({
                "type": KillSwitchTrigger.VOLATILITY_CRISIS,
                "value": volatility,
                "threshold": self.thresholds["volatility_threshold"],
                "severity": "HIGH"
            })
        
        # If violations found, use AI to decide on action
        if violations:
            return await self._evaluate_trigger_decision(
                violations,
                current_metrics,
                recent_activity
            )
        
        return None
    
    async def _evaluate_trigger_decision(self,
                                        violations: List[Dict],
                                        current_metrics: Dict[str, float],
                                        recent_activity: List[Dict]) -> Optional[KillSwitchEvent]:
        """
        Use AI to evaluate if kill switch should be triggered.
        """
        prompt = self.trigger_analysis_prompt.format(
            metrics=current_metrics,
            thresholds=self.thresholds,
            violations=violations,
            activity=recent_activity[-10:]  # Last 10 activities
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
            
            decision = json.loads(response.content[0].text)
            
            if decision.get("emergency_action_needed") == "YES":
                event_id = f"kill_{int(datetime.utcnow().timestamp())}"
                
                # Determine scope from AI decision
                scope = KillSwitchScope(decision.get("scope", "GLOBAL"))
                scope_value = decision.get("scope_value")
                
                # Create kill switch event
                event = KillSwitchEvent(
                    event_id=event_id,
                    trigger=KillSwitchTrigger(decision.get("trigger_type", "LOSS_THRESHOLD")),
                    scope=scope,
                    scope_value=scope_value,
                    action=KillSwitchAction(decision.get("recommended_action", "STOP_NEW_ORDERS")),
                    confidence=float(decision.get("confidence", 0.5)),
                    reasoning=decision.get("reasoning", ""),
                    metrics=current_metrics,
                    triggered_at=datetime.utcnow(),
                    resolved_at=None,
                    resolution=None
                )
                
                # Execute the kill switch
                await self._execute_kill_switch(event)
                
                return event
            
        except Exception as e:
            logger.error(
                "Kill switch evaluation failed",
                error=str(e)
            )
            
            # Fail safe - trigger emergency stop
            event = KillSwitchEvent(
                event_id=f"kill_failsafe_{int(datetime.utcnow().timestamp())}",
                trigger=KillSwitchTrigger.SYSTEM_ANOMALY,
                scope=KillSwitchScope.GLOBAL,
                scope_value=None,
                action=KillSwitchAction.STOP_NEW_ORDERS,
                confidence=1.0,
                reasoning=f"AI evaluation failed: {str(e)}",
                metrics=current_metrics,
                triggered_at=datetime.utcnow(),
                resolved_at=None,
                resolution=None
            )
            
            await self._execute_kill_switch(event)
            return event
        
        return None
    
    async def _execute_kill_switch(self, event: KillSwitchEvent):
        """
        Execute the kill switch action.
        """
        logger.critical(
            "KILL SWITCH ACTIVATED",
            event_id=event.event_id,
            trigger=event.trigger.value,
            action=event.action.value,
            scope=event.scope.value
        )
        
        # Store active event
        self.active_kill_switches[event.event_id] = event
        self.kill_switch_history.append(event)
        
        # Send alert
        if self.alert_system:
            await self.alert_system({
                "type": "KILL_SWITCH_ACTIVATED",
                "event_id": event.event_id,
                "trigger": event.trigger.value,
                "action": event.action.value,
                "reasoning": event.reasoning,
                "confidence": event.confidence
            })
        
        # Execute action based on type
        if event.action == KillSwitchAction.STOP_NEW_ORDERS:
            await self._stop_new_orders(event)
        elif event.action == KillSwitchAction.CANCEL_PENDING:
            await self._cancel_pending_orders(event)
        elif event.action == KillSwitchAction.LIQUIDATE_POSITIONS:
            await self._liquidate_positions(event)
        elif event.action == KillSwitchAction.REDUCE_EXPOSURE:
            await self._reduce_exposure(event)
        elif event.action == KillSwitchAction.SWITCH_TO_CASH:
            await self._switch_to_cash(event)
        elif event.action == KillSwitchAction.ESCALATE_TO_HUMAN:
            await self._escalate_to_human(event)
    
    async def _stop_new_orders(self, event: KillSwitchEvent):
        """Stop all new orders."""
        if self.order_manager:
            if event.scope == KillSwitchScope.GLOBAL:
                await self.order_manager({"action": "STOP_ALL_ORDERS"})
            elif event.scope == KillSwitchScope.SYMBOL:
                await self.order_manager({
                    "action": "STOP_ORDERS",
                    "symbol": event.scope_value
                })
            elif event.scope == KillSwitchScope.ACCOUNT:
                await self.order_manager({
                    "action": "STOP_ORDERS",
                    "account": event.scope_value
                })
        
        logger.info("New orders stopped", event_id=event.event_id)
    
    async def _cancel_pending_orders(self, event: KillSwitchEvent):
        """Cancel all pending orders."""
        if self.order_manager:
            if event.scope == KillSwitchScope.GLOBAL:
                await self.order_manager({"action": "CANCEL_ALL_PENDING"})
            elif event.scope == KillSwitchScope.SYMBOL:
                await self.order_manager({
                    "action": "CANCEL_PENDING",
                    "symbol": event.scope_value
                })
        
        logger.info("Pending orders canceled", event_id=event.event_id)
    
    async def _liquidate_positions(self, event: KillSwitchEvent):
        """Emergency position liquidation."""
        if self.position_manager:
            if event.scope == KillSwitchScope.GLOBAL:
                await self.position_manager({
                    "action": "LIQUIDATE_ALL",
                    "reason": "KILL_SWITCH",
                    "event_id": event.event_id
                })
            elif event.scope == KillSwitchScope.SYMBOL:
                await self.position_manager({
                    "action": "LIQUIDATE_SYMBOL",
                    "symbol": event.scope_value,
                    "reason": "KILL_SWITCH",
                    "event_id": event.event_id
                })
        
        logger.critical("Positions liquidated", event_id=event.event_id)
    
    async def _reduce_exposure(self, event: KillSwitchEvent):
        """Reduce exposure by 50%."""
        if self.position_manager:
            await self.position_manager({
                "action": "REDUCE_EXPOSURE",
                "percentage": 0.5,
                "scope": event.scope.value,
                "scope_value": event.scope_value,
                "reason": "KILL_SWITCH",
                "event_id": event.event_id
            })
        
        logger.warning("Exposure reduced", event_id=event.event_id)
    
    async def _switch_to_cash(self, event: KillSwitchEvent):
        """Switch all positions to cash."""
        await self._liquidate_positions(event)
        logger.critical("Switched to cash", event_id=event.event_id)
    
    async def _escalate_to_human(self, event: KillSwitchEvent):
        """Escalate to human intervention."""
        if self.alert_system:
            await self.alert_system({
                "type": "ESCALATION_REQUIRED",
                "event_id": event.event_id,
                "trigger": event.trigger.value,
                "reasoning": event.reasoning,
                "urgency": "CRITICAL"
            })
        
        logger.critical("Escalated to human", event_id=event.event_id)
    
    async def manual_trigger(self,
                           trigger: KillSwitchTrigger,
                           scope: KillSwitchScope,
                           scope_value: Optional[str] = None,
                           action: Optional[KillSwitchAction] = None,
                           reason: str = "") -> KillSwitchEvent:
        """
        Manually trigger kill switch.
        """
        event_id = f"manual_{int(datetime.utcnow().timestamp())}"
        
        event = KillSwitchEvent(
            event_id=event_id,
            trigger=trigger,
            scope=scope,
            scope_value=scope_value,
            action=action or KillSwitchAction.STOP_NEW_ORDERS,
            confidence=1.0,
            reasoning=f"Manual trigger: {reason}",
            metrics={},
            triggered_at=datetime.utcnow(),
            resolved_at=None,
            resolution=None
        )
        
        await self._execute_kill_switch(event)
        return event
    
    async def resolve_kill_switch(self,
                                 event_id: str,
                                 resolution: str,
                                 resolved_by: str) -> bool:
        """
        Resolve an active kill switch.
        """
        if event_id not in self.active_kill_switches:
            return False
        
        event = self.active_kill_switches[event_id]
        event.resolved_at = datetime.utcnow()
        event.resolution = f"{resolution} (by {resolved_by})"
        
        # Move to history
        self.kill_switch_history.append(event)
        del self.active_kill_switches[event_id]
        
        logger.info(
            "Kill switch resolved",
            event_id=event_id,
            resolution=resolution
        )
        
        # Send notification
        if self.alert_system:
            await self.alert_system({
                "type": "KILL_SWITCH_RESOLVED",
                "event_id": event_id,
                "resolution": resolution
            })
        
        return True
    
    async def get_kill_switch_status(self) -> Dict[str, Any]:
        """Get current kill switch status."""
        active_count = len(self.active_kill_switches)
        
        # Get recent history
        recent_events = [
            e for e in self.kill_switch_history
            if e.triggered_at > datetime.utcnow() - timedelta(hours=24)
        ]
        
        return {
            "active_kill_switches": active_count,
            "active_events": [asdict(e) for e in self.active_kill_switches.values()],
            "recent_events": [asdict(e) for e in recent_events],
            "total_events_24h": len(recent_events),
            "thresholds": self.thresholds,
            "system_status": "CRITICAL" if active_count > 0 else "NORMAL"
        }
    
    def update_thresholds(self, new_thresholds: Dict[str, float]):
        """Update kill switch thresholds."""
        self.thresholds.update(new_thresholds)
        logger.info("Kill switch thresholds updated", thresholds=new_thresholds)
    
    def get_active_event(self, event_id: str) -> Optional[KillSwitchEvent]:
        """Get specific active kill switch event."""
        return self.active_kill_switches.get(event_id)
