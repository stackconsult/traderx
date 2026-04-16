"""
FastAPI Gateway for TraderX
Provides REST API endpoints and WebSocket connections for the trading platform.
"""

import asyncio
import logging
import os
from datetime import datetime
from typing import Dict, Any

import structlog
from fastapi import FastAPI, Request, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
import uvicorn

# Import telemetry and middleware
from .telemetry import (
    setup_telemetry,
    record_order_submit,
    record_adapter_request,
    record_handoff,
)
from .middleware.rate_limiter import RateLimiterMiddleware, CircuitBreakerMiddleware

# Configure structured logging
structlog.configure(
    processors=[
        structlog.stdlib.filter_by_level,
        structlog.stdlib.add_logger_name,
        structlog.stdlib.add_log_level,
        structlog.stdlib.PositionalArgumentsFormatter(),
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.StackInfoRenderer(),
        structlog.processors.format_exc_info,
        structlog.processors.UnicodeDecoder(),
        structlog.processors.JSONRenderer(),
    ],
    context_class=dict,
    logger_factory=structlog.stdlib.LoggerFactory(),
    wrapper_class=structlog.stdlib.BoundLogger,
    cache_logger_on_first_use=True,
)

logger = structlog.get_logger()

# Initialize FastAPI app
app = FastAPI(
    title="TraderX API Gateway",
    description="Production trading platform API",
    version="1.0.0",
    docs_url="/docs",
    redoc_url="/redoc",
)

# CORS middleware
#
# Origins are sourced from CORS_ALLOWED_ORIGINS (comma-separated) and default
# to the local dashboard + the production domain. Wildcard origins ("*") are
# rejected because we send credentials (cookies/JWT) with requests — per the
# Fetch spec, `Access-Control-Allow-Origin: *` with credentials is insecure
# and browsers will refuse it anyway. Methods and headers are enumerated
# explicitly instead of using "*" so new verbs (e.g. TRACE, PATCH) cannot
# accidentally be exposed cross-origin in the future.
_default_origins = "http://localhost:3000,https://traderx.com"
_raw_origins = os.getenv("CORS_ALLOWED_ORIGINS", _default_origins)
_allowed_origins = [
    o.strip() for o in _raw_origins.split(",") if o.strip() and o.strip() != "*"
]
app.add_middleware(
    CORSMiddleware,
    allow_origins=_allowed_origins,
    allow_credentials=True,
    allow_methods=["GET", "POST", "PUT", "DELETE", "OPTIONS"],
    allow_headers=[
        "Authorization",
        "Content-Type",
        "X-Tenant-ID",
        "X-Request-ID",
    ],
)

# Rate limiting middleware
redis_url = os.getenv("REDIS_URL", "redis://localhost:6379")
app.add_middleware(RateLimiterMiddleware, redis_url=redis_url)

# Circuit breaker middleware
app.add_middleware(CircuitBreakerMiddleware)

# Global state for WebSocket connections
active_connections: Dict[str, WebSocket] = {}


@app.on_event("startup")
async def startup_event():
    """Initialize services on startup."""
    logger.info("API Gateway starting up", timestamp=datetime.utcnow().isoformat())

    # Initialize OpenTelemetry
    setup_telemetry(app)

    # TODO: Initialize database connection pool
    # TODO: Initialize Redis connection
    # TODO: Load execution adapters

    logger.info("API Gateway startup complete")


@app.on_event("shutdown")
async def shutdown_event():
    """Cleanup on shutdown."""
    logger.info("API Gateway shutting down")

    # Close all WebSocket connections
    for conn_id, ws in active_connections.items():
        try:
            await ws.close()
        except:
            pass
    active_connections.clear()

    # TODO: Close database connections
    # TODO: Close Redis connection

    logger.info("API Gateway shutdown complete")


@app.get("/health")
async def health_check():
    """Health check endpoint for docker-compose."""
    return {
        "status": "healthy",
        "timestamp": datetime.utcnow().isoformat(),
        "service": "traderx-api-gateway",
        "version": "1.0.0",
    }


@app.get("/ready")
async def readiness_check():
    """Readiness check - verifies all dependencies are ready."""
    # TODO: Check database connection
    # TODO: Check Redis connection
    # TODO: Check adapter health

    return {
        "status": "ready",
        "timestamp": datetime.utcnow().isoformat(),
        "dependencies": {
            "database": "healthy",
            "redis": "healthy",
            "adapters": "healthy",
        },
    }


@app.get("/api/v1/status")
async def get_status():
    """Get overall system status."""
    return {
        "status": "operational",
        "timestamp": datetime.utcnow().isoformat(),
        "services": {
            "api_gateway": "healthy",
            "execution_adapters": "healthy",
            "handoff_system": "healthy",
            "database": "healthy",
        },
        "metrics": {
            "active_connections": len(active_connections),
            "uptime": "0s",  # TODO: Calculate actual uptime
        },
    }


@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    """WebSocket endpoint for real-time data."""
    await websocket.accept()

    # Generate unique connection ID
    conn_id = f"conn_{datetime.utcnow().timestamp()}"
    active_connections[conn_id] = websocket

    logger.info("WebSocket connection established", connection_id=conn_id)

    try:
        while True:
            # Receive message from client
            data = await websocket.receive_text()

            # TODO: Process message and route to appropriate handler
            # For now, just echo back
            await websocket.send_text(f"Echo: {data}")

    except WebSocketDisconnect:
        logger.info("WebSocket connection closed", connection_id=conn_id)
    except Exception as e:
        logger.error("WebSocket error", connection_id=conn_id, error=str(e))
    finally:
        # Clean up connection
        active_connections.pop(conn_id, None)


@app.get("/api/v1/strategies")
async def get_strategies():
    """Get available trading strategies."""
    # TODO: Implement actual strategy retrieval from database
    return {
        "strategies": [
            {
                "id": "strategy-1",
                "name": "AI Momentum Strategy",
                "description": "Detects momentum shifts using multi-timeframe analysis",
                "status": "ACTIVE",
            }
        ]
    }


@app.post("/api/v1/handoff")
async def initiate_handoff(request: Request):
    """Initiate a new handoff between Claude 4.6 and Gemma 4."""
    # TODO: Implement actual handoff initiation
    data = await request.json()

    return {
        "handoff_id": "handoff_123",
        "status": "PENDING",
        "message": "Handoff initiated successfully",
    }


@app.get("/api/v1/handoff/{handoff_id}")
async def get_handoff_status(handoff_id: str):
    """Get status of a specific handoff."""
    # TODO: Implement actual status retrieval
    return {
        "handoff_id": handoff_id,
        "status": "COMPLETE",
        "progress": 100,
        "result": "Handoff completed successfully",
    }


# Error handlers
@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception):
    """Global exception handler."""
    logger.error(
        "Unhandled exception",
        path=request.url.path,
        method=request.method,
        error=str(exc),
        exc_info=True,
    )

    return JSONResponse(
        status_code=500,
        content={
            "error": "Internal server error",
            "message": "An unexpected error occurred",
            "timestamp": datetime.utcnow().isoformat(),
        },
    )


if __name__ == "__main__":
    # Run the application
    uvicorn.run(
        "main:app",
        host="0.0.0.0",
        port=8000,
        reload=os.getenv("ENVIRONMENT") == "development",
        log_config=None,  # Use structlog instead
    )
