import { useOrderStore } from "@/store/order-store";
import type { Order } from "@/types/store";

const mockOrder: Order = {
  orderId: "test-order-1",
  status: "pending",
  symbol: "AAPL",
  side: "buy",
  orderType: "limit",
  quantity: 100,
  filledQuantity: 0,
  price: 178.5,
  timeInForce: "day",
  createdAt: new Date().toISOString(),
};

describe("order-store", () => {
  beforeEach(() => {
    useOrderStore.setState({
      orders: [],
      pendingOrders: [],
      selectedOrder: null,
      isLoading: false,
      error: null,
    });
  });

  it("adds an order", () => {
    useOrderStore.getState().addOrder(mockOrder);
    expect(useOrderStore.getState().orders).toHaveLength(1);
    expect(useOrderStore.getState().orders[0].orderId).toBe("test-order-1");
  });

  it("tracks pending orders correctly", () => {
    useOrderStore.getState().addOrder(mockOrder);
    expect(useOrderStore.getState().pendingOrders).toHaveLength(1);
  });

  it("updates an order status", () => {
    useOrderStore.getState().addOrder(mockOrder);
    useOrderStore.getState().updateOrder("test-order-1", { status: "filled", filledQuantity: 100 });
    const updated = useOrderStore.getState().orders[0];
    expect(updated.status).toBe("filled");
    expect(updated.filledQuantity).toBe(100);
  });

  it("removes pending from pendingOrders when filled", () => {
    useOrderStore.getState().addOrder(mockOrder);
    useOrderStore.getState().updateOrder("test-order-1", { status: "filled" });
    expect(useOrderStore.getState().pendingOrders).toHaveLength(0);
  });

  it("removes an order", () => {
    useOrderStore.getState().addOrder(mockOrder);
    useOrderStore.getState().removeOrder("test-order-1");
    expect(useOrderStore.getState().orders).toHaveLength(0);
  });

  it("clears selectedOrder when removed", () => {
    useOrderStore.getState().addOrder(mockOrder);
    useOrderStore.getState().setSelectedOrder(mockOrder);
    useOrderStore.getState().removeOrder("test-order-1");
    expect(useOrderStore.getState().selectedOrder).toBeNull();
  });

  it("getOrdersBySymbol returns correct orders", () => {
    useOrderStore.getState().addOrder(mockOrder);
    useOrderStore.getState().addOrder({ ...mockOrder, orderId: "test-order-2", symbol: "GOOGL" });
    const appleOrders = useOrderStore.getState().getOrdersBySymbol("AAPL");
    expect(appleOrders).toHaveLength(1);
    expect(appleOrders[0].symbol).toBe("AAPL");
  });

  it("getOrdersByStatus returns correct orders", () => {
    useOrderStore.getState().addOrder(mockOrder);
    useOrderStore.getState().addOrder({ ...mockOrder, orderId: "test-order-2", status: "filled" });
    const pending = useOrderStore.getState().getOrdersByStatus("pending");
    expect(pending).toHaveLength(1);
  });

  it("sets loading state", () => {
    useOrderStore.getState().setLoading(true);
    expect(useOrderStore.getState().isLoading).toBe(true);
  });

  it("sets and clears error", () => {
    useOrderStore.getState().setError("Something failed");
    expect(useOrderStore.getState().error).toBe("Something failed");
    useOrderStore.getState().setError(null);
    expect(useOrderStore.getState().error).toBeNull();
  });
});
