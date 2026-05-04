"use client";

import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { cn, formatCurrency } from "@/lib/utils";
import { X } from "lucide-react";
import { useOrderStore } from "@/store";
import type { OrderState } from "@/types/store-states";
import type { Order } from "@/types/store";

type FilterStatus = "all" | Order["status"];

const STATUS_COLORS: Record<Order["status"], string> = {
  pending:   "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400",
  submitted: "bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400",
  filled:    "bg-up/10 text-up",
  cancelled: "bg-muted text-muted-foreground",
  rejected:  "bg-down/10 text-down",
};

const MOCK_ORDERS: Order[] = [
  { orderId: "o1", symbol: "AAPL", side: "buy",  orderType: "limit",  quantity: 100, filledQuantity: 100, price: 172.5,  avgPrice: 172.5,  status: "filled",    timeInForce: "day", createdAt: new Date(Date.now() - 3600000).toISOString() },
  { orderId: "o2", symbol: "NVDA", side: "buy",  orderType: "market", quantity: 20,  filledQuantity: 20,  price: undefined, avgPrice: 875.2, status: "filled",    timeInForce: "day", createdAt: new Date(Date.now() - 7200000).toISOString() },
  { orderId: "o3", symbol: "TSLA", side: "sell", orderType: "limit",  quantity: 50,  filledQuantity: 0,   price: 255.0, status: "pending",   timeInForce: "gtc", createdAt: new Date(Date.now() - 900000).toISOString() },
  { orderId: "o4", symbol: "GOOGL", side: "buy", orderType: "stop",   quantity: 30,  filledQuantity: 0,   price: 148.0, status: "cancelled", timeInForce: "day", createdAt: new Date(Date.now() - 86400000).toISOString() },
];

const FILTERS: { label: string; value: FilterStatus }[] = [
  { label: "All",       value: "all" },
  { label: "Pending",   value: "pending" },
  { label: "Filled",    value: "filled" },
  { label: "Cancelled", value: "cancelled" },
];

export function OrderHistoryTable() {
  const [filter, setFilter] = useState<FilterStatus>("all");
  const storeOrders = useOrderStore((state: OrderState) => state.orders);
  const orders = storeOrders.length > 0 ? storeOrders : MOCK_ORDERS;

  const filtered = filter === "all" ? orders : orders.filter((o) => o.status === filter);

  const formatTime = (iso: string) =>
    new Date(iso).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });

  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium">Order History</CardTitle>
          <div className="flex gap-1">
            {FILTERS.map(({ label, value }) => (
              <button
                key={value}
                onClick={() => setFilter(value)}
                className={cn(
                  "rounded px-2 py-0.5 text-xs font-medium transition-colors",
                  filter === value
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:bg-accent"
                )}
              >
                {label}
              </button>
            ))}
          </div>
        </div>
      </CardHeader>
      <CardContent className="p-0">
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b bg-muted/30">
                {["Time", "Symbol", "Side", "Type", "Qty", "Price", "Status", ""].map((h, i) => (
                  <th key={i} className="px-3 py-2 text-left text-xs font-medium text-muted-foreground">
                    {h}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {filtered.map((order) => (
                <tr key={order.orderId} className="border-b last:border-0 hover:bg-muted/20 transition-colors">
                  <td className="px-3 py-2 text-xs text-muted-foreground whitespace-nowrap">
                    {formatTime(order.createdAt)}
                  </td>
                  <td className="px-3 py-2 font-medium">{order.symbol}</td>
                  <td className="px-3 py-2">
                    <span className={cn("text-xs font-medium", order.side === "buy" ? "text-up" : "text-down")}>
                      {order.side.toUpperCase()}
                    </span>
                  </td>
                  <td className="px-3 py-2 text-xs capitalize text-muted-foreground">
                    {order.orderType.replace("_", " ")}
                  </td>
                  <td className="px-3 py-2 text-right">
                    {order.filledQuantity}/{order.quantity}
                  </td>
                  <td className="px-3 py-2 text-right">
                    {order.avgPrice ? formatCurrency(order.avgPrice) : order.price ? formatCurrency(order.price) : "MKT"}
                  </td>
                  <td className="px-3 py-2">
                    <span className={cn("rounded px-1.5 py-0.5 text-[10px] font-medium", STATUS_COLORS[order.status])}>
                      {order.status}
                    </span>
                  </td>
                  <td className="px-3 py-2">
                    {order.status === "pending" && (
                      <Button variant="ghost" size="sm" className="h-6 w-6 p-0 text-muted-foreground hover:text-down">
                        <X className="h-3 w-3" />
                      </Button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {filtered.length === 0 && (
            <div className="py-8 text-center text-sm text-muted-foreground">No orders found</div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
