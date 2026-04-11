"""
Feature pipeline — reads from QuestDB, computes features, writes to Redis
online store and Parquet offline store. Runs as a continuous streaming process.
"""

import asyncio
import os
import time
from datetime import datetime, timezone
from typing import Optional

import numpy as np
import pandas as pd
import pyarrow as pa
import pyarrow.parquet as pq
import asyncpg
import structlog

from .compute import compute_all
from .online import OnlineFeatureStore
from .drift import DriftDetector

logger = structlog.get_logger(__name__)

QUESTDB_PG_DSN = os.getenv(
    "QUESTDB_PG_DSN",
    "postgresql://admin:quest@127.0.0.1:8812/qdb",
)
PARQUET_ROOT = os.getenv("FEATURE_PARQUET_ROOT", "/var/traderx/features")
SYMBOLS = os.getenv("FEATURE_SYMBOLS", "BTC-USD,ETH-USD,SPY,QQQ").split(",")
POLL_INTERVAL_S = float(os.getenv("FEATURE_POLL_INTERVAL_S", "1.0"))
LOOKBACK_BARS = int(os.getenv("FEATURE_LOOKBACK_BARS", "500"))


class FeaturePipeline:
    """
    Continuous feature computation pipeline.

    Architecture:
      QuestDB (ohlcv_1m) → compute_all() → OnlineFeatureStore (Redis)
                                          → Parquet (offline/backtest)
                                          → DriftDetector → alerts
    """

    def __init__(
        self,
        symbols: list[str] = SYMBOLS,
        poll_interval: float = POLL_INTERVAL_S,
        lookback: int = LOOKBACK_BARS,
    ):
        self._symbols = symbols
        self._poll_interval = poll_interval
        self._lookback = lookback
        self._online_store = OnlineFeatureStore()
        self._drift = DriftDetector()
        self._pg: Optional[asyncpg.Pool] = None
        self._parquet_writers: dict = {}

    async def start(self) -> None:
        await self._online_store.connect()
        self._pg = await asyncpg.create_pool(QUESTDB_PG_DSN, min_size=2, max_size=8)
        logger.info("Feature pipeline started", symbols=self._symbols)
        await self._run_loop()

    async def _run_loop(self) -> None:
        while True:
            t0 = time.monotonic()
            tasks = [self._process_symbol(sym) for sym in self._symbols]
            await asyncio.gather(*tasks, return_exceptions=True)
            elapsed = time.monotonic() - t0
            sleep = max(0.0, self._poll_interval - elapsed)
            await asyncio.sleep(sleep)

    async def _process_symbol(self, symbol: str) -> None:
        try:
            rows = await self._fetch_bars(symbol)
            if len(rows) < 30:
                return

            close  = np.array([r["close"]    for r in rows], dtype=np.float64)
            volume = np.array([r["volume"]   for r in rows], dtype=np.float64)
            bid    = np.array([r["open"]     for r in rows], dtype=np.float64)  # placeholder
            ask    = np.array([r["close"]    for r in rows], dtype=np.float64)  # placeholder
            bid_sz = np.full(len(rows), 1.0, dtype=np.float64)
            ask_sz = np.full(len(rows), 1.0, dtype=np.float64)

            feats = compute_all(close, volume, bid, ask, bid_sz, ask_sz)

            # Latest scalar values for online store
            latest = {k: float(v[-1]) if not np.isnan(v[-1]) else 0.0 for k, v in feats.items()}
            latest["ts"] = rows[-1]["ts"].timestamp()
            await self._online_store.write(symbol, latest)

            # Drift detection on RSI
            rsi_series = feats["rsi_14"]
            await self._drift.update(symbol, "rsi_14", rsi_series[~np.isnan(rsi_series)])

            # Write Parquet snapshot (async, non-blocking)
            asyncio.ensure_future(self._write_parquet(symbol, rows, feats))

        except Exception as e:
            logger.error("feature pipeline error", symbol=symbol, error=str(e))

    async def _fetch_bars(self, symbol: str) -> list:
        async with self._pg.acquire() as conn:
            rows = await conn.fetch(
                """
                SELECT ts, open, high, low, close, volume, vwap
                FROM ohlcv_1m
                WHERE symbol = $1
                ORDER BY ts DESC
                LIMIT $2
                """,
                symbol,
                self._lookback,
            )
        return list(reversed(rows))

    async def _write_parquet(self, symbol: str, rows: list, feats: dict) -> None:
        try:
            today = datetime.now(timezone.utc).strftime("%Y%m%d")
            path = f"{PARQUET_ROOT}/{symbol}/{today}.parquet"
            os.makedirs(os.path.dirname(path), exist_ok=True)

            df = pd.DataFrame({
                "ts":     [r["ts"] for r in rows],
                "close":  [r["close"] for r in rows],
                "volume": [r["volume"] for r in rows],
                **{k: v for k, v in feats.items()},
            })
            table = pa.Table.from_pandas(df)
            pq.write_table(
                table, path,
                compression="zstd",
                compression_level=9,
                row_group_size=100_000,
            )
        except Exception as e:
            logger.warning("parquet write failed", symbol=symbol, error=str(e))


async def main() -> None:
    pipeline = FeaturePipeline()
    await pipeline.start()


if __name__ == "__main__":
    asyncio.run(main())
