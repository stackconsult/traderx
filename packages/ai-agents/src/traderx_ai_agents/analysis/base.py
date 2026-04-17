"""
Base analysis agent with LangGraph integration and circuit breaker patterns.
"""

from abc import ABC, abstractmethod
from typing import Dict, Any, Optional, List, Union
from dataclasses import dataclass
from enum import Enum
import asyncio
import logging
from datetime import datetime, timedelta

from langgraph.graph import StateGraph, END
from langchain_core.messages import HumanMessage, AIMessage
from langchain_openai import ChatOpenAI
from langchain_anthropic import ChatAnthropic

logger = logging.getLogger(__name__)


class ExecutionMode(Enum):
    """Agent execution mode"""
    SEQUENTIAL = "sequential"
    CONCURRENT = "concurrent"
    HYBRID = "hybrid"


class CircuitState(Enum):
    """Circuit breaker state"""
    CLOSED = "closed"  # Normal operation
    OPEN = "open"      # Failing, reject calls
    HALF_OPEN = "half_open"  # Testing if service recovered


@dataclass
class AnalysisResult:
    """Standardized analysis result"""
    agent_name: str
    analysis: str
    confidence: float  # 0.0 to 1.0
    data: Dict[str, Any]
    timestamp: datetime
    execution_time_ms: float
    model_used: str


class CircuitBreaker:
    """Circuit breaker for LLM API calls with exponential backoff retry"""
    
    def __init__(
        self,
        failure_threshold: int = 5,
        timeout_seconds: int = 60,
        expected_exception: type = Exception,
        max_retries: int = 3,
        base_delay: float = 1.0,
        max_delay: float = 30.0
    ):
        self.failure_threshold = failure_threshold
        self.timeout_seconds = timeout_seconds
        self.expected_exception = expected_exception
        self.max_retries = max_retries
        self.base_delay = base_delay
        self.max_delay = max_delay
        
        self.failure_count = 0
        self.last_failure_time: Optional[datetime] = None
        self.state = CircuitState.CLOSED
        
    async def call(self, func, *args, **kwargs):
        """Execute function with circuit breaker protection and retry logic"""
        if self.state == CircuitState.OPEN:
            if self._should_attempt_reset():
                self.state = CircuitState.HALF_OPEN
            else:
                raise Exception("Circuit breaker is OPEN")
                
        last_exception = None
        
        for attempt in range(self.max_retries + 1):
            try:
                result = await func(*args, **kwargs)
                self._on_success()
                return result
                
            except self.expected_exception as e:
                last_exception = e
                
                # Don't retry on last attempt
                if attempt == self.max_retries:
                    self._on_failure()
                    raise e
                    
                # Calculate delay with exponential backoff
                delay = min(
                    self.base_delay * (2 ** attempt),
                    self.max_delay
                )
                
                logger.warning(
                    f"Attempt {attempt + 1} failed, retrying in {delay:.2f}s: {str(e)}"
                )
                await asyncio.sleep(delay)
            
    def _should_attempt_reset(self) -> bool:
        """Check if enough time has passed to attempt reset"""
        if not self.last_failure_time:
            return False
        return datetime.now() - self.last_failure_time > timedelta(seconds=self.timeout_seconds)
        
    def _on_success(self):
        """Handle successful call"""
        self.failure_count = 0
        self.state = CircuitState.CLOSED
        
    def _on_failure(self):
        """Handle failed call"""
        self.failure_count += 1
        self.last_failure_time = datetime.now()
        
        if self.failure_count >= self.failure_threshold:
            self.state = CircuitState.OPEN
            logger.warning(f"Circuit breaker OPENED after {self.failure_count} failures")


class BaseAnalysisAgent(ABC):
    """
    Base class for all analysis agents with LangGraph integration.
    
    Features:
    - Configurable execution modes
    - Circuit breaker for LLM API protection
    - Caching for repeated queries
    - Structured output with confidence scores
    """
    
    def __init__(
        self,
        name: str,
        llm_provider: str = "openai",
        model_name: str = "gpt-4-turbo-preview",
        execution_mode: ExecutionMode = ExecutionMode.HYBRID,
        cache_results: bool = True,
    ):
        self.name = name
        self.execution_mode = execution_mode
        self.cache_results = cache_results
        
        # Initialize LLM
        if llm_provider == "openai":
            self.llm = ChatOpenAI(model=model_name, temperature=0.1)
        elif llm_provider == "anthropic":
            self.llm = ChatAnthropic(model=model_name, temperature=0.1)
        else:
            raise ValueError(f"Unsupported LLM provider: {llm_provider}")
            
        # Circuit breaker for API calls
        self.circuit_breaker = CircuitBreaker(
            failure_threshold=5,
            timeout_seconds=60,
            expected_exception=Exception
        )
        
        # Cache for results
        self._cache: Dict[str, AnalysisResult] = {}
        
        # LangGraph workflow
        self._build_workflow()
        
    def _build_workflow(self):
        """Build the LangGraph workflow for this agent"""
        workflow = StateGraph(dict)
        
        # Add nodes
        workflow.add_node("analyze", self._analyze_node)
        workflow.add_node("validate", self._validate_node)
        workflow.add_node("format", self._format_node)
        
        # Add edges
        workflow.add_edge("analyze", "validate")
        workflow.add_edge("validate", "format")
        workflow.add_edge("format", END)
        
        # Set entry point
        workflow.set_entry_point("analyze")
        
        # Compile workflow
        self.workflow = workflow.compile()
        
    async def analyze(self, input_data: Dict[str, Any]) -> AnalysisResult:
        """
        Analyze input data and return structured result.
        
        Args:
            input_data: Input data for analysis
            
        Returns:
            AnalysisResult with structured output and confidence
        """
        start_time = datetime.now()
        
        # Check cache first
        if self.cache_results:
            cache_key = self._generate_cache_key(input_data)
            if cache_key in self._cache:
                logger.debug(f"Cache hit for {self.name}")
                return self._cache[cache_key]
                
        try:
            # Execute analysis through workflow
            result = await self.workflow.ainvoke(input_data)
            
            # Create analysis result
            analysis_result = AnalysisResult(
                agent_name=self.name,
                analysis=result.get("analysis", ""),
                confidence=result.get("confidence", 0.5),
                data=result.get("data", {}),
                timestamp=start_time,
                execution_time_ms=(datetime.now() - start_time).total_seconds() * 1000,
                model_used=self.llm.model_name
            )
            
            # Cache result
            if self.cache_results:
                self._cache[cache_key] = analysis_result
                
            return analysis_result
            
        except Exception as e:
            logger.error(f"Analysis failed for {self.name}: {e}")
            # Return fallback result
            return AnalysisResult(
                agent_name=self.name,
                analysis=f"Analysis failed: {str(e)}",
                confidence=0.0,
                data={"error": str(e)},
                timestamp=start_time,
                execution_time_ms=(datetime.now() - start_time).total_seconds() * 1000,
                model_used="fallback"
            )
            
    async def _analyze_node(self, state: Dict[str, Any]) -> Dict[str, Any]:
        """Core analysis node - to be implemented by subclasses"""
        prompt = self._build_prompt(state)
        
        try:
            # Make LLM call with circuit breaker
            response = await self.circuit_breaker.call(
                self.llm.ainvoke, [HumanMessage(content=prompt)]
            )
            
            return {
                "raw_analysis": response.content,
                "state": state
            }
            
        except Exception as e:
            logger.error(f"LLM call failed for {self.name}: {e}")
            return {
                "raw_analysis": f"LLM call failed: {str(e)}",
                "state": state,
                "error": str(e)
            }
            
    async def _validate_node(self, state: Dict[str, Any]) -> Dict[str, Any]:
        """Validate analysis and extract confidence"""
        raw_analysis = state.get("raw_analysis", "")
        
        # Extract confidence from analysis (look for patterns)
        confidence = self._extract_confidence(raw_analysis)
        
        # Validate analysis quality
        is_valid = self._validate_analysis(raw_analysis)
        
        return {
            **state,
            "confidence": confidence if is_valid else 0.0,
            "is_valid": is_valid
        }
        
    async def _format_node(self, state: Dict[str, Any]) -> Dict[str, Any]:
        """Format final analysis result"""
        return {
            "analysis": state.get("raw_analysis", ""),
            "confidence": state.get("confidence", 0.0),
            "data": {
                "is_valid": state.get("is_valid", False),
                "execution_mode": self.execution_mode.value,
                "agent_type": self.__class__.__name__
            }
        }
        
    @abstractmethod
    def _build_prompt(self, input_data: Dict[str, Any]) -> str:
        """Build the analysis prompt - must be implemented by subclasses"""
        pass
        
    def _extract_confidence(self, analysis: str) -> float:
        """Extract confidence score from analysis text"""
        # Look for confidence indicators
        confidence_patterns = [
            "confidence: ",
            "certainty: ",
            "probability: ",
            "likelihood: "
        ]
        
        for pattern in confidence_patterns:
            if pattern.lower() in analysis.lower():
                try:
                    # Extract number after pattern
                    start = analysis.lower().find(pattern.lower()) + len(pattern)
                    end = analysis.find("%", start)
                    if end > start:
                        confidence_str = analysis[start:end].strip()
                        return float(confidence_str) / 100.0
                except:
                    pass
                    
        # Default confidence based on analysis length and certainty words
        high_certainty_words = ["definitely", "certainly", "clearly", "strongly"]
        low_certainty_words = ["might", "could", "possibly", "potentially"]
        
        high_count = sum(1 for word in high_certainty_words if word in analysis.lower())
        low_count = sum(1 for word in low_certainty_words if word in analysis.lower())
        
        base_confidence = 0.5
        confidence = base_confidence + (high_count * 0.1) - (low_count * 0.1)
        return max(0.0, min(1.0, confidence))
        
    def _validate_analysis(self, analysis: str) -> bool:
        """Validate that analysis meets minimum quality standards"""
        if not analysis or len(analysis) < 50:
            return False
            
        # Check for error indicators
        error_indicators = [
            "i cannot",
            "i'm unable",
            "i don't have",
            "as an ai",
            "i'm sorry"
        ]
        
        analysis_lower = analysis.lower()
        for indicator in error_indicators:
            if indicator in analysis_lower:
                return False
                
        return True
        
    def _generate_cache_key(self, input_data: Dict[str, Any]) -> str:
        """Generate cache key from input data"""
        # Simple hash-based key generation
        import hashlib
        import json
        
        # Sort keys for consistent hashing
        sorted_data = json.dumps(input_data, sort_keys=True)
        return hashlib.md5(sorted_data.encode()).hexdigest()
        
    async def batch_analyze(
        self,
        inputs: List[Dict[str, Any]],
        max_concurrent: int = 5
    ) -> List[AnalysisResult]:
        """
        Analyze multiple inputs with configurable concurrency.
        
        Args:
            inputs: List of input data to analyze
            max_concurrent: Maximum concurrent analyses
            
        Returns:
            List of analysis results
        """
        if self.execution_mode == ExecutionMode.SEQUENTIAL:
            # Process sequentially
            results = []
            for input_data in inputs:
                result = await self.analyze(input_data)
                results.append(result)
            return results
            
        elif self.execution_mode == ExecutionMode.CONCURRENT:
            # Process concurrently with semaphore
            semaphore = asyncio.Semaphore(max_concurrent)
            
            async def analyze_with_semaphore(input_data):
                async with semaphore:
                    return await self.analyze(input_data)
                    
            tasks = [analyze_with_semaphore(input_data) for input_data in inputs]
            return await asyncio.gather(*tasks)
            
        else:  # HYBRID
            # Use concurrent for independent analyses
            return await self.batch_analyze(inputs, max_concurrent)
