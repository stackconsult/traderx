"""
OpenTelemetry configuration for TraderX API Gateway
Provides distributed tracing and metrics collection.
"""

import os
import sys
import time
from typing import Dict, Any

from opentelemetry import trace, metrics
from opentelemetry.exporter.jaeger.thrift import JaegerExporter
from opentelemetry.exporter.prometheus import PrometheusMetricReader
from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor
from opentelemetry.instrumentation.asyncpg import AsyncPGInstrumentor
from opentelemetry.sdk.resources import SERVICE_NAME, Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader
import structlog

logger = structlog.get_logger(__name__)


def setup_telemetry(app=None):
    """Initialize OpenTelemetry for tracing and metrics."""
    
    # Service information
    service_name = os.getenv("SERVICE_NAME", "traderx-api-gateway")
    service_version = os.getenv("SERVICE_VERSION", "1.0.0")
    
    # Create resource with service metadata
    resource = Resource.create({
        SERVICE_NAME: service_name,
        "service.version": service_version,
        "environment": os.getenv("ENVIRONMENT", "development")
    })
    
    # Setup tracing
    setup_tracing(resource)
    
    # Setup metrics
    setup_metrics(resource)
    
    # Instrument FastAPI if app provided
    if app:
        FastAPIInstrumentor.instrument_app(app)
    
    # Instrument database
    AsyncPGInstrumentor().instrument()
    
    logger.info("OpenTelemetry initialized", service=service_name)


def setup_tracing(resource: Resource):
    """Setup distributed tracing with Jaeger."""
    
    # Configure Jaeger exporter
    jaeger_host = os.getenv("JAEGER_HOST", "localhost")
    jaeger_port = int(os.getenv("JAEGER_PORT", "14268"))
    
    jaeger_endpoint = f"http://{jaeger_host}:{jaeger_port}/api/traces"
    
    jaeger_exporter = JaegerExporter(
        endpoint=jaeger_endpoint,
        collector_endpoint=jaeger_endpoint,
    )
    
    # Create trace provider
    trace_provider = TracerProvider(resource=resource)
    
    # Add batch span processor
    span_processor = BatchSpanProcessor(jaeger_exporter)
    trace_provider.add_span_processor(span_processor)
    
    # Set global trace provider
    trace.set_tracer_provider(trace_provider)
    
    logger.info("Tracing initialized", jaeger_endpoint=jaeger_endpoint)


def setup_metrics(resource: Resource):
    """Setup metrics collection with Prometheus."""
    
    # Create Prometheus metric reader
    prometheus_reader = PrometheusMetricReader()
    
    # Create metric provider
    metric_reader = PeriodicExportingMetricReader(
        exporter=prometheus_reader,
        export_interval_millis=15000,  # 15 seconds
    )
    
    meter_provider = MeterProvider(
        resource=resource,
        metric_readers=[metric_reader],
    )
    
    # Set global meter provider
    metrics.set_meter_provider(meter_provider)
    
    # Create custom metrics
    create_custom_metrics()
    
    logger.info("Metrics initialized")


def create_custom_metrics():
    """Create custom trading platform metrics."""
    meter = metrics.get_meter(__name__)
    
    # Order metrics
    order_counter = meter.create_counter(
        "orders_total",
        description="Total number of orders",
    )
    
    order_latency = meter.create_histogram(
        "order_latency_seconds",
        description="Order submission latency",
        unit="s",
    )
    
    # Adapter metrics
    adapter_requests = meter.create_counter(
        "adapter_requests_total",
        description="Total adapter requests",
    )
    
    adapter_success = meter.create_counter(
        "adapter_success_total",
        description="Successful adapter requests",
    )
    
    # Handoff metrics
    handoff_duration = meter.create_histogram(
        "handoff_duration_seconds",
        description="Handoff completion time",
        unit="s",
    )
    
    handoff_counter = meter.create_counter(
        "handoffs_total",
        description="Total handoffs initiated",
    )
    
    # Database metrics
    db_query_duration = meter.create_histogram(
        "db_query_duration_seconds",
        description="Database query duration",
        unit="s",
    )
    
    # RLS metrics
    rls_violations = meter.create_counter(
        "rls_violations_total",
        description="Row Level Security violations",
    )
    
    # WebSocket metrics
    websocket_connections = meter.create_up_down_counter(
        "websocket_connections",
        description="Active WebSocket connections",
    )
    
    # Store metrics for later use
    metrics_registry = {
        "order_counter": order_counter,
        "order_latency": order_latency,
        "adapter_requests": adapter_requests,
        "adapter_success": adapter_success,
        "handoff_duration": handoff_duration,
        "handoff_counter": handoff_counter,
        "db_query_duration": db_query_duration,
        "rls_violations": rls_violations,
        "websocket_connections": websocket_connections,
    }
    
    # Store globally for access
    import sys
    sys.modules[__name__].metrics_registry = metrics_registry
    
    logger.info("Custom metrics created", count=len(metrics_registry))


def get_metrics():
    """Get the metrics registry."""
    return getattr(sys.modules[__name__], 'metrics_registry', {})


def record_order_submit(symbol: str, venue: str, latency_ms: float):
    """Record order submission metrics."""
    metrics = get_metrics()
    if "order_counter" in metrics:
        metrics["order_counter"].add(1, {"symbol": symbol, "venue": venue})
    if "order_latency" in metrics:
        metrics["order_latency"].record(latency_ms / 1000, {"symbol": symbol, "venue": venue})


def record_adapter_request(venue: str, success: bool):
    """Record adapter request metrics."""
    metrics = get_metrics()
    if "adapter_requests" in metrics:
        metrics["adapter_requests"].add(1, {"venue": venue})
    if "adapter_success" in metrics and success:
        metrics["adapter_success"].add(1, {"venue": venue})


def record_handoff(duration_ms: float, status: str):
    """Record handoff metrics."""
    metrics = get_metrics()
    if "handoff_counter" in metrics:
        metrics["handoff_counter"].add(1, {"status": status})
    if "handoff_duration" in metrics:
        metrics["handoff_duration"].record(duration_ms / 1000, {"status": status})


def record_rls_violation(tenant_id: str):
    """Record RLS violation."""
    metrics = get_metrics()
    if "rls_violations" in metrics:
        metrics["rls_violations"].add(1, {"tenant_id": tenant_id})
    
    # Also log the violation
    logger.error(
        "RLS violation detected",
        tenant_id=tenant_id,
        timestamp=time.time()
    )
