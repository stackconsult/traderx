"use client";

import { DashboardLayout } from "@/components/layout/dashboard-layout";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { PortfolioSummary } from "@/components/dashboard/portfolio-summary";
import { KeyMetrics } from "@/components/dashboard/key-metrics";
import { OrderEntry } from "@/components/trading/order-entry";
import { Watchlist } from "@/components/market/watchlist";
import { SignalFeed } from "@/components/signals/signal-feed";
import { TradingViewChart } from "@/components/charts/trading-view-chart";

export default function DashboardPage() {
  return (
    <DashboardLayout>
      {/* Top Row - Portfolio Summary & Key Metrics (4 cards) */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4 mb-4">
        <PortfolioSummary />
        <KeyMetrics />
      </div>

      {/* Main Content Area - 3-column responsive grid */}
      <div className="grid gap-4 lg:grid-cols-3">
        {/* Left + Center: Chart (2/3 desktop) */}
        <div className="lg:col-span-2 space-y-4">
          {/* TradingView Chart */}
          <Card className="h-[500px] overflow-hidden">
            <TradingViewChart className="h-full" />
          </Card>

          {/* Positions stub - Phase 3.4 real-time wiring */}
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">Open Positions</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="rounded-md bg-muted/50 p-4 text-center text-sm text-muted-foreground">
                Real-time position table — Phase 4 wire-up
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Right column: Order Entry + Watchlist + Signals (1/3 desktop) */}
        <div className="space-y-4">
          {/* Order Entry Form */}
          <OrderEntry />

          {/* Watchlist with flash tickers */}
          <Card>
            <CardContent className="pt-4">
              <Watchlist />
            </CardContent>
          </Card>

          {/* Neural Signal Feed */}
          <Card>
            <CardContent className="pt-4">
              <SignalFeed />
            </CardContent>
          </Card>
        </div>
      </div>
    </DashboardLayout>
  );
}
