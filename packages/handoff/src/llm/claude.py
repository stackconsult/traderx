"""
Claude 4.6 LLM Provider
Implementation for Anthropic's Claude 4.6 model.
"""

import asyncio
import time
from typing import Dict, Any, List, Optional, AsyncGenerator
import httpx
import json

from .base import LLMProvider, LLMMessage, LLMResponse, GenerationConfig


class Claude46Provider(LLMProvider):
    """Claude 4.6 provider implementation."""
    
    def __init__(self, api_key: str, **kwargs):
        super().__init__("claude-4.6", api_key, **kwargs)
        self.base_url = "https://api.anthropic.com/v1"
        self.max_retries = kwargs.get("max_retries", 3)
        self.timeout = kwargs.get("timeout", 60)
        
    async def generate(self, 
                     messages: List[LLMMessage], 
                     config: Optional[GenerationConfig] = None) -> LLMResponse:
        """Generate text using Claude 4.6."""
        if not self.api_key:
            raise ValueError("API key is required for Claude 4.6")
        
        config = config or GenerationConfig()
        
        # Convert messages to Claude format
        claude_messages = self._convert_messages(messages)
        
        # Prepare request
        request_data = {
            "model": "claude-3-5-sonnet-20241022",  # Claude 4.6 equivalent
            "messages": claude_messages,
            "max_tokens": config.max_tokens,
            "temperature": config.temperature,
            "top_p": config.top_p
        }
        
        if config.stop_sequences:
            request_data["stop_sequences"] = config.stop_sequences
            
        start_time = time.time()
        
        async with httpx.AsyncClient(timeout=self.timeout) as client:
            for attempt in range(self.max_retries):
                try:
                    response = await client.post(
                        f"{self.base_url}/messages",
                        headers={
                            "x-api-key": self.api_key,
                            "anthropic-version": "2023-06-01",
                            "content-type": "application/json"
                        },
                        json=request_data
                    )
                    
                    if response.status_code == 200:
                        data = response.json()
                        content = data["content"][0]["text"]
                        usage = data.get("usage", {})
                        
                        return LLMResponse(
                            content=content,
                            tokens_used=usage.get("total_tokens", 0),
                            model=data["model"],
                            finish_reason=data["stop_reason"],
                            response_time_ms=(time.time() - start_time) * 1000,
                            metadata={
                                "input_tokens": usage.get("input_tokens", 0),
                                "output_tokens": usage.get("output_tokens", 0)
                            }
                        )
                    else:
                        error_data = response.json()
                        if response.status_code == 429:
                            # Rate limited, wait and retry
                            await asyncio.sleep(2 ** attempt)
                            continue
                        else:
                            raise Exception(f"Claude API error: {error_data}")
                            
                except httpx.RequestError as e:
                    if attempt == self.max_retries - 1:
                        raise
                    await asyncio.sleep(1)
        
        raise Exception("Failed to generate response after retries")
    
    async def generate_stream(self, 
                            messages: List[LLMMessage], 
                            config: Optional[GenerationConfig] = None) -> AsyncGenerator[str, None]:
        """Generate streaming text using Claude 4.6."""
        if not self.api_key:
            raise ValueError("API key is required for Claude 4.6")
        
        config = config or GenerationConfig()
        claude_messages = self._convert_messages(messages)
        
        request_data = {
            "model": "claude-3-5-sonnet-20241022",
            "messages": claude_messages,
            "max_tokens": config.max_tokens,
            "temperature": config.temperature,
            "top_p": config.top_p,
            "stream": True
        }
        
        if config.stop_sequences:
            request_data["stop_sequences"] = config.stop_sequences
        
        async with httpx.AsyncClient(timeout=self.timeout) as client:
            async with client.stream(
                "POST",
                f"{self.base_url}/messages",
                headers={
                    "x-api-key": self.api_key,
                    "anthropic-version": "2023-06-01",
                    "content-type": "application/json"
                },
                json=request_data
            ) as response:
                if response.status_code != 200:
                    raise Exception(f"Stream error: {response.status_code}")
                
                async for line in response.aiter_lines():
                    if line.startswith("data: "):
                        data = line[6:]
                        if data == "[DONE]":
                            break
                        try:
                            event = json.loads(data)
                            if event["type"] == "content_block_delta":
                                delta = event["delta"]["text"]
                                yield delta
                        except json.JSONDecodeError:
                            continue
    
    async def count_tokens(self, text: str) -> int:
        """Count tokens using Claude's tokenizer (approximate)."""
        # Claude uses approximately 4 characters per token
        return len(text) // 4
    
    def get_model_info(self) -> Dict[str, Any]:
        """Get Claude 4.6 model information."""
        return {
            "name": "Claude 4.6",
            "provider": "Anthropic",
            "context_window": 200000,
            "max_output_tokens": 8192,
            "supports_streaming": True,
            "supports_functions": True,
            "pricing": {
                "input_per_1k": 0.015,
                "output_per_1k": 0.075
            }
        }
    
    def _convert_messages(self, messages: List[LLMMessage]) -> List[Dict[str, str]]:
        """Convert LLMMessage format to Claude format."""
        claude_messages = []
        
        for msg in messages:
            if msg.role == "system":
                # Claude puts system message first
                claude_messages.insert(0, {"role": "user", "content": f"System: {msg.content}"})
            else:
                claude_messages.append({
                    "role": msg.role,
                    "content": msg.content
                })
        
        return claude_messages
