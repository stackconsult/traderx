"use client";

import { useEffect, useRef } from "react";
import { useMarketDataStore } from "@/store";

export function useChartData(symbol: string) {
  const candles = useMarketDataStore((state) => state.candles[symbol] ?? []);
  const isLoading = useMarketDataStore((state) => state.isLoading);
  const setCandles = useMarketDataStore((state) => state.setCandles);
  const hasFetched = useRef(false);

  useEffect(() => {
    if (hasFetched.current || !symbol) return;
    hasFetched.current = true;

    // Generate mock OHLCV data seeded from symbol
    const now = Math.floor(Date.now() / 1000);
    const interval = 5 * 60; // 5-minute bars
    const base = symbol === "AAPL" ? 175 : symbol === "GOOGL" ? 140 : symbol === "MSFT" ? 375 : 245;

    const mock = Array.from({ length: 100 }, (_, i) => {
      const t = now - (99 - i) * interval;
      const open = base + (Math.random() - 0.5) * 10;
      const close = open + (Math.random() - 0.5) * 5;
      const high = Math.max(open, close) + Math.random() * 3;
      const low = Math.min(open, close) - Math.random() * 3;
      return {
        time: t,
        open: parseFloat(open.toFixed(2)),
        high: parseFloat(high.toFixed(2)),
        low: parseFloat(low.toFixed(2)),
        close: parseFloat(close.toFixed(2)),
        volume: Math.floor(Math.random() * 1000000) + 100000,
      };
    });

    setCandles(symbol, mock);
  }, [symbol, setCandles]);

  return { candles, isLoading };
}
