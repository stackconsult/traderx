"""
PSI-based feature drift detection with Redis pub/sub alerting.
Population Stability Index (PSI) > 0.2 = significant drift → retrain trigger.
"""

import os
from collections import deque
from typing import Deque, Dict, Optional

import numpy as np
import redis.asyncio as aioredis
import msgpack
import structlog

logger = structlog.get_logger(__name__)

PSI_WARN  = 0.10  # yellow alert
PSI_ALERT = 0.20  # red alert — trigger model retraining


def _psi(expected: np.ndarray, actual: np.ndarray, buckets: int = 10) -> float:
    """Compute Population Stability Index between two distributions."""
    breaks = np.percentile(expected, np.linspace(0, 100, buckets + 1))
    breaks[0]  = -np.inf
    breaks[-1] =  np.inf

    exp_cnt = np.histogram(expected, bins=breaks)[0].astype(float)
    act_cnt = np.histogram(actual,   bins=breaks)[0].astype(float)

    # Smooth zeros
    exp_pct = np.where(exp_cnt == 0, 1e-4, exp_cnt / exp_cnt.sum())
    act_pct = np.where(act_cnt == 0, 1e-4, act_cnt / act_cnt.sum())

    return float(np.sum((act_pct - exp_pct) * np.log(act_pct / exp_pct)))


class DriftDetector:
    """
    Maintains a reference window and a recent window per (symbol, feature).
    On each update, computes PSI and publishes to Redis channel 'feature_drift'
    when PSI exceeds threshold.
    """

    def __init__(
        self,
        reference_window: int = 1000,
        detection_window: int = 200,
        redis_url: Optional[str] = None,
    ):
        self._ref_win    = reference_window
        self._det_win    = detection_window
        self._redis_url  = redis_url or os.getenv("REDIS_URL", "redis://127.0.0.1:6379/0")
        self._redis: Optional[aioredis.Redis] = None
        # (symbol, feature) -> deque of reference values
        self._reference: Dict[str, Deque[float]] = {}
        # (symbol, feature) -> deque of recent values
        self._recent:    Dict[str, Deque[float]] = {}
        self._scores:    Dict[str, float]         = {}

    async def _ensure_redis(self) -> None:
        if self._redis is None:
            self._redis = await aioredis.from_url(
                self._redis_url, decode_responses=False
            )

    async def update(self, symbol: str, feature: str, values: np.ndarray) -> float:
        """
        Update internal windows with new values.
        Returns current PSI score (0 if reference not yet filled).
        """
        key = f"{symbol}:{feature}"

        if key not in self._reference:
            self._reference[key] = deque(maxlen=self._ref_win)
            self._recent[key]    = deque(maxlen=self._det_win)

        ref    = self._reference[key]
        recent = self._recent[key]

        for v in values.tolist():
            if len(ref) < self._ref_win:
                ref.append(v)
            else:
                recent.append(v)

        if len(ref) < self._ref_win or len(recent) < self._det_win:
            return 0.0

        score = _psi(np.array(ref), np.array(recent))
        self._scores[key] = score

        if score >= PSI_WARN:
            await self._publish(symbol, feature, score)

        return score

    async def _publish(self, symbol: str, feature: str, score: float) -> None:
        await self._ensure_redis()
        level = "CRITICAL" if score >= PSI_ALERT else "WARNING"
        msg = msgpack.packb(
            {"symbol": symbol, "feature": feature, "psi": score, "level": level},
            use_bin_type=True,
        )
        await self._redis.publish("feature_drift", msg)
        logger.warning(
            "Feature drift detected",
            symbol=symbol, feature=feature,
            psi=round(score, 4), level=level,
        )

    def get_score(self, symbol: str, feature: str) -> float:
        return self._scores.get(f"{symbol}:{feature}", 0.0)

    def all_scores(self) -> Dict[str, float]:
        return dict(self._scores)
