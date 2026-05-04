// Common types for all stores

export interface User {
  id: string;
  username: string;
  email: string;
}

export interface Order {
  orderId: string;
  status: "pending" | "submitted" | "filled" | "cancelled" | "rejected";
  symbol: string;
  side: "buy" | "sell";
  orderType: "market" | "limit" | "stop" | "stop_limit" | "iceberg";
  quantity: number;
  filledQuantity: number;
  price?: number;
  avgPrice?: number;
  timeInForce: "day" | "gtc" | "ioc" | "fok";
  createdAt: string;
  updatedAt?: string;
}

export interface Position {
  symbol: string;
  quantity: number;
  avgEntryPrice: number;
  marketPrice: number;
  marketValue: number;
  unrealizedPnl: number;
  realizedPnl: number;
  side: "long" | "short";
}

export interface MarketData {
  symbol: string;
  bid: number;
  ask: number;
  lastPrice: number;
  volume24h: number;
  timestamp: string;
}

export interface Candle {
  time: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

export interface Signal {
  signalId: string;
  symbol: string;
  direction: "long" | "short" | "neutral";
  confidence: number;
  strategy: string;
  timestamp: string;
  metadata?: Record<string, unknown>;
}

export interface PortfolioSummary {
  totalValue: number;
  cashBalance: number;
  totalExposure: number;
  totalUnrealizedPnl: number;
  totalRealizedPnl: number;
  buyingPower: number;
  marginUsed: number;
}

// WebSocket message types
export interface WsMessage {
  type: "market_data" | "order_update" | "position_update" | "signal" | "error" | "connected";
}

export interface MarketDataMessage extends WsMessage {
  type: "market_data";
  symbol: string;
  bid: number;
  ask: number;
  last: number;
  volume?: number;
  timestamp: string;
}

export interface OrderUpdateMessage extends WsMessage {
  type: "order_update";
  orderId: string;
  status: Order["status"];
  filledQty: number;
  avgPrice?: number;
}

export interface PositionUpdateMessage extends WsMessage {
  type: "position_update";
  symbol: string;
  quantity: number;
  unrealizedPnl: number;
  marketPrice: number;
}

export interface SignalMessage extends WsMessage {
  type: "signal";
  signalId: string;
  symbol: string;
  direction: Signal["direction"];
  confidence: number;
  strategy: string;
}
