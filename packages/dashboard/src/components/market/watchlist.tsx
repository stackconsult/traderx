"use client";

import { PriceTicker } from "./price-ticker";
import { Button } from "@/components/ui/button";
import { X, Plus } from "lucide-react";
import { cn } from "@/lib/utils";
import { useMarketDataStore } from "@/store";
import type { MarketDataState } from "@/types/store-states";

const DEFAULT_WATCHLIST = ["AAPL", "GOOGL", "MSFT", "TSLA", "NVDA"];

interface WatchlistProps {
  className?: string;
}

export function Watchlist({ className }: WatchlistProps) {
  const watchlist = useMarketDataStore((state: MarketDataState) => state.watchlist);
  const prices = useMarketDataStore((state: MarketDataState) => state.prices);
  const removeFromWatchlist = useMarketDataStore((state: MarketDataState) => state.removeFromWatchlist);

  const symbols = watchlist.length > 0 ? watchlist : DEFAULT_WATCHLIST;

  return (
    <div className={cn("space-y-3", className)}>
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium text-muted-foreground">Watchlist</h3>
        <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
          <Plus className="h-4 w-4" />
        </Button>
      </div>

      <div className="grid gap-2">
        {symbols.map((symbol) => {
          const data = prices[symbol];
          return (
            <div key={symbol} className="group relative" data-symbol={symbol}>
              <PriceTicker
                symbol={symbol}
                price={data?.price ?? 0}
                change={data?.change ?? 0}
                changePercent={data?.changePercent ?? 0}
                volume={data?.volume ?? 0}
                size="sm"
              />
              <Button
                variant="ghost"
                size="sm"
                className="absolute right-1 top-1 h-6 w-6 p-0 opacity-0 group-hover:opacity-100"
                onClick={() => removeFromWatchlist(symbol)}
              >
                <X className="h-3 w-3" />
              </Button>
            </div>
          );
        })}
      </div>
    </div>
  );
}
