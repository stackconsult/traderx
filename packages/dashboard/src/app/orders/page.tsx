"use client";

import { DashboardLayout } from "@/components/layout/dashboard-layout";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export default function OrdersPage() {
  return (
    <DashboardLayout>
      <Card>
        <CardHeader>
          <CardTitle>Order History</CardTitle>
        </CardHeader>
        <CardContent className="h-[600px] flex items-center justify-center text-muted-foreground">
          Order History Table
          <br />
          (Implementation in Phase 3.4)
        </CardContent>
      </Card>
    </DashboardLayout>
  );
}
