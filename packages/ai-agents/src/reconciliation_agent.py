"""
Agent-Based Reconciliation System
Uses AI agents to detect and resolve state mismatches between internal and external systems.
"""

import asyncio
import json
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
import anthropic
import structlog

logger = structlog.get_logger(__name__)


class ReconciliationStatus(Enum):
    """Reconciliation status determined by AI."""
    MATCHED = "matched"
    MISMATCH = "mismatch"
    PARTIAL = "partial"
    PENDING = "pending"
    RESOLVED = "resolved"


class BreakType(Enum):
    """Types of breaks the AI can identify."""
    PRICE_DIFFERENCE = "price_difference"
    QUANTITY_MISMATCH = "quantity_mismatch"
    MISSING_TRADE = "missing_trade"
    TIMING_DIFFERENCE = "timing_difference"
    FEE_DISCREPANCY = "fee_discrepancy"
    CURRENCY_MISMATCH = "currency_mismatch"


@dataclass
class ReconciliationBreak:
    """AI-identified reconciliation break."""
    break_id: str
    break_type: BreakType
    internal_value: Any
    external_value: Any
    difference: float
    confidence: float
    explanation: str
    suggested_action: str
    auto_resolvable: bool
    created_at: datetime
    resolved_at: Optional[datetime]


@dataclass
class ReconciliationResult:
    """Result of AI-powered reconciliation."""
    entity_type: str  # TRADE, POSITION, CASH
    entity_id: str
    status: ReconciliationStatus
    internal_state: Dict[str, Any]
    external_state: Dict[str, Any]
    breaks: List[ReconciliationBreak]
    ai_summary: str
    confidence: float
    timestamp: datetime


class ReconciliationAgent:
    """
    AI-powered reconciliation agent that detects and resolves
    state mismatches between internal and external systems.
    """
    
    def __init__(self, anthropic_api_key: str):
        self.client = anthropic.AsyncAnthropic(api_key=anthropic_api_key)
        self.active_reconciliations: Dict[str, ReconciliationResult] = {}
        self.break_history: List[ReconciliationBreak] = []
        
        # Reconciliation prompts
        self.trade_reconciliation_prompt = """
        You are an AI Trade Reconciliation Specialist. Compare these trade records:
        
        Internal Trade: {internal_trade}
        External Trade (Exchange): {external_trade}
        
        Identify and analyze:
        1. Do the trades match? (MATCHED/MISMATCH/PARTIAL)
        2. Any differences in price, quantity, fees, timing?
        3. Break type if mismatched
        4. Confidence in assessment (0-1)
        5. Explanation of differences
        6. Suggested resolution action
        7. Can this be auto-resolved?
        
        Consider market data timing, rounding differences, fee structures.
        
        Format as JSON.
        """
        
        self.position_reconciliation_prompt = """
        Reconcile these position records:
        
        Internal Position: {internal_position}
        External Position: {external_position}
        
        Analyze:
        1. Position quantity differences
        2. Average price discrepancies
        3. Unrealized P&L differences
        4. Break identification
        5. Resolution recommendations
        
        Format as JSON.
        """
        
        self.auto_resolution_prompt = """
        Attempt to auto-resolve this reconciliation break:
        
        Break: {break}
        Available Actions: {available_actions}
        
        Determine:
        1. Can this be auto-resolved?
        2. What action to take?
        3. Confidence in resolution
        4. Any manual intervention needed?
        
        Format as JSON.
        """
    
    async def reconcile_trade(self,
                             internal_trade: Dict[str, Any],
                             external_trade: Dict[str, Any]) -> ReconciliationResult:
        """
        Reconcile a trade between internal and external systems.
        """
        trade_id = internal_trade.get("trade_id", external_trade.get("trade_id", "unknown"))
        
        logger.info(
            "Reconciling trade",
            trade_id=trade_id,
            internal_symbol=internal_trade.get("symbol"),
            external_symbol=external_trade.get("symbol")
        )
        
        # AI reconciliation
        prompt = self.trade_reconciliation_prompt.format(
            internal_trade=internal_trade,
            external_trade=external_trade
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
            
            reconciliation_data = json.loads(response.content[0].text)
            
            # Create breaks if any
            breaks = []
            if reconciliation_data.get("status") != "MATCHED":
                break_id = f"break_{trade_id}_{int(datetime.utcnow().timestamp())}"
                
                break_obj = ReconciliationBreak(
                    break_id=break_id,
                    break_type=BreakType(reconciliation_data.get("break_type", "PRICE_DIFFERENCE")),
                    internal_value=internal_trade,
                    external_value=external_trade,
                    difference=float(reconciliation_data.get("difference", 0)),
                    confidence=float(reconciliation_data.get("confidence", 0.5)),
                    explanation=reconciliation_data.get("explanation", ""),
                    suggested_action=reconciliation_data.get("suggested_action", ""),
                    auto_resolvable=reconciliation_data.get("auto_resolvable", False),
                    created_at=datetime.utcnow(),
                    resolved_at=None
                )
                
                breaks.append(break_obj)
                self.break_history.append(break_obj)
            
            # Create result
            result = ReconciliationResult(
                entity_type="TRADE",
                entity_id=trade_id,
                status=ReconciliationStatus(reconciliation_data.get("status", "PENDING")),
                internal_state=internal_trade,
                external_state=external_trade,
                breaks=breaks,
                ai_summary=reconciliation_data.get("explanation", ""),
                confidence=float(reconciliation_data.get("confidence", 0.5)),
                timestamp=datetime.utcnow()
            )
            
            self.active_reconciliations[trade_id] = result
            
            # Try auto-resolution if needed
            if breaks and breaks[0].auto_resolvable:
                await self._attempt_auto_resolution(trade_id, breaks[0])
            
            return result
            
        except Exception as e:
            logger.error(
                "Trade reconciliation failed",
                trade_id=trade_id,
                error=str(e)
            )
            
            # Create error result
            return ReconciliationResult(
                entity_type="TRADE",
                entity_id=trade_id,
                status=ReconciliationStatus.PENDING,
                internal_state=internal_trade,
                external_state=external_trade,
                breaks=[],
                ai_summary=f"Reconciliation failed: {str(e)}",
                confidence=0.0,
                timestamp=datetime.utcnow()
            )
    
    async def reconcile_position(self,
                                internal_position: Dict[str, Any],
                                external_position: Dict[str, Any]) -> ReconciliationResult:
        """
        Reconcile position between internal and external systems.
        """
        position_key = f"{internal_position.get('symbol', 'unknown')}:{internal_position.get('account', 'unknown')}"
        
        logger.info(
            "Reconciling position",
            symbol=internal_position.get("symbol"),
            account=internal_position.get("account")
        )
        
        # AI position reconciliation
        prompt = self.position_reconciliation_prompt.format(
            internal_position=internal_position,
            external_position=external_position
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
            
            reconciliation_data = json.loads(response.content[0].text)
            
            # Process position breaks
            breaks = []
            if reconciliation_data.get("breaks"):
                for break_data in reconciliation_data["breaks"]:
                    break_obj = ReconciliationBreak(
                        break_id=f"pos_break_{position_key}_{int(datetime.utcnow().timestamp())}",
                        break_type=BreakType(break_data.get("type", "QUANTITY_MISMATCH")),
                        internal_value=break_data.get("internal_value"),
                        external_value=break_data.get("external_value"),
                        difference=float(break_data.get("difference", 0)),
                        confidence=float(break_data.get("confidence", 0.5)),
                        explanation=break_data.get("explanation", ""),
                        suggested_action=break_data.get("resolution", ""),
                        auto_resolvable=break_data.get("auto_resolvable", False),
                        created_at=datetime.utcnow(),
                        resolved_at=None
                    )
                    breaks.append(break_obj)
                    self.break_history.append(break_obj)
            
            result = ReconciliationResult(
                entity_type="POSITION",
                entity_id=position_key,
                status=ReconciliationStatus(reconciliation_data.get("status", "PENDING")),
                internal_state=internal_position,
                external_state=external_position,
                breaks=breaks,
                ai_summary=reconciliation_data.get("summary", ""),
                confidence=float(reconciliation_data.get("confidence", 0.5)),
                timestamp=datetime.utcnow()
            )
            
            self.active_reconciliations[position_key] = result
            
            return result
            
        except Exception as e:
            logger.error(
                "Position reconciliation failed",
                position_key=position_key,
                error=str(e)
            )
            
            return ReconciliationResult(
                entity_type="POSITION",
                entity_id=position_key,
                status=ReconciliationStatus.PENDING,
                internal_state=internal_position,
                external_state=external_position,
                breaks=[],
                ai_summary=f"Reconciliation failed: {str(e)}",
                confidence=0.0,
                timestamp=datetime.utcnow()
            )
    
    async def reconcile_cash(self,
                            internal_cash: Dict[str, Any],
                            external_cash: Dict[str, Any]) -> ReconciliationResult:
        """
        Reconcile cash balances.
        """
        account_id = internal_cash.get("account", "unknown")
        
        # Simple cash reconciliation
        internal_balance = float(internal_cash.get("balance", 0))
        external_balance = float(external_cash.get("balance", 0))
        difference = abs(internal_balance - external_balance)
        
        # AI analysis of cash differences
        if difference > 0.01:  # More than 1 cent difference
            prompt = f"""
            Analyze cash balance discrepancy:
            
            Internal: ${internal_balance:.2f}
            External: ${external_balance:.2f}
            Difference: ${difference:.2f}
            
            Account: {account_id}
            
            Identify likely causes and resolution steps.
            Format as JSON.
            """
            
            try:
                response = await self.client.messages.create(
                    model="claude-3-sonnet-20240229",
                    max_tokens=500,
                    messages=[{
                        "role": "user",
                        "content": prompt
                    }]
                )
                
                analysis = json.loads(response.content[0].text)
                
                break_obj = ReconciliationBreak(
                    break_id=f"cash_break_{account_id}_{int(datetime.utcnow().timestamp())}",
                    break_type=BreakType.QUANTITY_MISMATCH,
                    internal_value=internal_balance,
                    external_value=external_balance,
                    difference=difference,
                    confidence=0.8,
                    explanation=analysis.get("cause", "Unknown"),
                    suggested_action=analysis.get("resolution", "Manual review"),
                    auto_resolvable=False,
                    created_at=datetime.utcnow(),
                    resolved_at=None
                )
                
                self.break_history.append(break_obj)
                
                result = ReconciliationResult(
                    entity_type="CASH",
                    entity_id=account_id,
                    status=ReconciliationStatus.MISMATCH,
                    internal_state=internal_cash,
                    external_state=external_cash,
                    breaks=[break_obj],
                    ai_summary=analysis.get("explanation", ""),
                    confidence=0.8,
                    timestamp=datetime.utcnow()
                )
            else:
                result = ReconciliationResult(
                    entity_type="CASH",
                    entity_id=account_id,
                    status=ReconciliationStatus.MISMATCH,
                    internal_state=internal_cash,
                    external_state=external_cash,
                    breaks=[],
                    ai_summary=f"Cash difference of ${difference:.2f}",
                    confidence=0.5,
                    timestamp=datetime.utcnow()
                )
        else:
            result = ReconciliationResult(
                entity_type="CASH",
                entity_id=account_id,
                status=ReconciliationStatus.MATCHED,
                internal_state=internal_cash,
                external_state=external_cash,
                breaks=[],
                ai_summary="Cash balances match",
                confidence=1.0,
                timestamp=datetime.utcnow()
            )
        
        self.active_reconciliations[f"cash_{account_id}"] = result
        return result
    
    async def _attempt_auto_resolution(self, entity_id: str, break_obj: ReconciliationBreak):
        """
        Attempt to automatically resolve a break.
        """
        available_actions = [
            "ADJUST_INTERNAL",
            "ADJUST_EXTERNAL",
            "CREATE_ADJUSTMENT_ENTRY",
            "IGNORE_WITHIN_TOLERANCE",
            "ESCALATE_TO_HUMAN"
        ]
        
        prompt = self.auto_resolution_prompt.format(
            break=asdict(break_obj),
            available_actions=available_actions
        )
        
        try:
            response = await self.client.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=500,
                messages=[{
                    "role": "user",
                    "content": prompt
                }]
            )
            
            resolution = json.loads(response.content[0].text)
            
            if resolution.get("auto_resolvable", False):
                action = resolution.get("action", "IGNORE_WITHIN_TOLERANCE")
                
                # Mark as resolved
                break_obj.resolved_at = datetime.utcnow()
                
                # Update reconciliation result
                if entity_id in self.active_reconciliations:
                    self.active_reconciliations[entity_id].status = ReconciliationStatus.RESOLVED
                
                logger.info(
                    "Auto-resolved reconciliation break",
                    break_id=break_obj.break_id,
                    action=action
                )
                
        except Exception as e:
            logger.error(
                "Auto-resolution failed",
                break_id=break_obj.break_id,
                error=str(e)
            )
    
    async def batch_reconcile(self,
                             entity_type: str,
                             internal_records: List[Dict],
                             external_records: List[Dict]) -> List[ReconciliationResult]:
        """
        Perform batch reconciliation.
        """
        results = []
        
        if entity_type == "TRADE":
            # Match trades by ID or timestamp
            for internal in internal_records:
                matching_external = None
                trade_id = internal.get("trade_id")
                
                if trade_id:
                    matching_external = next(
                        (e for e in external_records if e.get("trade_id") == trade_id),
                        None
                    )
                
                if matching_external:
                    result = await self.reconcile_trade(internal, matching_external)
                    results.append(result)
                else:
                    # Missing external trade
                    break_obj = ReconciliationBreak(
                        break_id=f"missing_{trade_id}_{int(datetime.utcnow().timestamp())}",
                        break_type=BreakType.MISSING_TRADE,
                        internal_value=internal,
                        external_value=None,
                        difference=0,
                        confidence=0.9,
                        explanation="Trade exists internally but not in external system",
                        suggested_action="ESCALATE_TO_HUMAN",
                        auto_resolvable=False,
                        created_at=datetime.utcnow(),
                        resolved_at=None
                    )
                    
                    result = ReconciliationResult(
                        entity_type="TRADE",
                        entity_id=trade_id,
                        status=ReconciliationStatus.MISMATCH,
                        internal_state=internal,
                        external_state={},
                        breaks=[break_obj],
                        ai_summary="Missing external trade",
                        confidence=0.9,
                        timestamp=datetime.utcnow()
                    )
                    
                    results.append(result)
        
        return results
    
    async def get_reconciliation_summary(self,
                                       hours: int = 24) -> Dict[str, Any]:
        """
        Get AI-generated reconciliation summary.
        """
        cutoff_time = datetime.utcnow() - timedelta(hours=hours)
        
        recent_results = [
            r for r in self.active_reconciliations.values()
            if r.timestamp > cutoff_time
        ]
        
        if not recent_results:
            return {"message": "No recent reconciliations"}
        
        # Generate AI summary
        summary_prompt = f"""
        Analyze these reconciliation results:
        
        Results: {[asdict(r) for r in recent_results[:10]]}
        
        Provide:
        1. Overall reconciliation health
        2. Common break types
        3. Resolution success rate
        4. Recommendations for improvement
        5. Risk assessment
        
        Format as JSON.
        """
        
        try:
            response = await self.client.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=1500,
                messages=[{
                    "role": "user",
                    "content": summary_prompt
                }]
            )
            
            return json.loads(response.content[0].text)
            
        except Exception as e:
            logger.error("Summary generation failed", error=str(e))
            return {"error": "Summary unavailable"}
    
    def get_active_breaks(self) -> List[ReconciliationBreak]:
        """Get all active (unresolved) breaks."""
        return [b for b in self.break_history if b.resolved_at is None]
    
    def get_reconciliation(self, entity_id: str) -> Optional[ReconciliationResult]:
        """Get specific reconciliation result."""
        return self.active_reconciliations.get(entity_id)
