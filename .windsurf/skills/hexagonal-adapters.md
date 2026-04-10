# Hexagonal Liquidity Adapters

## Description
Implements unified liquidity routing across 12+ fragmented broker APIs using Hexagonal Architecture (Ports & Adapters pattern). Provides clean separation between domain logic and external integrations.

## Source
- Repository: nearshore-it/hexagonal-architecture
- Reference: https://github.com/nearshore-it/hexagonal-architecture

## Implementation Pattern

### Core Architecture
1. **Domain Port**: Abstract interface defining liquidity operations
2. **Adapters**: Concrete implementations for each broker (FIX, WebSocket, REST)
3. **Application Service**: Orchestrates multi-venue execution
4. **Domain Model**: Unified order/position representations

### Key Components

#### LiquidityPort Interface
```typescript
interface LiquidityPort {
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  submitOrder(order: UnifiedOrder): Promise<OrderResult>;
  cancelOrder(orderId: string): Promise<boolean>;
  getPositions(): Promise<Position[]>;
  getAccountInfo(): Promise<AccountInfo>;
}
```

#### Adapter Implementations
```typescript
// FIX Adapter for institutional brokers
class FIXAdapter implements LiquidityPort {
  private fixSession: FIXSession;
  
  async submitOrder(order: UnifiedOrder): Promise<OrderResult> {
    const fixMessage = this.convertToFIX(order);
    return this.fixSession.send(fixMessage);
  }
}

// WebSocket Adapter for crypto exchanges
class WebSocketAdapter implements LiquidityPort {
  private ws: WebSocket;
  
  async submitOrder(order: UnifiedOrder): Promise<OrderResult> {
    const wsMessage = this.convertToWS(order);
    this.ws.send(JSON.stringify(wsMessage));
    return this.waitForExecution();
  }
}

// REST Adapter for legacy APIs
class RESTAdapter implements LiquidityPort {
  private httpClient: HttpClient;
  
  async submitOrder(order: UnifiedOrder): Promise<OrderResult> {
    const response = await this.httpClient.post('/orders', order);
    return response.data;
  }
}
```

#### Unified Order Model
```typescript
interface UnifiedOrder {
  id: string;
  symbol: string;
  side: 'BUY' | 'SELL';
  quantity: number;
  orderType: 'MARKET' | 'LIMIT' | 'STOP';
  price?: number;
  timeInForce: 'IOC' | 'GTC' | 'FOK';
  tenantId: string;
  metadata: Record<string, any>;
}
```

### Integration Points
- **Phase 2 DeltaLag**: Consume signals for execution
- **ZK-Audit**: Log all adapter operations
- **PTP Service**: Timestamp all order submissions
- **Risk Manager**: Validate orders before routing

### Supported Venues
1. **Crypto**: Bybit, MEXC, Binance (WebSocket)
2. **Institutional**: Interactive Brokers (FIX)
3. **FX/CFD**: PrimeXM, LMAX (FIX/REST)
4. **Traditional**: Various REST APIs

### Performance Optimizations
- Connection pooling per adapter
- Async message batching
- Circuit breakers for failed venues
- Smart order routing based on latency/liquidity

### Error Handling
- Adapter-specific error translation
- Automatic failover between venues
- Retry policies with exponential backoff
- Dead letter queue for failed orders

## Success Criteria
- <10ms concurrent order submission across 5 venues
- Unified interface supports all 12+ broker APIs
- Adapter load test confirms performance targets
- Zero domain logic in adapter implementations
