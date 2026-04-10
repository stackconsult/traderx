"""
Meta-Coordinator - Multi-Model Handoff Orchestrator
Manages the handoff process between Claude 4.6 (Thinking) and Gemma 4 (Execution).
"""

import asyncio
import json
import logging
import uuid
from datetime import datetime, timedelta
from typing import Dict, Any, List, Optional, Type
from pathlib import Path

from .models.handoff_package import (
    HandoffPackage, HandoffStatus, AgentType, TaskDefinition,
    DeterministicPlan, ExecutionResult, HandoffError
)
from .turbo_quant import TurboQuant, CompressedContext
from .llm.base import LLMProvider, LLMMessage, GenerationConfig
from .llm.claude import Claude46Provider
from .llm.gemma import Gemma4Provider

logger = logging.getLogger(__name__)


class MetaCoordinator:
    """
    Orchestrates handoffs between thinking and execution agents.
    Maintains state, handles retries, and ensures compliance.
    """
    
    def __init__(self, 
                 claude_provider: Claude46Provider,
                 gemma_provider: Gemma4Provider,
                 turbo_quant: TurboQuant,
                 handoff_trace_path: str = "handoff-trace.json"):
        self.claude = claude_provider
        self.gemma = gemma_provider
        self.turbo_quant = turbo_quant
        self.handoff_trace_path = Path(handoff_trace_path)
        
        # Active handoffs
        self.active_handoffs: Dict[str, HandoffPackage] = {}
        
        # Configuration
        self.max_concurrent_handoffs = 10
        self.default_timeout = 300  # 5 minutes
        
        # Statistics
        self.stats = {
            "total_handoffs": 0,
            "successful_handoffs": 0,
            "failed_handoffs": 0,
            "reasoning_lock_in_errors": 0
        }
        
    async def initiate_handoff(self, task: TaskDefinition) -> HandoffPackage:
        """
        Initiate a new handoff process.
        
        Args:
            task: Task definition to be executed
            
        Returns:
            HandoffPackage for tracking
        """
        # Check concurrent handoff limit
        if len(self.active_handoffs) >= self.max_concurrent_handoffs:
            raise Exception("Maximum concurrent handoffs reached")
        
        # Create handoff package
        handoff = HandoffPackage(
            id=str(uuid.uuid4()),
            task=task,
            status=HandoffStatus.PENDING,
            source_agent=AgentType.CLAUDE_46,
            target_agent=AgentType.GEMMA_4
        )
        
        # Store active handoff
        self.active_handoffs[handoff.id] = handoff
        self.stats["total_handoffs"] += 1
        
        # Start handoff process
        asyncio.create_task(self._execute_handoff(handoff))
        
        return handoff
    
    async def _execute_handoff(self, handoff: HandoffPackage):
        """Execute the complete handoff process."""
        try:
            # Step 1: Generate plan with Claude 4.6
            await self._generate_plan(handoff)
            
            # Step 2: Compress context with TurboQuant
            await self._compress_context(handoff)
            
            # Step 3: Execute with Gemma 4
            await self._execute_plan(handoff)
            
            # Step 4: Validate reasoning lock-in
            if not handoff.validate_reasoning_lock_in():
                self.stats["reasoning_lock_in_errors"] += 1
                await self._handle_reasoning_lock_in(handoff)
                return
            
            # Complete handoff
            handoff.transition_to(HandoffStatus.COMPLETE)
            self.stats["successful_handoffs"] += 1
            
        except Exception as e:
            logger.error(f"Handoff {handoff.id} failed: {e}")
            handoff.transition_to(
                HandoffStatus.FAILED,
                HandoffError(
                    code="EXECUTION_ERROR",
                    message=str(e),
                    agent="meta-coordinator",
                    step="execution"
                )
            )
            self.stats["failed_handoffs"] += 1
            
            # Check if retry is possible
            if handoff.can_retry():
                await asyncio.sleep(2 ** handoff.retry_count)  # Exponential backoff
                handoff.increment_retry()
                asyncio.create_task(self._execute_handoff(handoff))
            
        finally:
            # Log handoff completion
            await self._log_handoff(handoff)
            
            # Remove from active if complete
            if handoff.status in [HandoffStatus.COMPLETE, HandoffStatus.FAILED, HandoffStatus.TIMEOUT]:
                self.active_handoffs.pop(handoff.id, None)
    
    async def _generate_plan(self, handoff: HandoffPackage):
        """Generate deterministic plan with Claude 4.6."""
        handoff.transition_to(HandoffStatus.ACCEPTED)
        
        # Prepare prompt for plan generation
        system_prompt = """
        You are Claude 4.6, a strategic planning AI. Generate a deterministic plan for the given task.
        
        Your plan must:
        1. Break down the task into specific, actionable steps
        2. Include dependencies between steps
        3. Define expected outcomes
        4. Provide rollback procedures
        
        Output format must be valid JSON with:
        - steps: Array of step objects with id, action, parameters, preconditions
        - dependencies: Array of dependency objects
        - expected_outcome: Object describing success criteria
        - rollback_plan: Object with rollback steps
        """
        
        user_prompt = f"""
        Task: {handoff.task.description}
        Type: {handoff.task.type}
        Priority: {handoff.task.priority}
        Context: {json.dumps(handoff.task.context, indent=2)}
        
        Generate a deterministic execution plan.
        """
        
        messages = [
            LLMMessage(role="system", content=system_prompt),
            LLMMessage(role="user", content=user_prompt)
        ]
        
        # Generate plan
        response = await self.claude.generate(messages, GenerationConfig(
            max_tokens=4096,
            temperature=0.3  # Lower temperature for deterministic output
        ))
        
        try:
            plan_data = json.loads(response.content)
            
            # Create deterministic plan
            handoff.plan = DeterministicPlan(
                id=str(uuid.uuid4()),
                task_id=handoff.task.id,
                steps=plan_data.get("steps", []),
                dependencies=plan_data.get("dependencies", []),
                expected_outcome=plan_data.get("expected_outcome", {}),
                rollback_plan=plan_data.get("rollback_plan")
            )
            
        except json.JSONDecodeError as e:
            raise Exception(f"Failed to parse plan JSON: {e}")
    
    async def _compress_context(self, handoff: HandoffPackage):
        """Compress conversation context using TurboQuant."""
        if not handoff.plan:
            raise Exception("Plan must be generated before context compression")
        
        # Prepare conversation history
        messages = [
            {
                "role": "system",
                "content": f"Task: {handoff.task.description}",
                "timestamp": handoff.created_at.isoformat()
            },
            {
                "role": "assistant",
                "content": f"Generated plan with {len(handoff.plan.steps)} steps",
                "timestamp": datetime.utcnow().isoformat()
            }
        ]
        
        # Compress context
        compressed_messages, metrics = self.turbo_quant.compress_context(
            messages, 
            target_ratio=0.1  # Compress to 10%
        )
        
        # Create compressed context
        handoff.context = CompressedContext(
            embeddings=[],  # Would be populated by TurboQuant
            metadata={
                "compression_metrics": metrics.__dict__,
                "original_messages": len(messages),
                "compressed_messages": len(compressed_messages)
            },
            original_size=metrics.original_size,
            compressed_size=metrics.compressed_size,
            compression_ratio=metrics.compression_ratio
        )
    
    async def _execute_plan(self, handoff: HandoffPackage):
        """Execute plan with Gemma 4."""
        handoff.transition_to(HandoffStatus.WORKING)
        
        # Prepare execution prompt
        system_prompt = """
        You are Gemma 4, an execution AI. Execute the provided plan exactly as specified.
        
        Rules:
        1. Follow the steps in the given order
        2. Respect all dependencies
        3. Do not deviate from the plan
        4. Report progress for each step
        5. Stop immediately if any step fails
        
        Output your reasoning trace for each step.
        """
        
        plan_prompt = f"""
        Plan to execute:
        {json.dumps({
            "steps": handoff.plan.steps,
            "dependencies": handoff.plan.dependencies,
            "expected_outcome": handoff.plan.expected_outcome
        }, indent=2)}
        
        Execute this plan and report results.
        """
        
        messages = [
            LLMMessage(role="system", content=system_prompt),
            LLMMessage(role="user", content=plan_prompt)
        ]
        
        # Execute with streaming to capture reasoning trace
        reasoning_trace = []
        full_response = ""
        
        async for chunk in self.gemma.generate_stream(messages):
            full_response += chunk
            # Capture reasoning steps (simplified)
            if "Step" in chunk or "Completed" in chunk:
                reasoning_trace.append({
                    "step_id": str(len(reasoning_trace)),
                    "content": chunk,
                    "timestamp": datetime.utcnow().isoformat()
                })
        
        # Create execution result
        handoff.execution = ExecutionResult(
            id=str(uuid.uuid4()),
            handoff_id=handoff.id,
            status="completed",
            outputs={"response": full_response},
            execution_time_ms=0,  # Would be measured
            tokens_used=0,  # Would be counted
            reasoning_trace=reasoning_trace
        )
    
    async def _handle_reasoning_lock_in(self, handoff: HandoffPackage):
        """Handle reasoning lock-in error."""
        logger.error(f"Reasoning lock-in detected in handoff {handoff.id}")
        
        # Log to JOURNAL.md as required
        await self._log_to_journal(
            f"REASONING_LOCK_IN_ERROR: Handoff {handoff.id} violated plan constraints",
            handoff.errors[-1].details if handoff.errors else {}
        )
        
        # Mark as failed
        handoff.transition_to(
            HandoffStatus.FAILED,
            HandoffError(
                code="REASONING_LOCK_IN",
                message="Execution deviated from plan",
                agent="meta-coordinator",
                step="validation"
            )
        )
    
    async def _log_handoff(self, handoff: HandoffPackage):
        """Log handoff to trace file."""
        trace_entry = {
            "handoff_id": handoff.id,
            "task_id": handoff.task.id,
            "status": handoff.status.value,
            "source_agent": handoff.source_agent.value,
            "target_agent": handoff.target_agent.value,
            "plan_hash": handoff.plan.to_hash() if handoff.plan else None,
            "execution_id": handoff.execution.id if handoff.execution else None,
            "errors": [error.__dict__ for error in handoff.errors],
            "retry_count": handoff.retry_count,
            "created_at": handoff.created_at.isoformat(),
            "completed_at": handoff.completed_at.isoformat() if handoff.completed_at else None,
            "metadata": handoff.metadata
        }
        
        # Append to trace file
        with open(self.handoff_trace_path, "a") as f:
            f.write(json.dumps(trace_entry) + "\n")
    
    async def _log_to_journal(self, message: str, details: Dict[str, Any]):
        """Log to JOURNAL.md for compliance."""
        journal_path = Path("JOURNAL.md")
        
        if journal_path.exists():
            timestamp = datetime.utcnow().isoformat()
            entry = f"\n## {timestamp}\n{message}\nDetails: {json.dumps(details, indent=2)}\n"
            
            with open(journal_path, "a") as f:
                f.write(entry)
    
    async def get_handoff_status(self, handoff_id: str) -> Optional[Dict[str, Any]]:
        """Get status of a specific handoff."""
        handoff = self.active_handoffs.get(handoff_id)
        if handoff:
            return handoff.to_dict()
        return None
    
    async def get_active_handoffs(self) -> List[Dict[str, Any]]:
        """Get all active handoffs."""
        return [h.to_dict() for h in self.active_handoffs.values()]
    
    async def get_statistics(self) -> Dict[str, Any]:
        """Get handoff statistics."""
        return {
            **self.stats,
            "active_handoffs": len(self.active_handoffs),
            "success_rate": (
                self.stats["successful_handoffs"] / self.stats["total_handoffs"]
                if self.stats["total_handoffs"] > 0 else 0
            )
        }
    
    async def shutdown(self):
        """Shutdown the meta-coordinator."""
        # Wait for active handoffs to complete or timeout
        timeout = timedelta(seconds=30)
        start_time = datetime.utcnow()
        
        while self.active_handoffs and (datetime.utcnow() - start_time) < timeout:
            await asyncio.sleep(1)
        
        # Force remaining handoffs to timeout
        for handoff in list(self.active_handoffs.values()):
            handoff.transition_to(HandoffStatus.TIMEOUT)
            await self._log_handoff(handoff)
        
        self.active_handoffs.clear()
