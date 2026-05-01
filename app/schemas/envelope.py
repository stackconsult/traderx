from datetime import datetime
from pydantic import BaseModel, Field
from typing import Dict, Any


class AgentEnvelope(BaseModel):
    """Envelope schema for inter-agent messaging in TraderX system."""
    
    correlation_id: str = Field(..., description="Correlation ID for message tracking")
    request_id: str = Field(..., description="Unique request identifier")
    timestamp: datetime = Field(..., description="Message timestamp")
    from_agent: str = Field(..., description="Source agent identifier")
    to_agent: str = Field(..., description="Destination agent identifier")
    message_type: str = Field(..., description="Message type identifier")
    bam_signal: str = Field(..., description="BAM signal for fabric routing")
    pulse_trace: str = Field(..., description="Pulse trace for audit trail")
    dual_key: str = Field(..., description="Dual key: signal_hex::bam_raw")
    payload: Dict[str, Any] = Field(..., description="Message payload")
    
    # Note: Field mapping from LexCore legal domains to TraderX financial domains
    # - jurisdiction → market
    # - body_of_law → asset_class
    
    class Config:
        json_schema_extra = {
            "example": {
                "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
                "request_id": "req_001",
                "timestamp": "2026-05-01T09:00:00Z",
                "from_agent": "SignalMiner",
                "to_agent": "FabricRouter",
                "message_type": "SIGNAL_SUBMIT",
                "bam_signal": "00000.00000001.0001",
                "pulse_trace": "trace_001",
                "dual_key": "A1B2C3::EQUITY_ENTRY",
                "payload": {"symbol": "AAPL", "direction": "long", "conviction": 0.7}
            }
        }
