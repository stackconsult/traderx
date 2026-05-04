"use client";

import { Header } from "./header";
import { Sidebar } from "./sidebar";
import { MobileNav } from "./mobile-nav";
import { cn } from "@/lib/utils";

interface DashboardLayoutProps {
  children: React.ReactNode;
  className?: string;
}

export function DashboardLayout({ children, className }: DashboardLayoutProps) {
  return (
    <div className="flex min-h-screen bg-background">
      {/* Desktop Sidebar */}
      <Sidebar className="hidden lg:flex" />

      {/* Main Content */}
      <div className="flex flex-1 flex-col">
        <Header />

        {/* Mobile Navigation */}
        <MobileNav className="lg:hidden" />

        {/* Page Content */}
        <main className={cn("flex-1 overflow-auto p-4 lg:p-6", className)}>
          {children}
        </main>
      </div>
    </div>
  );
}
