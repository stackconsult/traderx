import { validateOrder } from "@/lib/validation/order-schema";
import type { OrderFormData } from "@/lib/validation/order-schema";

const BUYING_POWER = 50000;

const baseOrder: OrderFormData = {
  symbol: "AAPL",
  side: "buy",
  orderType: "limit",
  quantity: 10,
  price: 178.5,
  timeInForce: "day",
};

describe("validateOrder", () => {
  it("accepts a valid limit buy order", () => {
    const result = validateOrder(baseOrder, BUYING_POWER);
    expect(result.valid).toBe(true);
    expect(result.errors).toHaveLength(0);
  });

  it("rejects order with no symbol", () => {
    const result = validateOrder({ ...baseOrder, symbol: "" }, BUYING_POWER);
    expect(result.valid).toBe(false);
    expect(result.errors).toContain("Symbol is required");
  });

  it("rejects order with zero quantity", () => {
    const result = validateOrder({ ...baseOrder, quantity: 0 }, BUYING_POWER);
    expect(result.valid).toBe(false);
    expect(result.errors).toContain("Quantity must be greater than 0");
  });

  it("rejects limit order without price", () => {
    const { price: _p, ...noPrice } = baseOrder;
    const result = validateOrder({ ...noPrice, price: undefined }, BUYING_POWER);
    expect(result.valid).toBe(false);
    expect(result.errors).toContain("Limit price is required");
  });

  it("rejects stop order without stopPrice", () => {
    const result = validateOrder(
      { ...baseOrder, orderType: "stop", price: undefined, stopPrice: undefined },
      BUYING_POWER
    );
    expect(result.valid).toBe(false);
    expect(result.errors).toContain("Stop price is required");
  });

  it("rejects order exceeding buying power", () => {
    const result = validateOrder(
      { ...baseOrder, quantity: 1000, price: 178.5 },
      1000 // very low buying power
    );
    expect(result.valid).toBe(false);
    expect(result.errors).toContain("Insufficient buying power");
  });

  it("accepts market order without price", () => {
    const result = validateOrder(
      { ...baseOrder, orderType: "market", price: undefined },
      BUYING_POWER
    );
    expect(result.valid).toBe(true);
  });

  it("accepts sell order regardless of buying power", () => {
    const result = validateOrder(
      { ...baseOrder, side: "sell", price: 178.5, quantity: 1000 },
      100 // low buying power, but it's a sell
    );
    expect(result.valid).toBe(true);
  });
});
