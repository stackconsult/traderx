"""
Rate limiting middleware for multi-tenant protection.
Implements per-tenant rate limiting using Redis.
"""

import time
import json
from typing import Dict, Optional
from fastapi import Request, HTTPException
from starlette.middleware.base import BaseHTTPMiddleware
import redis.asyncio as redis
import structlog

logger = structlog.get_logger(__name__)


class RateLimiterMiddleware(BaseHTTPMiddleware):
    """Rate limiting middleware with per-tenant quotas."""
    
    def __init__(self, app, redis_url: str):
        super().__init__(app)
        self.redis_url = redis_url
        self.redis_client: Optional[redis.Redis] = None
        
        # Rate limits per tenant tier
        self.limits = {
            "free": {"requests": 100, "window": 60},      # 100 req/min
            "basic": {"requests": 1000, "window": 60},     # 1k req/min
            "pro": {"requests": 10000, "window": 60},      # 10k req/min
            "enterprise": {"requests": 100000, "window": 60}  # 100k req/min
        }
    
    async def dispatch(self, request: Request, call_next):
        """Process request with rate limiting."""
        # Initialize Redis client if needed
        if not self.redis_client:
            self.redis_client = redis.from_url(self.redis_url)
        
        # Extract tenant from JWT or header
        tenant_id = self._extract_tenant_id(request)
        tenant_tier = await self._get_tenant_tier(tenant_id)
        
        # Check rate limit
        if not await self._check_rate_limit(tenant_id, tenant_tier):
            logger.warning(
                "Rate limit exceeded",
                tenant_id=tenant_id,
                tier=tenant_tier,
                path=request.url.path
            )
            raise HTTPException(
                status_code=429,
                detail="Rate limit exceeded",
                headers={
                    "Retry-After": "60",
                    "X-RateLimit-Limit": str(self.limits[tenant_tier]["requests"]),
                    "X-RateLimit-Window": str(self.limits[tenant_tier]["window"])
                }
            )
        
        # Process request
        response = await call_next(request)
        
        # Add rate limit headers
        remaining = await self._get_remaining_requests(tenant_id, tenant_tier)
        response.headers["X-RateLimit-Limit"] = str(self.limits[tenant_tier]["requests"])
        response.headers["X-RateLimit-Remaining"] = str(remaining)
        response.headers["X-RateLimit-Window"] = str(self.limits[tenant_tier]["window"])
        
        return response
    
    def _extract_tenant_id(self, request: Request) -> str:
        """Extract tenant ID from request."""
        # Try JWT token first
        auth_header = request.headers.get("Authorization")
        if auth_header and auth_header.startswith("Bearer "):
            # TODO: Decode JWT and extract tenant_id
            # For now, use a default
            return "default-tenant"
        
        # Fallback to header
        tenant_id = request.headers.get("X-Tenant-ID")
        if tenant_id:
            return tenant_id
        
        # Default tenant
        return "anonymous"
    
    async def _get_tenant_tier(self, tenant_id: str) -> str:
        """Get tenant tier from database or cache."""
        # TODO: Implement actual tier lookup
        # For now, return "free" tier
        return "free"
    
    async def _check_rate_limit(self, tenant_id: str, tier: str) -> bool:
        """Check if tenant has exceeded rate limit."""
        limit = self.limits[tier]
        key = f"rate_limit:{tenant_id}"
        
        try:
            # Use Redis sliding window algorithm
            current_time = int(time.time())
            window_start = current_time - limit["window"]
            
            # Remove old entries
            await self.redis_client.zremrangebyscore(key, 0, window_start)
            
            # Count current requests
            current_requests = await self.redis_client.zcard(key)
            
            if current_requests >= limit["requests"]:
                return False
            
            # Add current request
            await self.redis_client.zadd(key, {str(current_time): current_time})
            await self.redis_client.expire(key, limit["window"])
            
            return True
            
        except Exception as e:
            logger.error(
                "Rate limit check failed",
                tenant_id=tenant_id,
                error=str(e)
            )
            # Fail open - allow request if Redis is down
            return True
    
    async def _get_remaining_requests(self, tenant_id: str, tier: str) -> int:
        """Get remaining requests for tenant."""
        limit = self.limits[tier]
        key = f"rate_limit:{tenant_id}"
        
        try:
            current_requests = await self.redis_client.zcard(key)
            return max(0, limit["requests"] - current_requests)
        except:
            return limit["requests"]


class CircuitBreakerMiddleware(BaseHTTPMiddleware):
    """Circuit breaker for external adapter calls."""
    
    def __init__(self, app):
        super().__init__(app)
        self.circuit_states = {}
        self.failure_threshold = 5
        self.recovery_timeout = 60  # seconds
    
    async def dispatch(self, request: Request, call_next):
        """Process request with circuit breaker."""
        # Only apply to adapter endpoints
        if not request.url.path.startswith("/api/v1/adapters"):
            return await call_next(request)
        
        venue = request.path_params.get("venue", "unknown")
        circuit_state = self._get_circuit_state(venue)
        
        if circuit_state["state"] == "open":
            if time.time() - circuit_state["last_failure"] > self.recovery_timeout:
                # Try to close circuit
                circuit_state["state"] = "half_open"
                logger.info(
                    "Circuit breaker half-open",
                    venue=venue
                )
            else:
                logger.warning(
                    "Circuit breaker open",
                    venue=venue,
                    time_until_recovery=self.recovery_timeout - (time.time() - circuit_state["last_failure"])
                )
                raise HTTPException(
                    status_code=503,
                    detail=f"Service temporarily unavailable for {venue}"
                )
        
        try:
            response = await call_next(request)
            
            # Reset on success
            if circuit_state["state"] == "half_open":
                circuit_state["state"] = "closed"
                circuit_state["failures"] = 0
                logger.info(
                    "Circuit breaker closed",
                    venue=venue
                )
            
            return response
            
        except Exception as e:
            # Record failure
            circuit_state["failures"] += 1
            circuit_state["last_failure"] = time.time()
            
            if circuit_state["failures"] >= self.failure_threshold:
                circuit_state["state"] = "open"
                logger.error(
                    "Circuit breaker opened",
                    venue=venue,
                    failures=circuit_state["failures"]
                )
            
            raise e
    
    def _get_circuit_state(self, venue: str) -> Dict:
        """Get or create circuit state for venue."""
        if venue not in self.circuit_states:
            self.circuit_states[venue] = {
                "state": "closed",  # closed, open, half_open
                "failures": 0,
                "last_failure": 0
            }
        return self.circuit_states[venue]
