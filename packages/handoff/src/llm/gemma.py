"""
Gemma 4 LLM Provider
Implementation for Google's Gemma 4 model.
"""

import asyncio
import time
from typing import Dict, Any, List, Optional, AsyncGenerator
import httpx
import json

from .base import LLMProvider, LLMMessage, LLMResponse, GenerationConfig


class Gemma4Provider(LLMProvider):
    """Gemma 4 provider implementation."""
    
    def __init__(self, api_key: str, **kwargs):
        super().__init__("gemma-4", api_key, **kwargs)
        self.base_url = kwargs.get("base_url", "https://generativelanguage.googleapis.com/v1")
        self.project_id = kwargs.get("project_id")
        self.max_retries = kwargs.get("max_retries", 3)
        self.timeout = kwargs.get("timeout", 60)
        
    async def generate(self, 
                     messages: List[LLMMessage], 
                     config: Optional[GenerationConfig] = None) -> LLMResponse:
        """Generate text using Gemma 4."""
        if not self.api_key:
            raise ValueError("API key is required for Gemma 4")
        
        config = config or GenerationConfig()
        
        # Convert messages to Gemini format
        gemini_messages = self._convert_messages(messages)
        
        # Prepare request
        request_data = {
            "contents": gemini_messages,
            "generationConfig": {
                "maxOutputTokens": config.max_tokens,
                "temperature": config.temperature,
                "topP": config.top_p,
                "topK": config.top_k
            },
            "safetySettings": [
                {
                    "category": "HARM_CATEGORY_HARASSMENT",
                    "threshold": "BLOCK_NONE"
                },
                {
                    "category": "HARM_CATEGORY_HATE_SPEECH",
                    "threshold": "BLOCK_NONE"
                },
                {
                    "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT",
                    "threshold": "BLOCK_NONE"
                },
                {
                    "category": "HARM_CATEGORY_DANGEROUS_CONTENT",
                    "threshold": "BLOCK_NONE"
                }
            ]
        }
        
        if config.stop_sequences:
            request_data["generationConfig"]["stopSequences"] = config.stop_sequences
            
        start_time = time.time()
        
        url = f"{self.base_url}/models/gemma-1.5-pro:generateContent?key={self.api_key}"
        
        async with httpx.AsyncClient(timeout=self.timeout) as client:
            for attempt in range(self.max_retries):
                try:
                    response = await client.post(url, json=request_data)
                    
                    if response.status_code == 200:
                        data = response.json()
                        
                        if "candidates" in data and data["candidates"]:
                            candidate = data["candidates"][0]
                            content = candidate["content"]["parts"][0]["text"]
                            usage = data.get("usageMetadata", {})
                            
                            return LLMResponse(
                                content=content,
                                tokens_used=usage.get("totalTokenCount", 0),
                                model="gemma-1.5-pro",
                                finish_reason=candidate.get("finishReason", "STOP"),
                                response_time_ms=(time.time() - start_time) * 1000,
                                metadata={
                                    "input_tokens": usage.get("promptTokenCount", 0),
                                    "output_tokens": usage.get("candidatesTokenCount", 0)
                                }
                            )
                        else:
                            raise Exception("No candidates in response")
                    else:
                        error_data = response.json()
                        if response.status_code == 429:
                            # Rate limited, wait and retry
                            await asyncio.sleep(2 ** attempt)
                            continue
                        else:
                            raise Exception(f"Gemma API error: {error_data}")
                            
                except httpx.RequestError as e:
                    if attempt == self.max_retries - 1:
                        raise
                    await asyncio.sleep(1)
        
        raise Exception("Failed to generate response after retries")
    
    async def generate_stream(self, 
                            messages: List[LLMMessage], 
                            config: Optional[GenerationConfig] = None) -> AsyncGenerator[str, None]:
        """Generate streaming text using Gemma 4."""
        if not self.api_key:
            raise ValueError("API key is required for Gemma 4")
        
        config = config or GenerationConfig()
        gemini_messages = self._convert_messages(messages)
        
        request_data = {
            "contents": gemini_messages,
            "generationConfig": {
                "maxOutputTokens": config.max_tokens,
                "temperature": config.temperature,
                "topP": config.top_p,
                "topK": config.top_k
            },
            "safetySettings": [
                {
                    "category": "HARM_CATEGORY_HARASSMENT",
                    "threshold": "BLOCK_NONE"
                },
                {
                    "category": "HARM_CATEGORY_HATE_SPEECH",
                    "threshold": "BLOCK_NONE"
                },
                {
                    "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT",
                    "threshold": "BLOCK_NONE"
                },
                {
                    "category": "HARM_CATEGORY_DANGEROUS_CONTENT",
                    "threshold": "BLOCK_NONE"
                }
            ]
        }
        
        url = f"{self.base_url}/models/gemma-1.5-pro:streamGenerateContent?key={self.api_key}"
        
        async with httpx.AsyncClient(timeout=self.timeout) as client:
            async with client.stream("POST", url, json=request_data) as response:
                if response.status_code != 200:
                    raise Exception(f"Stream error: {response.status_code}")
                
                async for line in response.aiter_lines():
                    if line.startswith("data: "):
                        data = line[6:]
                        try:
                            event = json.loads(data)
                            if "candidates" in event and event["candidates"]:
                                candidate = event["candidates"][0]
                                if "content" in candidate and "parts" in candidate["content"]:
                                    delta = candidate["content"]["parts"][0].get("text", "")
                                    if delta:
                                        yield delta
                        except json.JSONDecodeError:
                            continue
    
    async def count_tokens(self, text: str) -> int:
        """Count tokens using Gemma's tokenizer (approximate)."""
        # Gemma uses approximately 4 characters per token
        return len(text) // 4
    
    def get_model_info(self) -> Dict[str, Any]:
        """Get Gemma 4 model information."""
        return {
            "name": "Gemma 4",
            "provider": "Google",
            "context_window": 1000000,
            "max_output_tokens": 8192,
            "supports_streaming": True,
            "supports_functions": True,
            "pricing": {
                "input_per_1k": 0.00125,
                "output_per_1k": 0.00375
            }
        }
    
    def _convert_messages(self, messages: List[LLMMessage]) -> List[Dict[str, Any]]:
        """Convert LLMMessage format to Gemini format."""
        gemini_messages = []
        
        for msg in messages:
            if msg.role == "system":
                # Gemini doesn't have system messages, prepend to first user message
                if gemini_messages and gemini_messages[0]["role"] == "user":
                    gemini_messages[0]["parts"][0]["text"] = f"System: {msg.content}\n\n{gemini_messages[0]['parts'][0]['text']}"
                else:
                    gemini_messages.append({
                        "role": "user",
                        "parts": [{"text": f"System: {msg.content}"}]
                    })
            else:
                gemini_messages.append({
                    "role": msg.role,
                    "parts": [{"text": msg.content}]
                })
        
        return gemini_messages
