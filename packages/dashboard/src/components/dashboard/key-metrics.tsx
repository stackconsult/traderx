"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { formatCurrency, formatNumber } from "@/lib/utils";
import { Activity, Target, BarChart3 } from "lucide-react";

// Mock data - will be connected to store in Phase 3.3
const mockMetrics = {
  openPositions: 12,
  pendingOrders: 3,
  dayTrades: 8,
  winRate: 68.5,
  avgPosition: 8500,
};

export function KeyMetrics() {
  return (
    <>
      {/* Open Positions */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Open Positions</CardTitle>
          <BarChart3 className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{mockMetrics.openPositions}</div>
          <p className="text-xs text-muted-foreground">
            {mockMetrics.pendingOrders} pending orders
          </p>
        </CardContent>
      </Card>

      {/* Win Rate */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Win Rate</CardTitle>
          <Target className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{formatNumber(mockMetrics.winRate, 1)}%</div>
          <p className="text-xs text-muted-foreground">
            {mockMetrics.dayTrades} trades today
          </p>
        </CardContent>
      </Card>
    </>
  );
}
