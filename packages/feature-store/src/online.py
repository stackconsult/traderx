"""
Online feature store — Redis-backed, <100μs serving latency.
Serializes feature vectors as msgpack bytes for minimal overhead.
"""

import asyncio
import os
import time
from typing import Any, Dict, List, Optional

import msgpack
import numpy as np
import redis.asyncio as aioredis
import redis as syncredis


class OnlineFeatureStore:
    """
    Async Redis-backed online feature store.
    Keys: f"features:{symbol}:{feature_name}" → msgpack-encoded scalar or array.
    Bulk key: f"features:{symbol}:__all__" → msgpack dict of all features.
    """

    BULK_TTL_SECONDS = 300  # 5 minutes

    def __init__(
        self,
        redis_url: Optional[str] = None,
        ttl_seconds: int = BULK_TTL_SECONDS,
    ):
        self._url = redis_url or os.getenv("REDIS_URL", "redis://127.0.0.1:6379/0")
        self._ttl = ttl_seconds
        self._client: Optional[aioredis.Redis] = None

    async def connect(self) -> None:
        self._client = await aioredis.from_url(
            self._url,
            encoding="utf-8",
            decode_responses=False,
            max_connections=64,
        )
        await self._client.ping()

    async def close(self) -> None:
        if self._client:
            await self._client.aclose()

    # ------------------------------------------------------------------
    # Write
    # ------------------------------------------------------------------
    async def write(self, symbol: str, features: Dict[str, Any]) -> None:
        """Write full feature dict for a symbol. O(1) pipeline round-trip."""
        assert self._client, "call connect() first"
        bulk_key = f"features:{symbol}:__all__"
        payload = msgpack.packb(
            {k: _encode_value(v) for k, v in features.items()},
            use_bin_type=True,
        )
        async with self._client.pipeline(transaction=False) as pipe:
            pipe.set(bulk_key, payload, ex=self._ttl)
            for name, val in features.items():
                per_key = f"features:{symbol}:{name}"
                pipe.set(per_key, msgpack.packb(_encode_value(val), use_bin_type=True), ex=self._ttl)
            await pipe.execute()

    # ------------------------------------------------------------------
    # Read — bulk (fastest path, single GET)
    # ------------------------------------------------------------------
    async def get_all(self, symbol: str) -> Optional[Dict[str, Any]]:
        assert self._client
        bulk_key = f"features:{symbol}:__all__"
        raw = await self._client.get(bulk_key)
        if raw is None:
            return None
        return {k: _decode_value(v) for k, v in msgpack.unpackb(raw, raw=False).items()}

    # ------------------------------------------------------------------
    # Read — selective (MGET, one round-trip)
    # ------------------------------------------------------------------
    async def get(self, symbol: str, names: List[str]) -> Dict[str, Optional[Any]]:
        assert self._client
        keys = [f"features:{symbol}:{n}" for n in names]
        values = await self._client.mget(keys)
        return {
            n: (_decode_value(msgpack.unpackb(v, raw=False)) if v is not None else None)
            for n, v in zip(names, values)
        }

    # ------------------------------------------------------------------
    # Drift signal: publish stale-feature alert
    # ------------------------------------------------------------------
    async def publish_drift(self, symbol: str, feature: str, score: float) -> None:
        assert self._client
        msg = msgpack.packb({"symbol": symbol, "feature": feature, "score": score}, use_bin_type=True)
        await self._client.publish("feature_drift", msg)


# ---------------------------------------------------------------------------
# Synchronous thin wrapper (for Rust FFI / gRPC server thread)
# ---------------------------------------------------------------------------
class SyncOnlineFeatureStore:
    def __init__(self, redis_url: Optional[str] = None, ttl_seconds: int = 300):
        url = redis_url or os.getenv("REDIS_URL", "redis://127.0.0.1:6379/0")
        self._client = syncredis.from_url(url, decode_responses=False)
        self._ttl = ttl_seconds

    def write(self, symbol: str, features: Dict[str, Any]) -> None:
        bulk_key = f"features:{symbol}:__all__"
        payload = msgpack.packb(
            {k: _encode_value(v) for k, v in features.items()},
            use_bin_type=True,
        )
        pipe = self._client.pipeline(transaction=False)
        pipe.set(bulk_key, payload, ex=self._ttl)
        pipe.execute()

    def get_all(self, symbol: str) -> Optional[Dict[str, Any]]:
        raw = self._client.get(f"features:{symbol}:__all__")
        if raw is None:
            return None
        return {k: _decode_value(v) for k, v in msgpack.unpackb(raw, raw=False).items()}

    def get(self, symbol: str, names: List[str]) -> Dict[str, Optional[Any]]:
        keys = [f"features:{symbol}:{n}" for n in names]
        values = self._client.mget(keys)
        return {
            n: (_decode_value(msgpack.unpackb(v, raw=False)) if v is not None else None)
            for n, v in zip(names, values)
        }


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
def _encode_value(v: Any) -> Any:
    if isinstance(v, np.ndarray):
        return {"__ndarray__": True, "dtype": str(v.dtype), "data": v.tobytes(), "shape": list(v.shape)}
    if isinstance(v, (np.floating, np.integer)):
        return float(v)
    return v


def _decode_value(v: Any) -> Any:
    if isinstance(v, dict) and v.get("__ndarray__"):
        arr = np.frombuffer(v["data"], dtype=np.dtype(v["dtype"]))
        return arr.reshape(v["shape"])
    return v
