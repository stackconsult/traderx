"""HSTR State Tensor - O(1) lookup + O(k) Sharpe updates"""
import time, hashlib
from datetime import datetime
from typing import Dict, Optional
from dataclasses import dataclass
import numpy as np
import structlog
logger = structlog.get_logger(__name__)

@dataclass
class StateSnapshot:
    timestamp: datetime
    symbol: str
    price: float
    volume: float
    sharpe_ratio: float
    confidence: float
    delta_hash: str

class HSTRStateTensor:
    def __init__(self, retention_hours: int = 48):
        self._latest: Dict[str, StateSnapshot] = {}
        self._history: Dict[str, list] = {}
        self.retention = retention_hours
        self.query_ns = []
        self.updates = 0
    
    def _calc_sharpe(self, returns: list) -> float:
        if len(returns) < 2:
            return 0.0
        arr = np.array(returns)
        return np.mean(arr) / (np.std(arr) + 1e-10) * np.sqrt(252)
    
    def upsert(self, symbol: str, price: float, volume: float, returns: list) -> StateSnapshot:
        start = time.perf_counter_ns()
        sharpe = self._calc_sharpe(returns)
        conf = min(1.0, volume / 1e6)
        prev_hash = self._latest[symbol].delta_hash if symbol in self._latest else "genesis"
        snap = StateSnapshot(datetime.utcnow(), symbol, price, volume, sharpe, conf, prev_hash)
        self._latest[symbol] = snap
        self._history.setdefault(symbol, []).append(snap)
        self.updates += 1
        logger.debug("upsert", symbol=symbol, sharpe=sharpe, ns=time.perf_counter_ns()-start)
        return snap
    
    def query(self, symbol: str) -> Optional[StateSnapshot]:
        start = time.perf_counter_ns()
        result = self._latest.get(symbol)
        self.query_ns.append(time.perf_counter_ns() - start)
        return result
    
    def get_metrics(self) -> dict:
        return {
            "symbols": len(self._latest),
            "updates": self.updates,
            "avg_query_ns": np.mean(self.query_ns) if self.query_ns else 0,
            "p99_query_ns": np.percentile(self.query_ns, 99) if len(self.query_ns) > 100 else 0
        }
