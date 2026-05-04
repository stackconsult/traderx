import { create } from "zustand";
import { devtools } from "zustand/middleware";
import type { MarketData, Candle } from "@/types/store";

interface MarketDataState {
  // State
  prices: Record<string, MarketData>;
  candles: Record<string, Candle[]>;
  watchlist: string[];
  selectedSymbol: string;
  selectedTimeframe: "1m" | "5m" | "15m" | "1h" | "4h" | "1d";
  isLoading: boolean;
  error: string | null;

  // Actions
  setPrice: (data: MarketData) => void;
  setPrices: (prices: Record<string, MarketData>) => void;
  setCandles: (symbol: string, candles: Candle[]) => void;
  addCandle: (symbol: string, candle: Candle) => void;
  updateLastCandle: (symbol: string, candle: Partial<Candle>) => void;
  addToWatchlist: (symbol: string) => void;
  removeFromWatchlist: (symbol: string) => void;
  setSelectedSymbol: (symbol: string) => void;
  setSelectedTimeframe: (timeframe: MarketDataState["selectedTimeframe"]) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;

  // Selectors
  getPrice: (symbol: string) => MarketData | undefined;
  getChange: (symbol: string) => number;
  getChangePercent: (symbol: string) => number;
}

export const useMarketDataStore = create<MarketDataState>()(
  devtools(
    (set, get) => ({
      // Initial state
      prices: {},
      candles: {},
      watchlist: ["AAPL", "GOOGL", "MSFT", "TSLA"],
      selectedSymbol: "AAPL",
      selectedTimeframe: "5m",
      isLoading: false,
      error: null,

      // Actions
      setPrice: (data) =>
        set((state) => ({
          prices: { ...state.prices, [data.symbol]: data },
        })),

      setPrices: (prices) => set({ prices }),

      setCandles: (symbol, candles) =>
        set((state) => ({
          candles: { ...state.candles, [symbol]: candles },
        })),

      addCandle: (symbol, candle) =>
        set((state) => {
          const existing = state.candles[symbol] || [];
          return {
            candles: {
              ...state.candles,
              [symbol]: [...existing, candle],
            },
          };
        }),

      updateLastCandle: (symbol, updates) =>
        set((state) => {
          const existing = state.candles[symbol] || [];
          if (existing.length === 0) return state;

          const lastIndex = existing.length - 1;
          const updated = [...existing];
          updated[lastIndex] = { ...updated[lastIndex], ...updates };

          return {
            candles: { ...state.candles, [symbol]: updated },
          };
        }),

      addToWatchlist: (symbol) =>
        set((state) => ({
          watchlist: state.watchlist.includes(symbol)
            ? state.watchlist
            : [...state.watchlist, symbol],
        })),

      removeFromWatchlist: (symbol) =>
        set((state) => ({
          watchlist: state.watchlist.filter((s) => s !== symbol),
        })),

      setSelectedSymbol: (symbol) => set({ selectedSymbol: symbol }),

      setSelectedTimeframe: (timeframe) => set({ selectedTimeframe: timeframe }),

      setLoading: (isLoading) => set({ isLoading }),

      setError: (error) => set({ error }),

      // Selectors
      getPrice: (symbol) => get().prices[symbol],

      getChange: (symbol) => {
        const price = get().prices[symbol];
        if (!price) return 0;
        return price.lastPrice - (price.lastPrice - (price.lastPrice * 0.01)); // Mock calculation
      },

      getChangePercent: (symbol) => {
        const change = get().getChange(symbol);
        const price = get().prices[symbol]?.lastPrice || 0;
        if (price === 0) return 0;
        return (change / price) * 100;
      },
    }),
    { name: "market-data-store" }
  )
);
