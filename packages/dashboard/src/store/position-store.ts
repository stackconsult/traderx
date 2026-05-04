import { create } from "zustand";
import { devtools } from "zustand/middleware";
import type { Position, PortfolioSummary } from "@/types/store";

interface PositionState {
  // State
  positions: Position[];
  summary: PortfolioSummary;
  selectedSymbol: string | null;
  isLoading: boolean;
  error: string | null;

  // Actions
  setPositions: (positions: Position[]) => void;
  updatePosition: (symbol: string, updates: Partial<Position>) => void;
  addPosition: (position: Position) => void;
  removePosition: (symbol: string) => void;
  setSummary: (summary: PortfolioSummary) => void;
  updateMarketPrice: (symbol: string, price: number) => void;
  setSelectedSymbol: (symbol: string | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;

  // Computed
  getPosition: (symbol: string) => Position | undefined;
  getTotalPnl: () => number;
}

const defaultSummary: PortfolioSummary = {
  totalValue: 0,
  cashBalance: 0,
  totalExposure: 0,
  totalUnrealizedPnl: 0,
  totalRealizedPnl: 0,
  buyingPower: 0,
  marginUsed: 0,
};

export const usePositionStore = create<PositionState>()(
  devtools(
    (set, get) => ({
      // Initial state
      positions: [],
      summary: defaultSummary,
      selectedSymbol: null,
      isLoading: false,
      error: null,

      // Actions
      setPositions: (positions) =>
        set((state) => {
          const totalValue = positions.reduce(
            (sum, p) => sum + p.marketValue,
            state.summary.cashBalance
          );
          const totalUnrealizedPnl = positions.reduce(
            (sum, p) => sum + p.unrealizedPnl,
            0
          );
          const totalExposure = positions.reduce(
            (sum, p) => sum + Math.abs(p.marketValue),
            0
          );

          return {
            positions,
            summary: {
              ...state.summary,
              totalValue,
              totalUnrealizedPnl,
              totalExposure,
            },
          };
        }),

      updatePosition: (symbol, updates) =>
        set((state) => ({
          positions: state.positions.map((p) =>
            p.symbol === symbol ? { ...p, ...updates } : p
          ),
        })),

      addPosition: (position) =>
        set((state) => ({
          positions: [...state.positions, position],
        })),

      removePosition: (symbol) =>
        set((state) => ({
          positions: state.positions.filter((p) => p.symbol !== symbol),
        })),

      setSummary: (summary) => set({ summary }),

      updateMarketPrice: (symbol, price) =>
        set((state) => {
          const positions = state.positions.map((p) => {
            if (p.symbol !== symbol) return p;

            const marketValue = p.quantity * price;
            const unrealizedPnl =
              p.side === "long"
                ? marketValue - p.quantity * p.avgEntryPrice
                : p.quantity * p.avgEntryPrice - marketValue;

            return {
              ...p,
              marketPrice: price,
              marketValue,
              unrealizedPnl,
            };
          });

          const totalUnrealizedPnl = positions.reduce(
            (sum, p) => sum + p.unrealizedPnl,
            0
          );

          return {
            positions,
            summary: {
              ...state.summary,
              totalUnrealizedPnl,
            },
          };
        }),

      setSelectedSymbol: (symbol) => set({ selectedSymbol: symbol }),

      setLoading: (isLoading) => set({ isLoading }),

      setError: (error) => set({ error }),

      // Selectors
      getPosition: (symbol) =>
        get().positions.find((p) => p.symbol === symbol),

      getTotalPnl: () =>
        get().positions.reduce((sum, p) => sum + p.unrealizedPnl, 0),
    }),
    { name: "position-store" }
  )
);
