"use client";

import { DashboardLayout } from "@/components/layout/dashboard-layout";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { PortfolioSummary } from "@/components/dashboard/portfolio-summary";
import { KeyMetrics } from "@/components/dashboard/key-metrics";
import { OrderEntry } from "@/components/trading/order-entry";

export default function DashboardPage() {
  return (
    <DashboardLayout>
      {/* Top Row - Portfolio Summary & Key Metrics */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4 mb-4">
        <PortfolioSummary />
        <KeyMetrics />
      </div>

      {/* Main Content Area */}
      <div className="grid gap-4 lg:grid-cols-3">
        {/* Left Column - Chart (60% on desktop) */}
        <div className="lg:col-span-2 space-y-4">
          <Card className="h-[500px]">
            <CardHeader>
              <CardTitle>Chart</CardTitle>
            </CardHeader>
            <CardContent className="h-[400px] flex items-center justify-center text-muted-foreground">
              TradingView Chart Component
              <br />
              (Implementation in Chunk 5.1)
            </CardContent>
          </Card>

          {/* Position Table */}
          <Card>
            <CardHeader>
              <CardTitle>Positions</CardTitle>
            </CardHeader>
            <CardContent className="h-[200px] flex items-center justify-center text-muted-foreground">
              Position Table Component
              <br />
              (Implementation in Chunk 3.3)
            </CardContent>
          </Card>
        </div>

        {/* Right Column - Order Entry + Watchlist (40% on desktop) */}
        <div className="space-y-4">
          <OrderEntry />

          {/* Watchlist */}
          <Card>
            <CardHeader>
              <CardTitle>Watchlist</CardTitle>
            </CardHeader>
            <CardContent className="h-[200px] flex items-center justify-center text-muted-foreground">
              Watchlist Component
              <br />
              (Implementation in Chunk 4.2)
            </CardContent>
          </Card>

          {/* Signal Feed */}
          <Card>
            <CardHeader>
              <CardTitle>Signals</CardTitle>
            </CardHeader>
            <CardContent className="h-[200px] flex items-center justify-center text-muted-foreground">
              Signal Feed Component
              <br />
              (Implementation in Chunk 5.2)
            </CardContent>
          </Card>
        </div>
      </div>
    </DashboardLayout>
  );
}
