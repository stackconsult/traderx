"""
Base LLM Provider Interface
Abstract base class for all LLM providers (Claude, Gemma, OpenAI, etc.).
"""

from abc import ABC, abstractmethod
from typing import Dict, Any, List, Optional, AsyncGenerator
from dataclasses import dataclass
import asyncio


@dataclass
class LLMMessage:
    """Message for LLM conversation."""
    role: str  # "system", "user", "assistant"
    content: str
    metadata: Optional[Dict[str, Any]] = None


@dataclass
class LLMResponse:
    """Response from LLM."""
    content: str
    tokens_used: int
    model: str
    finish_reason: str
    response_time_ms: float
    metadata: Optional[Dict[str, Any]] = None


@dataclass
class GenerationConfig:
    """Configuration for text generation."""
    max_tokens: int = 4096
    temperature: float = 0.7
    top_p: float = 0.9
    top_k: int = 40
    frequency_penalty: float = 0.0
    presence_penalty: float = 0.0
    stop_sequences: Optional[List[str]] = None


class LLMProvider(ABC):
    """Abstract base class for LLM providers."""
    
    def __init__(self, model_name: str, api_key: Optional[str] = None, **kwargs):
        self.model_name = model_name
        self.api_key = api_key
        self.config = kwargs
        
    @abstractmethod
    async def generate(self, 
                     messages: List[LLMMessage], 
                     config: Optional[GenerationConfig] = None) -> LLMResponse:
        """Generate text from messages."""
        pass
    
    @abstractmethod
    async def generate_stream(self, 
                            messages: List[LLMMessage], 
                            config: Optional[GenerationConfig] = None) -> AsyncGenerator[str, None]:
        """Generate text with streaming."""
        pass
    
    @abstractmethod
    async def count_tokens(self, text: str) -> int:
        """Count tokens in text."""
        pass
    
    @abstractmethod
    def get_model_info(self) -> Dict[str, Any]:
        """Get model information."""
        pass
    
    async def health_check(self) -> bool:
        """Check if provider is healthy."""
        try:
            test_messages = [
                LLMMessage(role="user", content="Hello")
            ]
            response = await self.generate(test_messages)
            return response.content is not None
        except:
            return False
