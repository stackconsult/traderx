import type { MarketData, Candle, Order, Position, PortfolioSummary, Signal, User } from "./store";

export interface AuthState {
  token: string | null;
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  setToken: (token: string) => void;
  setUser: (user: User) => void;
  login: (token: string, user: User) => void;
  logout: () => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  clearError: () => void;
}

export interface OrderState {
  orders: Order[];
  pendingOrders: Order[];
  selectedOrder: Order | null;
  isLoading: boolean;
  error: string | null;
  setOrders: (orders: Order[]) => void;
  addOrder: (order: Order) => void;
  updateOrder: (orderId: string, updates: Partial<Order>) => void;
  removeOrder: (orderId: string) => void;
  setSelectedOrder: (order: Order | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  getOrderById: (orderId: string) => Order | undefined;
  getOrdersBySymbol: (symbol: string) => Order[];
  getOrdersByStatus: (status: Order["status"]) => Order[];
}

export interface PositionState {
  positions: Position[];
  summary: PortfolioSummary;
  selectedSymbol: string | null;
  isLoading: boolean;
  error: string | null;
  setPositions: (positions: Position[]) => void;
  updatePosition: (symbol: string, updates: Partial<Position>) => void;
  addPosition: (position: Position) => void;
  removePosition: (symbol: string) => void;
  setSummary: (summary: PortfolioSummary) => void;
  updateMarketPrice: (symbol: string, price: number) => void;
  setSelectedSymbol: (symbol: string | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  getPosition: (symbol: string) => Position | undefined;
  getTotalPnl: () => number;
}

export type Timeframe = "1m" | "5m" | "15m" | "1h" | "4h" | "1d";

export interface MarketDataState {
  prices: Record<string, MarketData>;
  candles: Record<string, Candle[]>;
  watchlist: string[];
  selectedSymbol: string;
  selectedTimeframe: Timeframe;
  isLoading: boolean;
  error: string | null;
  setPrice: (data: MarketData) => void;
  setPrices: (prices: Record<string, MarketData>) => void;
  setCandles: (symbol: string, candles: Candle[]) => void;
  addCandle: (symbol: string, candle: Candle) => void;
  updateLastCandle: (symbol: string, candle: Partial<Candle>) => void;
  addToWatchlist: (symbol: string) => void;
  removeFromWatchlist: (symbol: string) => void;
  setSelectedSymbol: (symbol: string) => void;
  setSelectedTimeframe: (timeframe: Timeframe) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  getPrice: (symbol: string) => MarketData | undefined;
  getChange: (symbol: string) => number;
  getChangePercent: (symbol: string) => number;
}

export interface SignalState {
  signals: Signal[];
  filteredSignals: Signal[];
  minConfidence: number;
  selectedStrategies: string[];
  isLoading: boolean;
  error: string | null;
  setSignals: (signals: Signal[]) => void;
  addSignal: (signal: Signal) => void;
  removeSignal: (signalId: string) => void;
  setMinConfidence: (confidence: number) => void;
  setSelectedStrategies: (strategies: string[]) => void;
  filterSignals: () => void;
  clearSignals: () => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  getSignalsBySymbol: (symbol: string) => Signal[];
  getSignalsByStrategy: (strategy: string) => Signal[];
  getLatestSignal: (symbol: string) => Signal | undefined;
}
