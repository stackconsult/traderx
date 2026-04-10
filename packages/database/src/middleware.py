"""
Tenant Context Middleware
Validates and enforces tenant context for all database operations.
"""

import logging
from typing import Optional, Callable, Awaitable
from fastapi import Request, Response, HTTPException
from starlette.middleware.base import BaseHTTPMiddleware
import asyncio
import uuid

logger = logging.getLogger(__name__)


class TenantContextMiddleware(BaseHTTPMiddleware):
    """
    Middleware to extract and validate tenant context from requests.
    Ensures tenant_id is present and valid before any database operation.
    """
    
    def __init__(self, app, tenant_manager, jwt_secret: str):
        super().__init__(app)
        self.tenant_manager = tenant_manager
        self.jwt_secret = jwt_secret
        
    async def dispatch(self, request: Request, call_next: Callable) -> Response:
        """Extract tenant context and validate."""
        # Extract tenant from JWT token or subdomain
        tenant_id = await self._extract_tenant(request)
        
        if not tenant_id:
            raise HTTPException(
                status_code=401,
                detail="Tenant context required"
            )
            
        # Validate tenant exists and is active
        tenant = await self.tenant_manager.get_tenant(tenant_id)
        if not tenant:
            raise HTTPException(
                status_code=404,
                detail="Tenant not found"
            )
            
        if not tenant.is_active:
            raise HTTPException(
                status_code=403,
                detail="Tenant is not active"
            )
            
        # Add tenant to request state
        request.state.tenant_id = str(tenant.id)
        request.state.tenant = tenant
        
        # Continue with request
        response = await call_next(request)
        
        return response
        
    async def _extract_tenant(self, request: Request) -> Optional[str]:
        """Extract tenant ID from JWT or subdomain."""
        # Try JWT token first
        authorization = request.headers.get("authorization")
        if authorization and authorization.startswith("Bearer "):
            token = authorization.split(" ")[1]
            try:
                # Decode JWT (simplified - use proper JWT library in production)
                import jwt
                payload = jwt.decode(token, self.jwt_secret, algorithms=["HS256"])
                return payload.get("tenant_id")
            except Exception as e:
                logger.warning(f"JWT decode failed: {e}")
                
        # Try subdomain
        host = request.headers.get("host", "")
        parts = host.split(".")
        if len(parts) > 2 and parts[0] != "www" and parts[0] != "api":
            # subdomain.tenantx.com -> subdomain is tenant slug
            return parts[0]
            
        return None


async def require_tenant_context(request: Request) -> str:
    """
    Dependency to ensure tenant context is present.
    Use in FastAPI endpoints:
    
    @app.get("/api/entities")
    async def get_entities(request: Request, tenant_id: str = Depends(require_tenant_context)):
        ...
    """
    if not hasattr(request.state, "tenant_id"):
        raise HTTPException(
            status_code=401,
            detail="Tenant context not set"
        )
    return request.state.tenant_id


class TenantAwareDatabaseOperation:
    """
    Wrapper for database operations that ensures tenant context.
    """
    
    def __init__(self, tenant_manager):
        self.tenant_manager = tenant_manager
        
    async def execute_with_tenant(self, 
                                 tenant_id: str,
                                 operation: Callable,
                                 *args,
                                 **kwargs):
        """Execute database operation with tenant context."""
        # Validate tenant exists
        tenant = await self.tenant_manager.get_tenant(tenant_id)
        if not tenant:
            raise ValueError(f"Tenant not found: {tenant_id}")
            
        # Execute with tenant context
        async with self.tenant_manager.with_tenant(tenant_id) as conn:
            return await operation(conn, *args, **kwargs)
