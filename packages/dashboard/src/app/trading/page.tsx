"use client";

import { DashboardLayout } from "@/components/layout/dashboard-layout";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { OrderEntry } from "@/components/trading/order-entry";

export default function TradingPage() {
  return (
    <DashboardLayout>
      <div className="grid gap-4 lg:grid-cols-3">
        {/* Main Chart Area */}
        <div className="lg:col-span-2">
          <Card className="h-[600px]">
            <CardHeader>
              <CardTitle>Advanced Trading</CardTitle>
            </CardHeader>
            <CardContent className="h-[500px] flex items-center justify-center text-muted-foreground">
              Full TradingView Chart
              <br />
              (Implementation in Chunk 5.1)
            </CardContent>
          </Card>
        </div>

        {/* Order Entry Panel */}
        <div>
          <OrderEntry />
        </div>
      </div>
    </DashboardLayout>
  );
}
