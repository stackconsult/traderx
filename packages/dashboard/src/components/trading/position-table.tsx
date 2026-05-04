"use client";

import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { cn, formatCurrency, formatNumber } from "@/lib/utils";
import { ArrowUpDown, ArrowUp, ArrowDown, X } from "lucide-react";
import { usePositionStore } from "@/store";
import type { PositionState } from "@/types/store-states";
import type { Position } from "@/types/store";

type SortKey = "symbol" | "quantity" | "unrealizedPnl" | "marketValue";
type SortDir = "asc" | "desc";

const MOCK_POSITIONS: Position[] = [
  { symbol: "AAPL", side: "long", quantity: 100, avgEntryPrice: 172.5, marketPrice: 178.35, marketValue: 17835, unrealizedPnl: 585, unrealizedPnlPct: 3.39, realizedPnl: 0 },
  { symbol: "NVDA", side: "long", quantity: 20, avgEntryPrice: 875.0, marketPrice: 892.45, marketValue: 17849, unrealizedPnl: 349, unrealizedPnlPct: 1.99, realizedPnl: 0 },
  { symbol: "TSLA", side: "short", quantity: 50, avgEntryPrice: 252.0, marketPrice: 245.67, marketValue: 12283, unrealizedPnl: 316.5, unrealizedPnlPct: 2.51, realizedPnl: 0 },
  { symbol: "GOOGL", side: "long", quantity: 30, avgEntryPrice: 145.0, marketPrice: 142.8, marketValue: 4284, unrealizedPnl: -66, unrealizedPnlPct: -1.52, realizedPnl: 0 },
];

export function PositionTable() {
  const [sortKey, setSortKey] = useState<SortKey>("unrealizedPnl");
  const [sortDir, setSortDir] = useState<SortDir>("desc");
  const storePositions = usePositionStore((state: PositionState) => state.positions);

  const positions = storePositions.length > 0 ? storePositions : MOCK_POSITIONS;

  const handleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortDir(sortDir === "asc" ? "desc" : "asc");
    } else {
      setSortKey(key);
      setSortDir("desc");
    }
  };

  const sorted = [...positions].sort((a, b) => {
    const mul = sortDir === "asc" ? 1 : -1;
    if (sortKey === "symbol") return mul * a.symbol.localeCompare(b.symbol);
    return mul * ((a[sortKey] as number) - (b[sortKey] as number));
  });

  const SortIcon = ({ col }: { col: SortKey }) => {
    if (sortKey !== col) return <ArrowUpDown className="ml-1 h-3 w-3 text-muted-foreground" />;
    return sortDir === "asc"
      ? <ArrowUp className="ml-1 h-3 w-3" />
      : <ArrowDown className="ml-1 h-3 w-3" />;
  };

  const totalPnl = positions.reduce((sum: number, p: Position) => sum + p.unrealizedPnl, 0);

  return (
    <Card>
      <CardHeader className="pb-2 flex flex-row items-center justify-between">
        <CardTitle className="text-sm font-medium">Open Positions</CardTitle>
        <span className={cn("text-sm font-semibold", totalPnl >= 0 ? "text-up" : "text-down")}>
          {totalPnl >= 0 ? "+" : ""}{formatCurrency(totalPnl)} total P&L
        </span>
      </CardHeader>
      <CardContent className="p-0">
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b bg-muted/30">
                {[
                  { key: "symbol" as SortKey, label: "Symbol" },
                  { key: "quantity" as SortKey, label: "Qty" },
                  { key: null, label: "Entry" },
                  { key: null, label: "Current" },
                  { key: "unrealizedPnl" as SortKey, label: "P&L" },
                  { key: "marketValue" as SortKey, label: "Value" },
                  { key: null, label: "" },
                ].map(({ key, label }, i) => (
                  <th
                    key={i}
                    className={cn(
                      "px-3 py-2 text-left text-xs font-medium text-muted-foreground",
                      key && "cursor-pointer select-none hover:text-foreground"
                    )}
                    onClick={() => key && handleSort(key)}
                  >
                    <span className="inline-flex items-center">
                      {label}
                      {key && <SortIcon col={key} />}
                    </span>
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {sorted.map((pos) => {
                const isPositive = pos.unrealizedPnl >= 0;
                return (
                  <tr key={pos.symbol} className="border-b last:border-0 hover:bg-muted/20 transition-colors">
                    <td className="px-3 py-2">
                      <div className="flex items-center gap-1.5">
                        <span className={cn("text-[10px] font-medium px-1 rounded", pos.side === "long" ? "bg-up/20 text-up" : "bg-down/20 text-down")}>
                          {pos.side.toUpperCase()}
                        </span>
                        <span className="font-medium">{pos.symbol}</span>
                      </div>
                    </td>
                    <td className="px-3 py-2 text-right">{formatNumber(pos.quantity)}</td>
                    <td className="px-3 py-2 text-right">{formatCurrency(pos.avgEntryPrice)}</td>
                    <td className="px-3 py-2 text-right">{formatCurrency(pos.marketPrice)}</td>
                    <td className="px-3 py-2 text-right">
                      <div className={cn("font-medium", isPositive ? "text-up" : "text-down")}>
                        {isPositive ? "+" : ""}{formatCurrency(pos.unrealizedPnl)}
                      </div>
                      <div className={cn("text-xs", isPositive ? "text-up" : "text-down")}>
                        {isPositive ? "+" : ""}{formatNumber(pos.unrealizedPnlPct)}%
                      </div>
                    </td>
                    <td className="px-3 py-2 text-right">{formatCurrency(pos.marketValue)}</td>
                    <td className="px-3 py-2">
                      <Button variant="ghost" size="sm" className="h-7 w-7 p-0 text-muted-foreground hover:text-down">
                        <X className="h-3.5 w-3.5" />
                      </Button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
          {positions.length === 0 && (
            <div className="py-8 text-center text-sm text-muted-foreground">No open positions</div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
