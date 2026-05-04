import { create } from "zustand";
import { devtools } from "zustand/middleware";
import type { Signal } from "@/types/store";

interface SignalState {
  // State
  signals: Signal[];
  filteredSignals: Signal[];
  minConfidence: number;
  selectedStrategies: string[];
  isLoading: boolean;
  error: string | null;

  // Actions
  setSignals: (signals: Signal[]) => void;
  addSignal: (signal: Signal) => void;
  removeSignal: (signalId: string) => void;
  setMinConfidence: (confidence: number) => void;
  setSelectedStrategies: (strategies: string[]) => void;
  filterSignals: () => void;
  clearSignals: () => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;

  // Selectors
  getSignalsBySymbol: (symbol: string) => Signal[];
  getSignalsByStrategy: (strategy: string) => Signal[];
  getLatestSignal: (symbol: string) => Signal | undefined;
}

export const useSignalStore = create<SignalState>()(
  devtools(
    (set, get) => ({
      // Initial state
      signals: [],
      filteredSignals: [],
      minConfidence: 0.7,
      selectedStrategies: [],
      isLoading: false,
      error: null,

      // Actions
      setSignals: (signals) => {
        set({ signals });
        get().filterSignals();
      },

      addSignal: (signal) =>
        set((state) => {
          const newSignals = [signal, ...state.signals].slice(0, 100); // Keep last 100
          return { signals: newSignals };
        }),

      removeSignal: (signalId) =>
        set((state) => ({
          signals: state.signals.filter((s) => s.signalId !== signalId),
        })),

      setMinConfidence: (minConfidence) => {
        set({ minConfidence });
        get().filterSignals();
      },

      setSelectedStrategies: (selectedStrategies) => {
        set({ selectedStrategies });
        get().filterSignals();
      },

      filterSignals: () =>
        set((state) => ({
          filteredSignals: state.signals.filter((signal) => {
            const confidenceMatch = signal.confidence >= state.minConfidence;
            const strategyMatch =
              state.selectedStrategies.length === 0 ||
              state.selectedStrategies.includes(signal.strategy);
            return confidenceMatch && strategyMatch;
          }),
        })),

      clearSignals: () => set({ signals: [], filteredSignals: [] }),

      setLoading: (isLoading) => set({ isLoading }),

      setError: (error) => set({ error }),

      // Selectors
      getSignalsBySymbol: (symbol) =>
        get().signals.filter((s) => s.symbol === symbol),

      getSignalsByStrategy: (strategy) =>
        get().signals.filter((s) => s.strategy === strategy),

      getLatestSignal: (symbol) =>
        get()
          .signals.filter((s) => s.symbol === symbol)
          .sort(
            (a, b) =>
              new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
          )[0],
    }),
    { name: "signal-store" }
  )
);
