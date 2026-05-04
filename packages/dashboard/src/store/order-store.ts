import { create } from "zustand";
import { devtools } from "zustand/middleware";
import type { Order } from "@/types/store";

interface OrderState {
  // State
  orders: Order[];
  pendingOrders: Order[];
  selectedOrder: Order | null;
  isLoading: boolean;
  error: string | null;

  // Actions
  setOrders: (orders: Order[]) => void;
  addOrder: (order: Order) => void;
  updateOrder: (orderId: string, updates: Partial<Order>) => void;
  removeOrder: (orderId: string) => void;
  setSelectedOrder: (order: Order | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;

  // Selectors
  getOrderById: (orderId: string) => Order | undefined;
  getOrdersBySymbol: (symbol: string) => Order[];
  getOrdersByStatus: (status: Order["status"]) => Order[];
}

export const useOrderStore = create<OrderState>()(
  devtools(
    (set, get) => ({
      // Initial state
      orders: [],
      pendingOrders: [],
      selectedOrder: null,
      isLoading: false,
      error: null,

      // Actions
      setOrders: (orders) =>
        set({
          orders,
          pendingOrders: orders.filter((o) => o.status === "pending"),
        }),

      addOrder: (order) =>
        set((state) => ({
          orders: [order, ...state.orders],
          pendingOrders:
            order.status === "pending"
              ? [order, ...state.pendingOrders]
              : state.pendingOrders,
        })),

      updateOrder: (orderId, updates) =>
        set((state) => {
          const updatedOrders = state.orders.map((order) =>
            order.orderId === orderId ? { ...order, ...updates } : order
          );
          return {
            orders: updatedOrders,
            pendingOrders: updatedOrders.filter((o) => o.status === "pending"),
            selectedOrder:
              state.selectedOrder?.orderId === orderId
                ? { ...state.selectedOrder, ...updates }
                : state.selectedOrder,
          };
        }),

      removeOrder: (orderId) =>
        set((state) => ({
          orders: state.orders.filter((o) => o.orderId !== orderId),
          pendingOrders: state.pendingOrders.filter((o) => o.orderId !== orderId),
          selectedOrder:
            state.selectedOrder?.orderId === orderId ? null : state.selectedOrder,
        })),

      setSelectedOrder: (order) => set({ selectedOrder: order }),

      setLoading: (isLoading) => set({ isLoading }),

      setError: (error) => set({ error }),

      // Selectors
      getOrderById: (orderId) => get().orders.find((o) => o.orderId === orderId),

      getOrdersBySymbol: (symbol) =>
        get().orders.filter((o) => o.symbol === symbol),

      getOrdersByStatus: (status) =>
        get().orders.filter((o) => o.status === status),
    }),
    { name: "order-store" }
  )
);
