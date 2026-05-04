"use client";

import { PriceTicker } from "./price-ticker";
import { Button } from "@/components/ui/button";
import { X, Plus } from "lucide-react";
import { cn } from "@/lib/utils";
import { useMarketDataStore } from "@/store";

interface WatchlistProps {
  className?: string;
}

// Mock data - will be replaced with real-time data
const MOCK_DATA = [
  { symbol: "AAPL", price: 178.35, change: 2.45, changePercent: 1.39, volume: 52400000 },
  { symbol: "GOOGL", price: 142.8, change: -0.92, changePercent: -0.64, volume: 18900000 },
  { symbol: "MSFT", price: 378.91, change: 5.12, changePercent: 1.37, volume: 22100000 },
  { symbol: "TSLA", price: 245.67, change: -3.21, changePercent: -1.29, volume: 98200000 },
  { symbol: "NVDA", price: 892.45, change: 12.34, changePercent: 1.40, volume: 45600000 },
];

export function Watchlist({ className }: WatchlistProps) {
  const watchlist = useMarketDataStore((state) => state.watchlist);
  const removeFromWatchlist = useMarketDataStore((state) => state.removeFromWatchlist);

  return (
    <div className={cn("space-y-3", className)}>
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium text-muted-foreground">Watchlist</h3>
        <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
          <Plus className="h-4 w-4" />
        </Button>
      </div>

      <div className="grid gap-2">
        {MOCK_DATA.map((item) => (
          <div key={item.symbol} className="group relative">
            <PriceTicker
              symbol={item.symbol}
              price={item.price}
              change={item.change}
              changePercent={item.changePercent}
              volume={item.volume}
              size="sm"
            />
            {watchlist.includes(item.symbol) && (
              <Button
                variant="ghost"
                size="sm"
                className="absolute right-1 top-1 h-6 w-6 p-0 opacity-0 group-hover:opacity-100"
                onClick={() => removeFromWatchlist(item.symbol)}
              >
                <X className="h-3 w-3" />
              </Button>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
