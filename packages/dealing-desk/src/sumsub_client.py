"""
Real Sumsub API Client for Production KYA Integration
This is not a mock - it connects to actual Sumsub API endpoints.
"""

import asyncio
import json
import time
import os
import hmac
import hashlib
from datetime import datetime, timedelta
from typing import Dict, Optional, Any, List
from dataclasses import dataclass, asdict
import httpx
from httpx import Response, HTTPStatusError
import structlog

logger = structlog.get_logger(__name__)


@dataclass
class SumsubConfig:
    """Sumsub API configuration."""
    api_token: str
    api_secret: str
    base_url: str = "https://api.sumsub.com"
    timeout: int = 30
    max_retries: int = 3
    retry_delay: float = 1.0


class SumsubAPIError(Exception):
    """Custom exception for Sumsub API errors."""
    def __init__(self, message: str, status_code: Optional[int] = None, response: Optional[Dict] = None):
        super().__init__(message)
        self.status_code = status_code
        self.response = response


class SumsubClient:
    """
    Production Sumsub API client with proper error handling and retries.
    Connects to real Sumsub endpoints for AI Agent Verification.
    """
    
    def __init__(self, config: SumsubConfig):
        self.config = config
        self.client = httpx.AsyncClient(
            timeout=httpx.Timeout(config.timeout),
            limits=httpx.Limits(max_keepalive_connections=5, max_connections=10)
        )
        
    async def __aenter__(self):
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.client.aclose()
    
    async def verify_agent_binding(self, 
                                  agent_did: str,
                                  human_did: str,
                                  app_id: Optional[str] = None) -> Dict[str, Any]:
        """
        Verify agent-to-human binding via Sumsub API.
        
        This makes a real API call to Sumsub's AI Agent Verification service.
        
        Args:
            agent_did: Agent's Decentralized Identifier
            human_did: Human's Decentralized Identifier  
            app_id: Optional application ID for multi-app setups
            
        Returns:
            Verification response from Sumsub
            
        Raises:
            SumsubAPIError: If verification fails
        """
        endpoint = "/resources/ai/agents/verifyBinding"
        
        payload = {
            "agentDid": agent_did,
            "humanDid": human_did,
            "timestamp": datetime.utcnow().isoformat() + "Z"
        }
        
        if app_id:
            payload["appId"] = app_id
        
        # Make API call with retries
        response = await self._make_request("POST", endpoint, payload)
        
        if response.get("verified"):
            logger.info(
                "Agent binding verified successfully",
                agent_did=agent_did,
                human_did=human_did,
                verification_level=response.get("verificationLevel"),
                risk_score=response.get("riskScore")
            )
            return response
        else:
            raise SumsubAPIError(
                f"Agent binding verification failed: {response.get('reason', 'Unknown')}",
                response=response
            )
    
    async def trigger_liveness_test(self, 
                                   human_id: str,
                                   reason: str,
                                   priority: str = "NORMAL") -> Dict[str, Any]:
        """
        Trigger targeted liveness test for human principal.
        
        This makes a real API call to trigger Sumsub's liveness verification.
        
        Args:
            human_id: Human principal ID from Sumsub
            reason: Reason for liveness test
            priority: Priority level (NORMAL, HIGH, URGENT)
            
        Returns:
            Liveness test session details
            
        Raises:
            SumsubAPIError: If liveness test cannot be triggered
        """
        endpoint = "/resources/ai/agents/triggerLiveness"
        
        payload = {
            "humanId": human_id,
            "reason": reason,
            "priority": priority,
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "callbackUrl": os.getenv("SUMSUB_CALLBACK_URL", "https://api.traderx.com/kya/callback")
        }
        
        response = await self._make_request("POST", endpoint, payload)
        
        logger.info(
            "Liveness test triggered",
            human_id=human_id,
            reason=reason,
            session_id=response.get("sessionId"),
            expires_at=response.get("expiresAt")
        )
        
        return response
    
    async def get_agent_verification_status(self, 
                                           agent_id: str) -> Dict[str, Any]:
        """
        Get current verification status for an agent.
        
        Args:
            agent_id: Agent ID from Sumsub
            
        Returns:
            Current verification status
        """
        endpoint = f"/resources/ai/agents/{agent_id}/status"
        
        response = await self._make_request("GET", endpoint)
        
        return response
    
    async def get_liveness_result(self, 
                                 session_id: str) -> Dict[str, Any]:
        """
        Get liveness test result.
        
        Args:
            session_id: Liveness test session ID
            
        Returns:
            Liveness test result
        """
        endpoint = f"/resources/ai/agents/liveness/{session_id}"
        
        response = await self._make_request("GET", endpoint)
        
        return response
    
    async def create_agent_verification(self,
                                       agent_id: str,
                                       agent_type: str,
                                       human_id: str,
                                       metadata: Optional[Dict] = None) -> Dict[str, Any]:
        """
        Create a new agent verification request.
        
        Args:
            agent_id: Unique agent identifier
            agent_type: Type of agent (TRADING, ANALYSIS, RISK_MANAGEMENT)
            human_id: Human principal ID
            metadata: Additional metadata
            
        Returns:
            Created verification request details
        """
        endpoint = "/resources/ai/agents"
        
        payload = {
            "agentId": agent_id,
            "agentType": agent_type,
            "humanId": human_id,
            "metadata": metadata or {},
            "timestamp": datetime.utcnow().isoformat() + "Z"
        }
        
        response = await self._make_request("POST", endpoint, payload)
        
        logger.info(
            "Agent verification created",
            agent_id=agent_id,
            agent_type=agent_type,
            verification_id=response.get("verificationId")
        )
        
        return response
    
    async def _make_request(self, 
                           method: str,
                           endpoint: str,
                           payload: Optional[Dict] = None,
                           retry_count: int = 0) -> Dict[str, Any]:
        """
        Make HTTP request to Sumsub API with retry logic.
        
        Args:
            method: HTTP method
            endpoint: API endpoint
            payload: Request payload
            retry_count: Current retry attempt
            
        Returns:
            Parsed JSON response
            
        Raises:
            SumsubAPIError: If request fails after retries
        """
        url = f"{self.config.base_url}{endpoint}"
        
        # Prepare headers
        headers = {
            "Content-Type": "application/json",
            "Accept": "application/json"
        }
        
        # Add auth token
        if self.config.api_token:
            headers["X-App-Token"] = self.config.api_token
        
        # Sign request if payload provided
        if payload:
            signature = self._sign_request(payload)
            headers["X-Signature"] = signature
        
        # Make request
        try:
            if method.upper() == "GET":
                response = await self.client.get(url, headers=headers)
            elif method.upper() == "POST":
                response = await self.client.post(url, json=payload, headers=headers)
            elif method.upper() == "PUT":
                response = await self.client.put(url, json=payload, headers=headers)
            else:
                raise ValueError(f"Unsupported HTTP method: {method}")
            
            # Handle response
            if response.status_code == 200:
                return response.json()
            elif response.status_code == 401:
                raise SumsubAPIError(
                    "Authentication failed: Invalid API token or signature",
                    status_code=response.status_code
                )
            elif response.status_code == 429:
                # Rate limited - wait and retry
                if retry_count < self.config.max_retries:
                    wait_time = self.config.retry_delay * (2 ** retry_count)
                    logger.warning(
                        "Rate limited, retrying in {wait_time}s",
                        retry_count=retry_count + 1
                    )
                    await asyncio.sleep(wait_time)
                    return await self._make_request(method, endpoint, payload, retry_count + 1)
                else:
                    raise SumsubAPIError(
                        "Rate limit exceeded, max retries reached",
                        status_code=response.status_code
                    )
            else:
                # Other error - check if retryable
                if self._is_retryable_error(response.status_code) and retry_count < self.config.max_retries:
                    wait_time = self.config.retry_delay * (2 ** retry_count)
                    logger.warning(
                        "Retryable error, retrying in {wait_time}s",
                        status_code=response.status_code,
                        retry_count=retry_count + 1
                    )
                    await asyncio.sleep(wait_time)
                    return await self._make_request(method, endpoint, payload, retry_count + 1)
                else:
                    error_msg = f"API request failed: {response.status_code}"
                    try:
                        error_detail = response.json().get("error", {})
                        error_msg += f" - {error_detail}"
                    except:
                        error_msg += f" - {response.text}"
                    
                    raise SumsubAPIError(
                        error_msg,
                        status_code=response.status_code,
                        response=response.json() if response.headers.get("content-type", "").startswith("application/json") else None
                    )
        
        except httpx.RequestError as e:
            if retry_count < self.config.max_retries:
                wait_time = self.config.retry_delay * (2 ** retry_count)
                logger.warning(
                    "Network error, retrying in {wait_time}s",
                    error=str(e),
                    retry_count=retry_count + 1
                )
                await asyncio.sleep(wait_time)
                return await self._make_request(method, endpoint, payload, retry_count + 1)
            else:
                raise SumsubAPIError(f"Network error after retries: {str(e)}")
    
    def _sign_request(self, payload: Dict[str, Any]) -> str:
        """
        Sign request with HMAC-SHA256 using API secret.
        
        Args:
            payload: Request payload to sign
            
        Returns:
            Hex-encoded signature
        """
        # Create canonical representation
        payload_str = json.dumps(payload, sort_keys=True, separators=(',', ':'))
        
        # Generate HMAC-SHA256 signature
        signature = hmac.new(
            self.config.api_secret.encode(),
            payload_str.encode(),
            hashlib.sha256
        ).hexdigest()
        
        return signature
    
    def _is_retryable_error(self, status_code: int) -> bool:
        """
        Check if an HTTP status code indicates a retryable error.
        
        Args:
            status_code: HTTP status code
            
        Returns:
            True if error is retryable
        """
        # Retry on server errors and some client errors
        retryable_codes = {408, 429, 500, 502, 503, 504}
        return status_code in retryable_codes


# Factory function for creating Sumsub client
async def create_sumsub_client() -> SumsubClient:
    """
    Create Sumsub client from environment variables.
    
    Returns:
        Configured SumsubClient instance
        
    Raises:
        ValueError: If required environment variables are missing
    """
    api_token = os.getenv("SUMSUB_API_TOKEN")
    api_secret = os.getenv("SUMSUB_API_SECRET")
    base_url = os.getenv("SUMSUB_BASE_URL", "https://api.sumsub.com")
    
    if not api_token:
        raise ValueError("SUMSUB_API_TOKEN environment variable is required")
    if not api_secret:
        raise ValueError("SUMSUB_API_SECRET environment variable is required")
    
    config = SumsubConfig(
        api_token=api_token,
        api_secret=api_secret,
        base_url=base_url,
        timeout=int(os.getenv("SUMSUB_TIMEOUT", "30")),
        max_retries=int(os.getenv("SUMSUB_MAX_RETRIES", "3")),
        retry_delay=float(os.getenv("SUMSUB_RETRY_DELAY", "1.0"))
    )
    
    return SumsubClient(config)
