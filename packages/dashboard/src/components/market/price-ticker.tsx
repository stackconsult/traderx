"use client";

import { usePriceFlash } from "@/hooks/use-price-flash";
import { formatNumber, formatCurrency, cn } from "@/lib/utils";
import { TrendingUp, TrendingDown } from "lucide-react";

interface PriceTickerProps {
  symbol: string;
  price: number;
  change?: number;
  changePercent?: number;
  volume?: number;
  className?: string;
  size?: "sm" | "md" | "lg";
}

export function PriceTicker({
  symbol,
  price,
  change = 0,
  changePercent = 0,
  volume,
  className,
  size = "md",
}: PriceTickerProps) {
  const flash = usePriceFlash(price);
  const isPositive = change >= 0;

  const sizeClasses = {
    sm: {
      symbol: "text-xs",
      price: "text-sm font-semibold",
      change: "text-xs",
    },
    md: {
      symbol: "text-sm font-medium",
      price: "text-lg font-bold",
      change: "text-sm",
    },
    lg: {
      symbol: "text-base font-medium",
      price: "text-2xl font-bold",
      change: "text-sm",
    },
  };

  const flashClass = flash === "up" ? "flash-up" : flash === "down" ? "flash-down" : "";

  return (
    <div className={cn("rounded-lg border bg-card p-3", flashClass, className)}>
      <div className="flex items-center justify-between">
        <span className={cn("text-muted-foreground", sizeClasses[size].symbol)}>
          {symbol}
        </span>
        {isPositive ? (
          <TrendingUp className="h-3 w-3 text-up" />
        ) : (
          <TrendingDown className="h-3 w-3 text-down" />
        )}
      </div>

      <div className={cn("mt-1", sizeClasses[size].price)}>
        {formatCurrency(price)}
      </div>

      <div className={cn("flex items-center gap-1", sizeClasses[size].change)}>
        <span className={isPositive ? "text-up" : "text-down"}>
          {isPositive ? "+" : ""}
          {formatNumber(change)}
        </span>
        <span className={isPositive ? "text-up" : "text-down"}>
          ({isPositive ? "+" : ""}
          {formatNumber(changePercent)}%)
        </span>
      </div>

      {volume !== undefined && (
        <div className="mt-1 text-xs text-muted-foreground">
          Vol: {volume.toLocaleString()}
        </div>
      )}
    </div>
  );
}
