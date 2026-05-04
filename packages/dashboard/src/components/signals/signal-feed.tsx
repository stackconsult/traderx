"use client";

import { useState } from "react";
import { SignalCard } from "./signal-card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useSignalStore } from "@/store";
import { cn } from "@/lib/utils";
import { Filter } from "lucide-react";
import type { Signal } from "@/types/store";

// Mock signals for development
const MOCK_SIGNALS: Signal[] = [
  { signalId: "s1", symbol: "AAPL", direction: "long", confidence: 0.87, strategy: "momentum-breakout", timestamp: new Date(Date.now() - 90000).toISOString() },
  { signalId: "s2", symbol: "TSLA", direction: "short", confidence: 0.73, strategy: "mean-reversion", timestamp: new Date(Date.now() - 180000).toISOString() },
  { signalId: "s3", symbol: "NVDA", direction: "long", confidence: 0.91, strategy: "trend-following", timestamp: new Date(Date.now() - 300000).toISOString() },
  { signalId: "s4", symbol: "MSFT", direction: "neutral", confidence: 0.65, strategy: "pattern-detection", timestamp: new Date(Date.now() - 420000).toISOString() },
];

export function SignalFeed() {
  const [dismissed, setDismissed] = useState<Set<string>>(new Set());
  const minConfidence = useSignalStore((state) => state.minConfidence);
  const setMinConfidence = useSignalStore((state) => state.setMinConfidence);
  const [showFilter, setShowFilter] = useState(false);

  const visible = MOCK_SIGNALS.filter(
    (s) => !dismissed.has(s.signalId) && s.confidence >= minConfidence
  );

  const handleDismiss = (signalId: string) => {
    setDismissed((prev) => new Set([...prev, signalId]));
  };

  const handleOrderFromSignal = (signal: Signal) => {
    console.log("Create order from signal:", signal);
  };

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium text-muted-foreground">
          Signals
          {visible.length > 0 && (
            <span className="ml-1.5 rounded-full bg-primary px-1.5 py-0.5 text-[10px] text-primary-foreground">
              {visible.length}
            </span>
          )}
        </h3>
        <Button
          variant="ghost"
          size="sm"
          className="h-7 w-7 p-0"
          onClick={() => setShowFilter(!showFilter)}
        >
          <Filter className="h-3.5 w-3.5" />
        </Button>
      </div>

      {showFilter && (
        <div className="flex items-center gap-2 rounded-md bg-muted p-2">
          <span className="text-xs text-muted-foreground whitespace-nowrap">Min confidence</span>
          <input
            type="range"
            min="0"
            max="1"
            step="0.05"
            value={minConfidence}
            onChange={(e) => setMinConfidence(parseFloat(e.target.value))}
            className="flex-1"
          />
          <span className="text-xs font-medium w-8">{Math.round(minConfidence * 100)}%</span>
        </div>
      )}

      <div className={cn("space-y-2", visible.length === 0 && "text-center py-4")}>
        {visible.length === 0 ? (
          <p className="text-xs text-muted-foreground">No signals above {Math.round(minConfidence * 100)}% confidence</p>
        ) : (
          visible.map((signal) => (
            <SignalCard
              key={signal.signalId}
              signal={signal}
              onOrderFromSignal={handleOrderFromSignal}
              onDismiss={handleDismiss}
            />
          ))
        )}
      </div>
    </div>
  );
}
