"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { formatCurrency, formatPercentage } from "@/lib/utils";
import { TrendingUp, TrendingDown, Wallet } from "lucide-react";
import { cn } from "@/lib/utils";

// Mock data - will be connected to store in Phase 3.3
const mockData = {
  totalValue: 125430.50,
  totalPnl: 2340.80,
  totalPnlPercent: 1.90,
  buyingPower: 45000.00,
};

export function PortfolioSummary() {
  const isPositive = mockData.totalPnl >= 0;

  return (
    <>
      {/* Total Portfolio Value */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">
            Total Portfolio Value
          </CardTitle>
          <Wallet className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {formatCurrency(mockData.totalValue)}
          </div>
          <div className={cn(
            "flex items-center text-xs",
            isPositive ? "text-up" : "text-down"
          )}>
            {isPositive ? (
              <TrendingUp className="mr-1 h-3 w-3" />
            ) : (
              <TrendingDown className="mr-1 h-3 w-3" />
            )}
            <span>
              {isPositive ? "+" : ""}
              {formatCurrency(mockData.totalPnl)} ({formatPercentage(mockData.totalPnlPercent)})
            </span>
            <span className="ml-1 text-muted-foreground">today</span>
          </div>
        </CardContent>
      </Card>

      {/* Buying Power */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Buying Power</CardTitle>
          <div className="h-4 w-4 rounded-full bg-up/20" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {formatCurrency(mockData.buyingPower)}
          </div>
          <p className="text-xs text-muted-foreground">
            Available for trading
          </p>
        </CardContent>
      </Card>
    </>
  );
}
