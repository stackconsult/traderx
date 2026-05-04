"use client";

import { DashboardLayout } from "@/components/layout/dashboard-layout";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { PortfolioSummary } from "@/components/dashboard/portfolio-summary";
import { KeyMetrics } from "@/components/dashboard/key-metrics";
import { OrderEntry } from "@/components/trading/order-entry";
import { Watchlist } from "@/components/market/watchlist";
import { SignalFeed } from "@/components/signals/signal-feed";
import { TradingViewChart } from "@/components/charts/trading-view-chart";
import { PositionTable } from "@/components/trading/position-table";
import { OrderHistoryTable } from "@/components/trading/order-history-table";
import { ErrorBoundary } from "@/components/error-boundary";
import { SkeletonChart } from "@/components/ui/skeleton";

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
            <ErrorBoundary fallback={<SkeletonChart />}>
              <TradingViewChart className="h-full" />
            </ErrorBoundary>
          </Card>

          {/* Position Table */}
          <ErrorBoundary>
            <PositionTable />
          </ErrorBoundary>

          {/* Order History */}
          <ErrorBoundary>
            <OrderHistoryTable />
          </ErrorBoundary>
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
