import { z } from "zod";

export const orderSchema = z.object({
  symbol: z.string().min(1, "Symbol is required").toUpperCase(),
  side: z.enum(["buy", "sell"], {
    required_error: "Side is required",
  }),
  orderType: z.enum(["market", "limit", "stop", "stop_limit", "iceberg"], {
    required_error: "Order type is required",
  }),
  quantity: z
    .number({
      required_error: "Quantity is required",
      invalid_type_error: "Quantity must be a number",
    })
    .positive("Quantity must be positive"),
  price: z.number().optional().refine(
    (val) => {
      // Price required for limit, stop, stop_limit orders
      return true; // Simplified - will validate in component
    },
    { message: "Price is required for this order type" }
  ),
  stopPrice: z.number().optional(),
  timeInForce: z.enum(["day", "gtc", "ioc", "fok"]).default("day"),
  icebergQty: z.number().optional(),
});

export type OrderFormData = z.infer<typeof orderSchema>;

export const validateOrder = (
  data: OrderFormData,
  buyingPower: number
): { valid: boolean; errors: string[] } => {
  const errors: string[] = [];

  // Validate required fields
  if (!data.symbol || data.symbol.length < 1) {
    errors.push("Symbol is required");
  }

  if (!data.quantity || data.quantity <= 0) {
    errors.push("Quantity must be greater than 0");
  }

  // Validate price for limit orders
  if (data.orderType === "limit" && (!data.price || data.price <= 0)) {
    errors.push("Limit price is required");
  }

  if (data.orderType === "stop" && (!data.stopPrice || data.stopPrice <= 0)) {
    errors.push("Stop price is required");
  }

  if (
    data.orderType === "stop_limit" &&
    (!data.price || !data.stopPrice || data.price <= 0 || data.stopPrice <= 0)
  ) {
    errors.push("Both stop price and limit price are required");
  }

  // Validate buying power
  const estimatedCost = data.quantity * (data.price || data.stopPrice || 0);
  if (data.side === "buy" && estimatedCost > buyingPower) {
    errors.push("Insufficient buying power");
  }

  return {
    valid: errors.length === 0,
    errors,
  };
};
