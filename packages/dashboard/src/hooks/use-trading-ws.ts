"use client";

import { useEffect, useRef } from "react";
import { getWebSocket, closeWebSocket } from "@/lib/websocket";
import { useAuthStore } from "@/store/auth-store";
import { useMarketDataStore } from "@/store/market-data-store";
import type { AuthState } from "@/types/store-states";
import type { MarketDataState } from "@/types/store-states";

export function useTradingWs() {
  const isAuthenticated = useAuthStore((state: AuthState) => state.isAuthenticated);
  const watchlist = useMarketDataStore((state: MarketDataState) => state.watchlist);
  const initialised = useRef(false);

  useEffect(() => {
    if (!isAuthenticated || initialised.current) return;
    initialised.current = true;

    const ws = getWebSocket();
    ws.connect();

    if (watchlist.length > 0) {
      ws.subscribe(watchlist);
    }

    return () => {
      closeWebSocket();
      initialised.current = false;
    };
  }, [isAuthenticated]);

  useEffect(() => {
    if (!isAuthenticated || watchlist.length === 0) return;
    const ws = getWebSocket();
    ws.subscribe(watchlist);
  }, [watchlist, isAuthenticated]);
}
