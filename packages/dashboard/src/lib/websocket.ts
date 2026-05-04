import { useAuthStore } from "@/store/auth-store";
import { useMarketDataStore } from "@/store/market-data-store";
import { useOrderStore } from "@/store/order-store";
import { usePositionStore } from "@/store/position-store";
import { useSignalStore } from "@/store/signal-store";
import { toast } from "@/hooks/use-toast";
import type {
  WsMessage,
  MarketDataMessage,
  OrderUpdateMessage,
  PositionUpdateMessage,
  SignalMessage,
} from "@/types/store";

export class TradingWebSocket {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;
  private heartbeatInterval: NodeJS.Timeout | null = null;
  private messageQueue: string[] = [];
  private isConnecting = false;

  constructor(private url: string) { }

  connect(): void {
    if (this.isConnecting || this.ws?.readyState === WebSocket.OPEN) {
      return;
    }

    this.isConnecting = true;
    const token = useAuthStore.getState().token;

    if (!token) {
      console.error("No auth token available");
      this.isConnecting = false;
      return;
    }

    try {
      this.ws = new WebSocket(`${this.url}?token=${token}`);

      this.ws.onopen = () => {
        console.log("WebSocket connected");
        this.isConnecting = false;
        this.reconnectAttempts = 0;
        this.startHeartbeat();
        this.flushMessageQueue();
      };

      this.ws.onmessage = (event) => {
        this.handleMessage(event.data);
      };

      this.ws.onclose = () => {
        console.log("WebSocket closed");
        this.stopHeartbeat();
        this.attemptReconnect();
      };

      this.ws.onerror = (error) => {
        console.error("WebSocket error:", error);
        this.isConnecting = false;
      };
    } catch (error) {
      console.error("WebSocket connection error:", error);
      this.isConnecting = false;
      this.attemptReconnect();
    }
  }

  disconnect(): void {
    this.stopHeartbeat();
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  private handleMessage(data: string): void {
    try {
      const message: WsMessage = JSON.parse(data);

      switch (message.type) {
        case "market_data":
          this.handleMarketData(message as MarketDataMessage);
          break;
        case "order_update":
          this.handleOrderUpdate(message as OrderUpdateMessage);
          break;
        case "position_update":
          this.handlePositionUpdate(message as PositionUpdateMessage);
          break;
        case "signal":
          this.handleSignal(message as SignalMessage);
          break;
        case "connected":
          console.log("WebSocket session established");
          break;
        case "error":
          console.error("WebSocket error message:", message);
          break;
        default:
          console.warn("Unknown message type:", message.type);
      }
    } catch (error) {
      console.error("Failed to parse WebSocket message:", error);
    }
  }

  private handleMarketData(message: MarketDataMessage): void {
    useMarketDataStore.getState().setPrice({
      symbol: message.symbol,
      bid: message.bid,
      ask: message.ask,
      lastPrice: message.last,
      price: message.last,
      change: message.change || 0,
      changePercent: message.changePercent || 0,
      volume: message.volume || 0,
      volume24h: message.volume || 0,
      timestamp: message.timestamp,
    });
  }

  private handleOrderUpdate(message: OrderUpdateMessage): void {
    useOrderStore.getState().updateOrder(message.orderId, {
      status: message.status,
      filledQuantity: message.filledQty,
      avgPrice: message.avgPrice,
    });
    if (message.status === "filled") {
      toast({
        title: "Order filled",
        description: `Order ${message.orderId.slice(0, 8)} filled at ${message.avgPrice ?? "market"}`,
        variant: "success",
      });
    } else if (message.status === "rejected") {
      toast({
        title: "Order rejected",
        description: `Order ${message.orderId.slice(0, 8)} was rejected`,
        variant: "destructive",
      });
    }
  }

  private handlePositionUpdate(message: PositionUpdateMessage): void {
    usePositionStore.getState().updateMarketPrice(
      message.symbol,
      message.marketPrice
    );
  }

  private handleSignal(message: SignalMessage): void {
    useSignalStore.getState().addSignal({
      signalId: message.signalId,
      symbol: message.symbol,
      direction: message.direction,
      confidence: message.confidence,
      strategy: message.strategy,
      timestamp: new Date().toISOString(),
    });
  }

  private attemptReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error("Max reconnect attempts reached");
      return;
    }

    this.reconnectAttempts++;
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);

    console.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);

    setTimeout(() => {
      this.connect();
    }, delay);
  }

  private startHeartbeat(): void {
    this.heartbeatInterval = setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.ws.send(JSON.stringify({ type: "ping" }));
      }
    }, 30000); // 30 seconds
  }

  private stopHeartbeat(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }
  }

  private flushMessageQueue(): void {
    while (this.messageQueue.length > 0) {
      const message = this.messageQueue.shift();
      if (message) {
        this.send(message);
      }
    }
  }

  send(message: string): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(message);
    } else {
      this.messageQueue.push(message);
    }
  }

  subscribe(symbols: string[]): void {
    this.send(
      JSON.stringify({
        type: "subscribe",
        symbols,
      })
    );
  }

  unsubscribe(symbols: string[]): void {
    this.send(
      JSON.stringify({
        type: "unsubscribe",
        symbols,
      })
    );
  }
}

// Singleton instance
let wsInstance: TradingWebSocket | null = null;

export function getWebSocket(url?: string): TradingWebSocket {
  if (!wsInstance) {
    wsInstance = new TradingWebSocket(
      url || process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:8080/ws"
    );
  }
  return wsInstance;
}

export function closeWebSocket(): void {
  if (wsInstance) {
    wsInstance.disconnect();
    wsInstance = null;
  }
}
