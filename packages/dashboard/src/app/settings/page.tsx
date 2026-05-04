"use client";

import { DashboardLayout } from "@/components/layout/dashboard-layout";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";

export default function SettingsPage() {
  return (
    <DashboardLayout>
      <div className="space-y-6">
        <div>
          <h1 className="text-3xl font-bold">Settings</h1>
          <p className="text-muted-foreground">
            Manage your account and trading preferences
          </p>
        </div>

        <Separator />

        {/* Account Settings */}
        <Card>
          <CardHeader>
            <CardTitle>Account</CardTitle>
            <CardDescription>
              Manage your account details and security
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid gap-2">
              <label className="text-sm font-medium">Username</label>
              <p className="text-sm text-muted-foreground">trader_pro</p>
            </div>
            <div className="grid gap-2">
              <label className="text-sm font-medium">Email</label>
              <p className="text-sm text-muted-foreground">trader@example.com</p>
            </div>
            <Button variant="outline">Change Password</Button>
          </CardContent>
        </Card>

        {/* Trading Preferences */}
        <Card>
          <CardHeader>
            <CardTitle>Trading Preferences</CardTitle>
            <CardDescription>
              Configure your default trading settings
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid gap-2">
              <label className="text-sm font-medium">Default Order Type</label>
              <p className="text-sm text-muted-foreground">Limit</p>
            </div>
            <div className="grid gap-2">
              <label className="text-sm font-medium">Default Time in Force</label>
              <p className="text-sm text-muted-foreground">Day</p>
            </div>
            <Button variant="outline">Edit Preferences</Button>
          </CardContent>
        </Card>

        {/* Notifications */}
        <Card>
          <CardHeader>
            <CardTitle>Notifications</CardTitle>
            <CardDescription>
              Manage your notification preferences
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm">Order Execution Alerts</span>
              <Button variant="outline" size="sm">Enabled</Button>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">Price Alerts</span>
              <Button variant="outline" size="sm">Enabled</Button>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">Signal Notifications</span>
              <Button variant="outline" size="sm">Disabled</Button>
            </div>
          </CardContent>
        </Card>
      </div>
    </DashboardLayout>
  );
}
