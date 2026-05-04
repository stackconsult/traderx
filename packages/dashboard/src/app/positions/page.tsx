"use client";

import { DashboardLayout } from "@/components/layout/dashboard-layout";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export default function PositionsPage() {
  return (
    <DashboardLayout>
      <Card>
        <CardHeader>
          <CardTitle>Positions</CardTitle>
        </CardHeader>
        <CardContent className="h-[600px] flex items-center justify-center text-muted-foreground">
          Full Position Table
          <br />
          (Implementation in Chunk 3.3)
        </CardContent>
      </Card>
    </DashboardLayout>
  );
}
