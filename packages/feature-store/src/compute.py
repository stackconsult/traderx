"""
Feature kernel library — Numba JIT compiled for <100μs online latency.
All kernels operate on numpy arrays for vectorized batch computation.
"""

import numpy as np
import numba as nb
from numba import njit, float64, int64

# ---------------------------------------------------------------------------
# Returns
# ---------------------------------------------------------------------------
@njit(float64[:](float64[:]), cache=True)
def returns(close: np.ndarray) -> np.ndarray:
    n = len(close)
    out = np.empty(n, dtype=nb.float64)
    out[0] = 0.0
    for i in range(1, n):
        if close[i - 1] != 0.0:
            out[i] = (close[i] - close[i - 1]) / close[i - 1]
        else:
            out[i] = 0.0
    return out


@njit(float64[:](float64[:]), cache=True)
def log_returns(close: np.ndarray) -> np.ndarray:
    n = len(close)
    out = np.empty(n, dtype=nb.float64)
    out[0] = 0.0
    for i in range(1, n):
        if close[i - 1] > 0.0 and close[i] > 0.0:
            out[i] = np.log(close[i] / close[i - 1])
        else:
            out[i] = 0.0
    return out


# ---------------------------------------------------------------------------
# Volatility
# ---------------------------------------------------------------------------
@njit(float64[:](float64[:], int64), cache=True)
def realized_vol(close: np.ndarray, window: int) -> np.ndarray:
    lr = log_returns(close)
    n = len(lr)
    out = np.empty(n, dtype=nb.float64)
    for i in range(n):
        if i < window:
            out[i] = np.nan
        else:
            sl = lr[i - window + 1 : i + 1]
            out[i] = np.std(sl) * np.sqrt(252.0 * 390.0)  # annualized 1-min
    return out


# ---------------------------------------------------------------------------
# RSI
# ---------------------------------------------------------------------------
@njit(float64[:](float64[:], int64), cache=True)
def rsi(close: np.ndarray, period: int) -> np.ndarray:
    n = len(close)
    out = np.full(n, np.nan, dtype=nb.float64)
    if n < period + 1:
        return out
    gains = np.empty(n - 1, dtype=nb.float64)
    losses = np.empty(n - 1, dtype=nb.float64)
    for i in range(n - 1):
        d = close[i + 1] - close[i]
        gains[i]  = d if d > 0.0 else 0.0
        losses[i] = -d if d < 0.0 else 0.0
    avg_gain = np.mean(gains[:period])
    avg_loss = np.mean(losses[:period])
    for i in range(period, n - 1):
        avg_gain = (avg_gain * (period - 1) + gains[i]) / period
        avg_loss = (avg_loss * (period - 1) + losses[i]) / period
        if avg_loss == 0.0:
            out[i + 1] = 100.0
        else:
            rs = avg_gain / avg_loss
            out[i + 1] = 100.0 - 100.0 / (1.0 + rs)
    return out


# ---------------------------------------------------------------------------
# MACD signal line
# ---------------------------------------------------------------------------
@njit(cache=True)
def _ema(close: np.ndarray, period: int) -> np.ndarray:
    n = len(close)
    out = np.empty(n, dtype=nb.float64)
    k = 2.0 / (period + 1.0)
    out[0] = close[0]
    for i in range(1, n):
        out[i] = close[i] * k + out[i - 1] * (1.0 - k)
    return out


@njit(cache=True)
def macd_signal(close: np.ndarray, fast: int = 12, slow: int = 26, signal: int = 9):
    ema_fast = _ema(close, fast)
    ema_slow = _ema(close, slow)
    macd_line = ema_fast - ema_slow
    sig_line  = _ema(macd_line, signal)
    return macd_line, sig_line, macd_line - sig_line   # macd, signal, histogram


# ---------------------------------------------------------------------------
# Bollinger Bands
# ---------------------------------------------------------------------------
@njit(cache=True)
def bollinger_bands(close: np.ndarray, window: int = 20, n_std: float = 2.0):
    n = len(close)
    mid   = np.full(n, np.nan, dtype=nb.float64)
    upper = np.full(n, np.nan, dtype=nb.float64)
    lower = np.full(n, np.nan, dtype=nb.float64)
    for i in range(window - 1, n):
        sl = close[i - window + 1 : i + 1]
        m  = np.mean(sl)
        s  = np.std(sl)
        mid[i]   = m
        upper[i] = m + n_std * s
        lower[i] = m - n_std * s
    return upper, mid, lower


# ---------------------------------------------------------------------------
# VWAP deviation
# ---------------------------------------------------------------------------
@njit(cache=True)
def vwap_deviation(close: np.ndarray, volume: np.ndarray, window: int = 390) -> np.ndarray:
    n = len(close)
    out = np.full(n, np.nan, dtype=nb.float64)
    for i in range(window - 1, n):
        pv  = np.sum(close[i - window + 1 : i + 1] * volume[i - window + 1 : i + 1])
        vol = np.sum(volume[i - window + 1 : i + 1])
        vw  = pv / vol if vol > 0.0 else close[i]
        out[i] = (close[i] - vw) / vw if vw != 0.0 else 0.0
    return out


# ---------------------------------------------------------------------------
# Volume z-score
# ---------------------------------------------------------------------------
@njit(float64[:](float64[:], int64), cache=True)
def volume_zscore(volume: np.ndarray, window: int = 20) -> np.ndarray:
    n = len(volume)
    out = np.full(n, np.nan, dtype=nb.float64)
    for i in range(window - 1, n):
        sl = volume[i - window + 1 : i + 1]
        m  = np.mean(sl)
        s  = np.std(sl)
        out[i] = (volume[i] - m) / s if s > 0.0 else 0.0
    return out


# ---------------------------------------------------------------------------
# Order Flow Imbalance (OFI)
# ---------------------------------------------------------------------------
@njit(cache=True)
def order_flow_imbalance(bid_size: np.ndarray, ask_size: np.ndarray) -> np.ndarray:
    n = len(bid_size)
    out = np.empty(n, dtype=nb.float64)
    for i in range(n):
        total = bid_size[i] + ask_size[i]
        if total > 0.0:
            out[i] = (bid_size[i] - ask_size[i]) / total
        else:
            out[i] = 0.0
    return out


# ---------------------------------------------------------------------------
# Spread in basis points
# ---------------------------------------------------------------------------
@njit(float64[:](float64[:], float64[:]), cache=True)
def spread_bps(bid: np.ndarray, ask: np.ndarray) -> np.ndarray:
    n = len(bid)
    out = np.empty(n, dtype=nb.float64)
    for i in range(n):
        mid = (bid[i] + ask[i]) * 0.5
        if mid > 0.0:
            out[i] = (ask[i] - bid[i]) / mid * 10_000.0
        else:
            out[i] = 0.0
    return out


# ---------------------------------------------------------------------------
# Batch computation helper
# ---------------------------------------------------------------------------
def compute_all(
    close: np.ndarray,
    volume: np.ndarray,
    bid: np.ndarray,
    ask: np.ndarray,
    bid_size: np.ndarray,
    ask_size: np.ndarray,
) -> dict:
    """Compute full feature set in one call. Returns dict of arrays."""
    macd_l, sig_l, _ = macd_signal(close)
    upper_bb, mid_bb, lower_bb = bollinger_bands(close)
    return {
        "returns_1":        returns(close),
        "log_returns_1":    log_returns(close),
        "realized_vol_5":   realized_vol(close, 5),
        "realized_vol_20":  realized_vol(close, 20),
        "realized_vol_60":  realized_vol(close, 60),
        "rsi_14":           rsi(close, 14),
        "macd_signal":      sig_l,
        "bb_upper":         upper_bb,
        "bb_lower":         lower_bb,
        "bb_width":         (upper_bb - lower_bb) / np.where(mid_bb != 0, mid_bb, 1.0),
        "vwap_dev":         vwap_deviation(close, volume),
        "volume_zscore":    volume_zscore(volume),
        "ofi":              order_flow_imbalance(bid_size, ask_size),
        "spread_bps":       spread_bps(bid, ask),
    }
