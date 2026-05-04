"use client";

import { cn } from "@/lib/utils";
import { useMarketDataStore } from "@/store";

type Timeframe = "1m" | "5m" | "15m" | "1h" | "4h" | "1d";

const TIMEFRAMES: Timeframe[] = ["1m", "5m", "15m", "1h", "4h", "1d"];

export function TimeframeSelector() {
  const selected = useMarketDataStore((state) => state.selectedTimeframe);
  const setSelected = useMarketDataStore((state) => state.setSelectedTimeframe);

  return (
    <div className="flex items-center gap-1">
      {TIMEFRAMES.map((tf) => (
        <button
          key={tf}
          onClick={() => setSelected(tf)}
          className={cn(
            "rounded px-2 py-1 text-xs font-medium transition-colors",
            selected === tf
              ? "bg-primary text-primary-foreground"
              : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
          )}
        >
          {tf.toUpperCase()}
        </button>
      ))}
    </div>
  );
}
