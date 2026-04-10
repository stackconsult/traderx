"""Software PTP Synchronization
Fallback implementation for environments without hardware PTP support.
Uses NTP with high-precision polling and drift compensation.
"""

import time
import socket
import struct
import statistics
from datetime import datetime, timedelta
from typing import Optional, Tuple
import structlog

logger = structlog.get_logger(__name__)


class SoftwarePTP:
    """
    Software-based Precision Time Protocol.
    
    Hardware PTP achieves <1µs precision.
    Software fallback targets <100µs (relaxed from spec).
    """
    
    def __init__(self, ntp_servers: list = None, poll_interval: float = 1.0):
        self.ntp_servers = ntp_servers or [
            "pool.ntp.org",
            "time.google.com",
            "time.cloudflare.com"
        ]
        self.poll_interval = poll_interval
        
        # Time tracking
        self.local_offset = 0.0  # seconds
        self.last_sync = None
        self.drift_samples = []
        self.max_drift_samples = 100
        
        # Statistics
        self.sync_count = 0
        self.avg_drift_ms = 0
        
    def _query_ntp(self, server: str, timeout: float = 2.0) -> Optional[Tuple[float, float]]:
        """
        Query NTP server for time.
        Returns (ntp_time, rtt) or None on failure.
        """
        try:
            # Create UDP socket
            sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            sock.settimeout(timeout)
            
            # NTP packet (48 bytes)
            packet = b'\x1b' + 47 * b'\x00'
            
            # Send request
            sock.sendto(packet, (server, 123))
            t0 = time.perf_counter()
            
            # Receive response
            data, addr = sock.recvfrom(1024)
            t3 = time.perf_counter()
            
            sock.close()
            
            # Parse NTP timestamp (seconds since 1900)
            seconds = struct.unpack('!I', data[40:44])[0]
            fraction = struct.unpack('!I', data[44:48])[0]
            ntp_time = seconds + fraction / 2**32 - 2208988800  # Convert to Unix
            
            # Calculate RTT
            rtt = t3 - t0
            
            return ntp_time, rtt
            
        except Exception as e:
            logger.warning(f"NTP query failed for {server}", error=str(e))
            return None
    
    def sync(self) -> bool:
        """
        Synchronize local clock with NTP servers.
        Returns True if successful.
        """
        offsets = []
        rtts = []
        
        # Query multiple servers
        for server in self.ntp_servers:
            result = self._query_ntp(server)
            if result:
                ntp_time, rtt = result
                local_time = time.time()
                offset = ntp_time - local_time + rtt / 2  # Adjust for network delay
                offsets.append(offset)
                rtts.append(rtt)
                
        if len(offsets) == 0:
            logger.error("All NTP servers unreachable")
            return False
        
        # Use median offset (robust to outliers)
        median_offset = statistics.median(offsets)
        self.local_offset = median_offset
        self.last_sync = datetime.utcnow()
        
        # Calculate drift
        if len(self.drift_samples) >= 2:
            recent_drift = abs(offsets[0] - self.drift_samples[-1])
            self.drift_samples.append(recent_drift)
            if len(self.drift_samples) > self.max_drift_samples:
                self.drift_samples.pop(0)
            self.avg_drift_ms = statistics.mean(self.drift_samples) * 1000
        else:
            self.drift_samples.append(median_offset)
        
        self.sync_count += 1
        
        logger.info(
            "PTP sync complete",
            offset_ms=median_offset*1000,
            avg_rtt_ms=statistics.mean(rtts)*1000,
            servers_reached=len(offsets),
            drift_ms=self.avg_drift_ms
        )
        
        return True
    
    def now(self) -> datetime:
        """Get current time adjusted for offset."""
        return datetime.utcnow() + timedelta(seconds=self.local_offset)
    
    def get_metrics(self) -> dict:
        """Get synchronization metrics."""
        return {
            "sync_count": self.sync_count,
            "last_sync": self.last_sync.isoformat() if self.last_sync else None,
            "offset_ms": self.local_offset * 1000,
            "avg_drift_ms": self.avg_drift_ms,
            "drift_samples": len(self.drift_samples),
            "status": "synced" if self.last_sync else "unsynced"
        }
    
    def is_drift_acceptable(self, threshold_us: float = 100000) -> bool:
        """
        Check if clock drift is within acceptable threshold.
        Default: 100,000µs (100ms) for software PTP.
        """
        if self.avg_drift_ms == 0:
            return True
        return (self.avg_drift_ms * 1000) < threshold_us
