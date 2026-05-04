"use client";

import { cn, formatNumber } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { TrendingUp, TrendingDown, Minus, Zap, X } from "lucide-react";
import type { Signal } from "@/types/store";

interface SignalCardProps {
  signal: Signal;
  onOrderFromSignal?: (signal: Signal) => void;
  onDismiss?: (signalId: string) => void;
}

const directionConfig = {
  long: { icon: TrendingUp, label: "Long", color: "text-up", bg: "bg-up/10" },
  short: { icon: TrendingDown, label: "Short", color: "text-down", bg: "bg-down/10" },
  neutral: { icon: Minus, label: "Neutral", color: "text-muted-foreground", bg: "bg-muted" },
};

export function SignalCard({ signal, onOrderFromSignal, onDismiss }: SignalCardProps) {
  const config = directionConfig[signal.direction];
  const Icon = config.icon;

  const timeAgo = (timestamp: string) => {
    const seconds = Math.floor((Date.now() - new Date(timestamp).getTime()) / 1000);
    if (seconds < 60) return `${seconds}s ago`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    return `${Math.floor(seconds / 3600)}h ago`;
  };

  const confidencePct = Math.round(signal.confidence * 100);

  return (
    <div className="rounded-lg border bg-card p-3 space-y-2">
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-2">
          <div className={cn("rounded-full p-1", config.bg)}>
            <Icon className={cn("h-3 w-3", config.color)} />
          </div>
          <div>
            <div className="flex items-center gap-1.5">
              <span className="text-sm font-semibold">{signal.symbol}</span>
              <span className={cn("text-xs font-medium", config.color)}>
                {config.label}
              </span>
            </div>
            <p className="text-xs text-muted-foreground">{signal.strategy}</p>
          </div>
        </div>
        <div className="flex items-center gap-1">
          <span className="text-xs text-muted-foreground">
            {timeAgo(signal.timestamp)}
          </span>
          {onDismiss && (
            <Button
              variant="ghost"
              size="sm"
              className="h-6 w-6 p-0 text-muted-foreground"
              onClick={() => onDismiss(signal.signalId)}
            >
              <X className="h-3 w-3" />
            </Button>
          )}
        </div>
      </div>

      {/* Confidence bar */}
      <div className="space-y-1">
        <div className="flex items-center justify-between text-xs">
          <span className="text-muted-foreground">Confidence</span>
          <span className={cn("font-medium", confidencePct >= 80 ? "text-up" : confidencePct >= 60 ? "text-foreground" : "text-down")}>
            {confidencePct}%
          </span>
        </div>
        <div className="h-1.5 rounded-full bg-muted overflow-hidden">
          <div
            className={cn(
              "h-full rounded-full transition-all",
              confidencePct >= 80 ? "bg-up" : confidencePct >= 60 ? "bg-primary" : "bg-down",
              confidencePct === 100 ? "w-full" :
                confidencePct >= 90 ? "w-[90%]" :
                  confidencePct >= 80 ? "w-[80%]" :
                    confidencePct >= 70 ? "w-[70%]" :
                      confidencePct >= 60 ? "w-[60%]" : "w-[50%]"
            )}
          />
        </div>
      </div>

      {/* One-click order */}
      {onOrderFromSignal && (
        <Button
          size="sm"
          variant="outline"
          className="w-full h-7 text-xs"
          onClick={() => onOrderFromSignal(signal)}
        >
          <Zap className="mr-1 h-3 w-3" />
          Trade Signal
        </Button>
      )}
    </div>
  );
}
