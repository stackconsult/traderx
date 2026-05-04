"use client";

import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { cn, formatCurrency } from "@/lib/utils";
import { ArrowUp, ArrowDown, AlertCircle } from "lucide-react";
import { validateOrder } from "@/lib/validation/order-schema";
import type { OrderFormData } from "@/lib/validation/order-schema";
import { api } from "@/lib/api";
import { usePositionStore, useOrderStore } from "@/store";
import type { PositionState, OrderState } from "@/types/store-states";
import type { Order } from "@/types/store";
import { toast } from "@/hooks/use-toast";

export function OrderEntry() {
  const buyingPower = usePositionStore((state: PositionState) => state.summary.buyingPower);
  const addOrder = useOrderStore((state: OrderState) => state.addOrder);
  const [side, setSide] = useState<"buy" | "sell">("buy");
  const [orderType, setOrderType] = useState<OrderFormData["orderType"]>("market");
  const [symbol, setSymbol] = useState("");
  const [quantity, setQuantity] = useState("");
  const [price, setPrice] = useState("");
  const [stopPrice, setStopPrice] = useState("");
  const [timeInForce, setTimeInForce] = useState<OrderFormData["timeInForce"]>("day");
  const [errors, setErrors] = useState<string[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrors([]);

    const orderData: OrderFormData = {
      symbol: symbol.toUpperCase(),
      side,
      orderType,
      quantity: parseFloat(quantity) || 0,
      price: price ? parseFloat(price) : undefined,
      stopPrice: stopPrice ? parseFloat(stopPrice) : undefined,
      timeInForce,
    };

    const effectiveBuyingPower = buyingPower > 0 ? buyingPower : 45000;
    const validation = validateOrder(orderData, effectiveBuyingPower);

    if (!validation.valid) {
      setErrors(validation.errors);
      return;
    }

    setIsSubmitting(true);

    try {
      const order = await api.post<Order>("/api/orders", orderData);
      addOrder(order);
      toast({
        title: "Order submitted",
        description: `${side.toUpperCase()} ${quantity} ${symbol.toUpperCase()} — ${orderType}`,
        variant: "success",
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : "Order submission failed";
      setErrors([message]);
      toast({ title: "Order rejected", description: message, variant: "destructive" });
    } finally {
      setIsSubmitting(false);
    }

    // Reset form
    setSymbol("");
    setQuantity("");
    setPrice("");
    setStopPrice("");
  };

  // Calculate estimated cost
  const effectiveBuyingPower = buyingPower > 0 ? buyingPower : 45000;
  const estimatedPrice = orderType === "market" ? 150 : parseFloat(price) || 0;
  const qty = parseFloat(quantity) || 0;
  const estimatedCost = qty * estimatedPrice;
  const isLargeOrder = estimatedCost > effectiveBuyingPower * 0.1;

  const showPrice = orderType === "limit" || orderType === "stop_limit";
  const showStopPrice = orderType === "stop" || orderType === "stop_limit";

  return (
    <Card>
      <CardHeader>
        <CardTitle>Order Entry</CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Side Toggle */}
          <div className="grid grid-cols-2 gap-2">
            <Button
              type="button"
              variant={side === "buy" ? "default" : "outline"}
              className={cn(
                side === "buy" && "bg-up hover:bg-up/90"
              )}
              onClick={() => setSide("buy")}
            >
              <ArrowUp className="mr-2 h-4 w-4" />
              Buy
            </Button>
            <Button
              type="button"
              variant={side === "sell" ? "default" : "outline"}
              className={cn(
                side === "sell" && "bg-down hover:bg-down/90"
              )}
              onClick={() => setSide("sell")}
            >
              <ArrowDown className="mr-2 h-4 w-4" />
              Sell
            </Button>
          </div>

          {/* Symbol */}
          <div className="space-y-2">
            <Label htmlFor="symbol">Symbol</Label>
            <Input
              id="symbol"
              placeholder="e.g. AAPL"
              value={symbol}
              onChange={(e) => setSymbol(e.target.value.toUpperCase())}
              required
            />
          </div>

          {/* Order Type */}
          <div className="space-y-2">
            <Label>Order Type</Label>
            <Select value={orderType} onValueChange={(v) => setOrderType(v as OrderFormData["orderType"])}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="market">Market</SelectItem>
                <SelectItem value="limit">Limit</SelectItem>
                <SelectItem value="stop">Stop</SelectItem>
                <SelectItem value="stop_limit">Stop Limit</SelectItem>
                <SelectItem value="iceberg">Iceberg</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Quantity */}
          <div className="space-y-2">
            <Label htmlFor="quantity">Quantity</Label>
            <Input
              id="quantity"
              type="number"
              min="1"
              step="1"
              placeholder="100"
              value={quantity}
              onChange={(e) => setQuantity(e.target.value)}
              required
            />
          </div>

          {/* Price (conditional) */}
          {showPrice && (
            <div className="space-y-2">
              <Label htmlFor="price">Limit Price</Label>
              <Input
                id="price"
                type="number"
                min="0.01"
                step="0.01"
                placeholder="150.00"
                value={price}
                onChange={(e) => setPrice(e.target.value)}
                required={showPrice}
              />
            </div>
          )}

          {/* Stop Price (conditional) */}
          {showStopPrice && (
            <div className="space-y-2">
              <Label htmlFor="stopPrice">Stop Price</Label>
              <Input
                id="stopPrice"
                type="number"
                min="0.01"
                step="0.01"
                placeholder="145.00"
                value={stopPrice}
                onChange={(e) => setStopPrice(e.target.value)}
                required={showStopPrice}
              />
            </div>
          )}

          {/* Time in Force */}
          <div className="space-y-2">
            <Label>Time in Force</Label>
            <Select value={timeInForce} onValueChange={(v) => setTimeInForce(v as OrderFormData["timeInForce"])}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="day">Day</SelectItem>
                <SelectItem value="gtc">Good Till Cancelled</SelectItem>
                <SelectItem value="ioc">Immediate or Cancel</SelectItem>
                <SelectItem value="fok">Fill or Kill</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Errors */}
          {errors.length > 0 && (
            <div className="rounded-md bg-destructive/10 p-3 text-sm">
              <div className="flex items-center gap-2 text-destructive">
                <AlertCircle className="h-4 w-4" />
                <span className="font-medium">Please fix the following:</span>
              </div>
              <ul className="mt-1 list-inside list-disc text-destructive">
                {errors.map((error, i) => (
                  <li key={i}>{error}</li>
                ))}
              </ul>
            </div>
          )}

          {/* Order Preview */}
          {qty > 0 && (
            <div className="rounded-md bg-muted p-3 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Est. {side === "buy" ? "Cost" : "Proceeds"}</span>
                <span className="font-medium">{formatCurrency(estimatedCost)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Buying Power</span>
                <span className="font-medium">{formatCurrency(effectiveBuyingPower)}</span>
              </div>
              {isLargeOrder && (
                <p className="mt-2 text-xs text-yellow-600">
                  Warning: This order exceeds 10% of your buying power
                </p>
              )}
            </div>
          )}

          {/* Submit */}
          <Button
            type="submit"
            className={cn(
              "w-full",
              side === "buy" ? "bg-up hover:bg-up/90" : "bg-down hover:bg-down/90"
            )}
            disabled={isSubmitting}
          >
            {isSubmitting ? "Submitting..." : `${side === "buy" ? "Buy" : "Sell"} ${symbol || "Symbol"}`}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}
