"use client";

import { useState, useEffect, useRef } from "react";
import { SignalCard } from "./signal-card";
import { Button } from "@/components/ui/button";
import { useSignalStore } from "@/store";
import { cn } from "@/lib/utils";
import { Filter } from "lucide-react";
import type { Signal } from "@/types/store";
import type { SignalState } from "@/types/store-states";
import { toast } from "@/hooks/use-toast";

export function SignalFeed() {
  const [dismissed, setDismissed] = useState<Set<string>>(new Set());
  const signals = useSignalStore((state: SignalState) => state.signals);
  const minConfidence = useSignalStore((state: SignalState) => state.minConfidence);
  const setMinConfidence = useSignalStore((state: SignalState) => state.setMinConfidence);
  const [showFilter, setShowFilter] = useState(false);
  const prevCountRef = useRef(signals.length);

  useEffect(() => {
    if (signals.length > prevCountRef.current) {
      const newest = signals[0];
      if (newest) {
        toast({
          title: `New signal: ${newest.symbol}`,
          description: `${newest.direction.toUpperCase()} — ${newest.strategy} (${Math.round(newest.confidence * 100)}% confidence)`,
          variant: newest.direction === "long" ? "success" : newest.direction === "short" ? "destructive" : "default",
        });
      }
    }
    prevCountRef.current = signals.length;
  }, [signals.length]);

  const visible = signals.filter(
    (s) => !dismissed.has(s.signalId) && s.confidence >= minConfidence
  );

  const handleDismiss = (signalId: string) => {
    setDismissed((prev: Set<string>) => new Set([...prev, signalId]));
  };

  const handleOrderFromSignal = (signal: Signal) => {
    window.dispatchEvent(new CustomEvent("traderx:prefill-order", {
      detail: {
        symbol: signal.symbol,
        side: signal.direction === "long" ? "buy" : signal.direction === "short" ? "sell" : "buy",
      },
    }));
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
