# Phase 3: Frontend Trading Dashboard - Multi-Agent Workflow Orchestration

## Executive Summary

**Phase 3 Mission**: Build a production-grade React trading dashboard that meets or exceeds HFT platform benchmarks (Robin Hood, Interactive Brokers, TradingView).

**Agent Team Assignment**:
- **Workflow Optimizer** (Lead): Reviews Phase 3, identifies gaps, establishes benchmarks
- **AI Engineer**: API integration, WebSocket handlers, state management
- **Frontend Developer**: UI/UX implementation, charting, real-time components
- **Test Writer Fixer**: Test specs, validation, deployment handoff
- **Backend Architect**: API calibration, WebSocket performance, deployment specs
- **Rapid Prototyper**: Initial UI shell, component scaffolding

---

## Part 1: Workflow Optimizer - Phase 3 Assessment

### 1.1 Phase 3 Requirements Analysis

**User Stories (HFT Platform Benchmarks)**:

| Feature | Robin Hood | Interactive Brokers | TradingView | TraderX Target |
|---------|------------|---------------------|-------------|----------------|
| Order Entry | <2s | <3s | N/A | **<1.5s** |
| Price Updates | 200ms | 100ms | 50ms | **<100ms** |
| Chart Rendering | 60fps | 30fps | 60fps | **60fps** |
| WebSocket Latency | 500ms | 200ms | 100ms | **<150ms** |
| Portfolio Load | 3s | 5s | 2s | **<2s** |
| Mobile Responsive | Yes | No | Yes | **Yes** |
| Dark Mode | Yes | No | Yes | **Yes** |

### 1.2 Gap Analysis - Current State vs Target

**Phase 2 API Status** (Ready):
- ✅ REST API endpoints (orders, positions, market data)
- ✅ JWT authentication
- ✅ WebSocket real-time feed
- ⚠️ OMS integration (stubbed, needs wiring)
- ⚠️ Market data (mock, needs exchange adapter)

**Phase 3 Frontend Gaps**:
1. **No React App**: Dashboard folder exists but minimal implementation
2. **No Charting Library**: Need TradingView-grade charts
3. **No Real-Time Layer**: WebSocket client not implemented
4. **No State Management**: Zustand/Redux store missing
5. **No Design System**: No component library (shadcn/ui needed)
6. **No API Client**: HTTP client with auth not configured

### 1.3 Critical Path Dependencies

```
Critical Path for Phase 3:

Day 1-2: Infrastructure Setup
├── shadcn/ui init (Rapid Prototyper)
├── Tailwind configuration (Frontend Developer)
├── Zustand store setup (AI Engineer)
└── API client wrapper (AI Engineer)

Day 3-5: Core Trading Components
├── Order entry form (Rapid Prototyper → Frontend Developer)
├── Position table (Rapid Prototyper → Frontend Developer)
├── Chart component (AI Engineer + Frontend Developer)
└── Real-time price ticker (AI Engineer)

Day 6-8: Dashboard Assembly
├── Portfolio summary (Frontend Developer)
├── Market data panel (Frontend Developer)
├── Signal feed integration (AI Engineer)
└── WebSocket integration (AI Engineer)

Day 9-10: Polish & Testing
├── Dark mode (Frontend Developer)
├── Mobile responsive (Frontend Developer)
├── Test suite (Test Writer Fixer)
└── Performance optimization (AI Engineer)
```

### 1.4 Risk Mitigation - Chunking Strategy

**To Avoid Terminal Strain & Build Bottlenecks**:

**Rule 1: Max 3 Files Per Commit**
```bash
# Bad: 15 files in one commit
git add packages/dashboard/src/
git commit -m "all the things"

# Good: Incremental commits
git add packages/dashboard/src/components/ui/
git commit -m "feat(ui): shadcn button, input, select components"

git add packages/dashboard/src/lib/api.ts
git commit -m "feat(api): axios client with JWT interceptor"

git add packages/dashboard/src/store/
git commit -m "feat(store): zustand order and position stores"
```

**Rule 2: Validate Each Chunk**
```bash
# Before each commit:
npm run lint       # ESLint passes
npm run type-check # TypeScript passes
npm run build      # Vite build succeeds
```

**Rule 3: Feature Flag Approach**
```typescript
// Use feature flags to merge incomplete work
const FEATURES = {
  ORDER_ENTRY: true,
  REAL_TIME_CHARTS: false, // Still in progress
  SIGNALS_PANEL: false,    // Phase 3.1
}
```

---

## Part 2: HFT Platform Benchmark Research

### 2.1 Robin Hood Analysis

**Strengths**:
- Zero-commission messaging
- Fractional share trading
- Instant deposit visualization
- Clean mobile-first UI
- Push notifications for order fills

**Implementation Requirements for TraderX**:
```typescript
// Real-time P&L display (like Robin Hood)
interface PortfolioSummaryProps {
  totalValue: number;
  dayChange: number;        // Robin Hood shows this prominently
  dayChangePercent: number;
  buyingPower: number;
  cashBalance: number;
  marginUsed: number;
}

// Color scheme (Robin Hood style)
const COLORS = {
  up: '#00C805',      // Bright green for gains
  down: '#FF5000',    // Bright red for losses
  primary: '#000000', // Black text
  background: '#FFFFFF',
  card: '#F5F5F5',
};
```

### 2.2 Interactive Brokers (IBKR) Analysis

**Strengths**:
- Professional-grade order types (iceberg, bracket, conditional)
- Real-time streaming data
- Advanced charting with studies
- Risk management visualization
- Multi-monitor support

**Implementation Requirements**:
```typescript
// Advanced order entry (like IBKR)
interface OrderFormData {
  symbol: string;
  side: 'buy' | 'sell';
  quantity: number;
  orderType: 'market' | 'limit' | 'stop' | 'stop_limit' | 'iceberg';
  limitPrice?: number;
  stopPrice?: number;
  trailingAmount?: number;
  timeInForce: 'day' | 'gtc' | 'ioc' | 'fok';
  bracketTakeProfit?: number;
  bracketStopLoss?: number;
}

// Risk visualization
interface RiskMetricsProps {
  positionDelta: number;
  gammaExposure: number;
  portfolioBeta: number;
  var95: number;  // Value at Risk
}
```

### 2.3 TradingView Analysis

**Strengths**:
- Industry-leading charting library (Lightweight Charts)
- Pine Script custom indicators
- Social features (ideas, scripts)
- Multi-timeframe sync
- Drawing tools and annotations

**Implementation Requirements**:
```typescript
// Chart configuration (TradingView-style)
interface ChartConfig {
  symbol: string;
  interval: '1m' | '5m' | '15m' | '1h' | '4h' | '1d';
  chartType: 'candles' | 'line' | 'area' | 'bars';
  studies: StudyConfig[];
  drawings: Drawing[];
}

interface StudyConfig {
  type: 'sma' | 'ema' | 'rsi' | 'macd' | 'bollinger';
  params: Record<string, number>;
  color: string;
}
```

---

## Part 3: Agent-Specific Build Guidelines

### 3.1 AI Engineer Assignments

**Primary Tasks**:
1. **API Integration Layer** (`src/lib/api.ts`)
   - Axios instance with JWT interceptor
   - Error handling with exponential backoff
   - Request/response interceptors for logging

2. **WebSocket Client** (`src/lib/websocket.ts`)
   - Native WebSocket with auto-reconnect
   - Message type handlers (market_data, order_update, position_update, signal)
   - Heartbeat/ping-pong handling

3. **Zustand Stores** (`src/store/`)
   - `authStore.ts`: JWT token, user session, login/logout
   - `orderStore.ts`: Order CRUD, order history, pending orders
   - `positionStore.ts`: Positions, P&L calculations, exposure
   - `marketDataStore.ts`: Price cache, order book, tick history
   - `signalStore.ts`: Neural signals, confidence filtering

**Code Quality Rules**:
- All API calls must have TypeScript return types
- All stores must have devtools integration
- All WebSocket handlers must have error boundaries
- Maximum 200 lines per file

### 3.2 Frontend Developer Assignments

**Primary Tasks**:
1. **Design System Setup** (`src/components/ui/`)
   - Initialize shadcn/ui with `npx shadcn-ui@latest init`
   - Install components: button, input, select, card, dialog, dropdown-menu, table, tabs, toggle
   - Configure dark mode with `next-themes` or custom solution

2. **Layout Components** (`src/components/layout/`)
   - `DashboardLayout.tsx`: Sidebar + main content area
   - `Header.tsx`: Logo, navigation, user menu, theme toggle
   - `Sidebar.tsx`: Route navigation, active state

3. **Trading Components** (`src/components/trading/`)
   - `OrderEntry.tsx`: Order form with validation, preview
   - `PositionTable.tsx`: Sortable, filterable positions
   - `MarketDataPanel.tsx`: Watchlist, price grid
   - `PortfolioSummary.tsx`: P&L cards, exposure charts

4. **Chart Integration** (`src/components/charts/`)
   - `TradingViewChart.tsx`: Lightweight Charts wrapper
   - `PriceChart.tsx`: Real-time candlestick updates
   - `VolumeChart.tsx`: Volume histogram

**UI/UX Requirements**:
- Mobile-first responsive design
- Sub-100ms interaction feedback
- Loading states for all async operations
- Error boundaries with fallback UI
- Keyboard navigation support (accessibility)

### 3.3 Test Writer Fixer Assignments

**Primary Tasks**:
1. **Test Specifications** (`src/__tests__/`)
   - `api.test.ts`: Mock API calls, test error handling
   - `stores.test.ts`: Zustand store unit tests
   - `components.test.tsx`: React Testing Library for key components
   - `websocket.test.ts`: Mock WebSocket server tests

2. **Integration Test Suite**
   - End-to-end: Login → Place Order → Verify Position
   - WebSocket: Connect → Receive Market Data → Disconnect
   - Performance: Render 1000 position rows < 500ms

3. **Deployment Handoff Spec**
   - Environment variable checklist
   - Build verification steps
   - Rollback procedures

### 3.4 Backend Architect Assignments

**Primary Tasks**:
1. **API Calibration** (`packages/oms-engine/src/api_server/`)
   - Verify JWT expiration handling
   - Rate limiting per endpoint
   - WebSocket connection limits

2. **WebSocket Performance**
   - Message batching (group updates every 100ms)
   - Connection pooling
   - Graceful degradation when overloaded

3. **Deployment Specs**
   - Docker configuration for frontend + backend
   - Nginx reverse proxy config
   - SSL/TLS certificate setup
   - Environment variable templates

### 3.5 Rapid Prototyper Assignments

**Primary Tasks**:
1. **UI Shell** (`src/app/`)
   - Initialize Next.js/Vite project structure
   - Set up routing (dashboard, orders, positions, settings)
   - Create placeholder pages for all routes

2. **Component Scaffolding**
   - Create component file templates
   - Set up Storybook (optional)
   - Create mock data generators

---

## Part 4: Systematic Build Flow (Chunking Execution)

### Phase 3.0: Infrastructure (Day 1)

**Chunk 1**: Project Setup (Rapid Prototyper)
```bash
cd packages/dashboard
npx shadcn-ui@latest init --yes --template next --base-color neutral
npm install zustand axios lightweight-charts @tanstack/react-table
```
Commit: `feat(dashboard): initialize shadcn/ui with Next.js`

**Chunk 2**: Tailwind Config (Frontend Developer)
- Configure custom colors (Robin Hood palette)
- Set up dark mode plugin
- Add custom utilities

Commit: `feat(ui): configure Tailwind with trading color palette`

**Chunk 3**: API Client (AI Engineer)
```typescript
// src/lib/api.ts
import axios from 'axios';

export const apiClient = axios.create({
  baseURL: process.env.NEXT_PUBLIC_API_URL,
  timeout: 10000,
});

apiClient.interceptors.request.use((config) => {
  const token = useAuthStore.getState().token;
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});
```
Commit: `feat(api): axios client with JWT interceptor`

### Phase 3.1: Core Components (Days 2-4)

**Chunk 4**: Auth Store + Login Page (AI Engineer)
```typescript
// src/store/authStore.ts
import { create } from 'zustand';
import { apiClient } from '@/lib/api';

interface AuthState {
  token: string | null;
  user: User | null;
  login: (credentials: LoginCredentials) => Promise<void>;
  logout: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  token: null,
  user: null,
  login: async (credentials) => {
    const { data } = await apiClient.post('/auth/login', credentials);
    set({ token: data.token, user: data.user });
  },
  logout: () => set({ token: null, user: null }),
}));
```
Commit: `feat(auth): zustand auth store with login/logout`

**Chunk 5**: Order Store + Types (AI Engineer)
```typescript
// src/store/orderStore.ts
interface OrderState {
  orders: Order[];
  pendingOrders: Order[];
  createOrder: (order: CreateOrderRequest) => Promise<void>;
  cancelOrder: (orderId: string) => Promise<void>;
}
```
Commit: `feat(orders): order store with CRUD operations`

**Chunk 6**: Order Entry Form UI (Frontend Developer)
```typescript
// src/components/trading/OrderEntry.tsx
export function OrderEntry() {
  const createOrder = useOrderStore((state) => state.createOrder);
  const [orderType, setOrderType] = useState('market');
  
  return (
    <Card>
      <form onSubmit={handleSubmit}>
        <SymbolInput />
        <SideToggle />
        <QuantityInput />
        <OrderTypeSelect onChange={setOrderType} />
        {orderType === 'limit' && <LimitPriceInput />}
        <SubmitButton />
      </form>
    </Card>
  );
}
```
Commit: `feat(ui): order entry form with validation`

**Chunk 7**: Position Table (Frontend Developer)
```typescript
// src/components/trading/PositionTable.tsx
import { useReactTable, getCoreRowModel } from '@tanstack/react-table';

export function PositionTable() {
  const positions = usePositionStore((state) => state.positions);
  const table = useReactTable({
    data: positions,
    columns: positionColumns,
    getCoreRowModel: getCoreRowModel(),
  });
  // ...
}
```
Commit: `feat(ui): sortable position table with P&L display`

### Phase 3.2: Real-Time Layer (Days 5-6)

**Chunk 8**: WebSocket Client (AI Engineer)
```typescript
// src/lib/websocket.ts
export class TradingWebSocket {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  
  connect(token: string) {
    this.ws = new WebSocket(`ws://api/ws?token=${token}`);
    this.ws.onmessage = this.handleMessage;
    this.ws.onclose = this.handleReconnect;
  }
  
  private handleMessage = (event: MessageEvent) => {
    const message: WsMessage = JSON.parse(event.data);
    switch (message.type) {
      case 'market_data':
        useMarketDataStore.getState().updatePrice(message);
        break;
      case 'order_update':
        useOrderStore.getState().updateOrder(message);
        break;
    }
  };
}
```
Commit: `feat(websocket): auto-reconnecting WebSocket client`

**Chunk 9**: Real-Time Price Ticker (Frontend Developer)
```typescript
// src/components/market/PriceTicker.tsx
export function PriceTicker({ symbol }: { symbol: string }) {
  const price = useMarketDataStore((state) => state.prices[symbol]);
  const prevPrice = useRef(price);
  const isUp = price > prevPrice.current;
  
  useEffect(() => {
    prevPrice.current = price;
  }, [price]);
  
  return (
    <span className={cn(
      "transition-colors duration-300",
      isUp ? "text-green-500" : "text-red-500"
    )}>
      ${price.toFixed(2)}
    </span>
  );
}
```
Commit: `feat(ui): real-time price ticker with color transitions`

### Phase 3.3: Chart Integration (Days 7-8)

**Chunk 10**: TradingView Chart Component (AI Engineer + Frontend Developer)
```typescript
// src/components/charts/TradingViewChart.tsx
import { createChart, IChartApi, CandlestickData } from 'lightweight-charts';

export function TradingViewChart({ symbol, interval }: ChartProps) {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const seriesRef = useRef<ISeriesApi<'Candlestick'> | null>(null);
  
  useEffect(() => {
    if (!chartContainerRef.current) return;
    
    chartRef.current = createChart(chartContainerRef.current, {
      width: chartContainerRef.current.clientWidth,
      height: 400,
      layout: {
        background: { color: '#1a1a1a' },
        textColor: '#d1d4dc',
      },
      grid: {
        vertLines: { color: '#2b2b2b' },
        horzLines: { color: '#2b2b2b' },
      },
    });
    
    seriesRef.current = chartRef.current.addCandlestickSeries({
      upColor: '#26a69a',
      downColor: '#ef5350',
    });
    
    // WebSocket subscription for real-time updates
    const unsubscribe = useMarketDataStore.subscribe(
      (state) => state.candles[symbol],
      (candles) => {
        seriesRef.current?.setData(candles);
      }
    );
    
    return () => {
      unsubscribe();
      chartRef.current?.remove();
    };
  }, [symbol, interval]);
  
  return <div ref={chartContainerRef} className="w-full h-[400px]" />;
}
```
Commit: `feat(charts): TradingView Lightweight Charts integration`

### Phase 3.4: Dashboard Assembly (Days 8-9)

**Chunk 11**: Portfolio Summary (Frontend Developer)
```typescript
// src/components/portfolio/PortfolioSummary.tsx
export function PortfolioSummary() {
  const summary = usePositionStore((state) => state.summary);
  const isPositive = summary.dayChange >= 0;
  
  return (
    <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
      <MetricCard
        label="Total Value"
        value={summary.totalValue}
        format="currency"
      />
      <MetricCard
        label="Day Change"
        value={summary.dayChange}
        format="currency"
        delta={isPositive ? 'positive' : 'negative'}
      />
      <MetricCard
        label="Buying Power"
        value={summary.buyingPower}
        format="currency"
      />
      <MetricCard
        label="Margin Used"
        value={summary.marginUsed}
        format="percent"
      />
    </div>
  );
}
```
Commit: `feat(ui): portfolio summary with metric cards`

**Chunk 12**: Main Dashboard Layout (Frontend Developer)
```typescript
// src/app/dashboard/page.tsx
export default function DashboardPage() {
  return (
    <DashboardLayout>
      <div className="grid grid-cols-12 gap-4">
        <div className="col-span-12 lg:col-span-8">
          <PortfolioSummary />
          <TradingViewChart symbol="AAPL" interval="1m" />
        </div>
        <div className="col-span-12 lg:col-span-4">
          <OrderEntry />
          <PositionTable />
          <SignalFeed />
        </div>
      </div>
    </DashboardLayout>
  );
}
```
Commit: `feat(dashboard): main dashboard layout with grid system`

### Phase 3.5: Polish & Testing (Day 10)

**Chunk 13**: Dark Mode (Frontend Developer)
```typescript
// src/components/theme-provider.tsx
export function ThemeProvider({ children }: { children: React.ReactNode }) {
  return (
    <ThemeContext.Provider value={{ theme, toggleTheme }}>
      <div className={theme}>{children}</div>
    </ThemeContext.Provider>
  );
}
```
Commit: `feat(ui): dark mode toggle with persistence`

**Chunk 14**: Mobile Responsive (Frontend Developer)
```typescript
// Responsive adjustments
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
  {/* Metrics collapse to single column on mobile */}
</div>
```
Commit: `feat(ui): mobile responsive layout adjustments`

**Chunk 15**: Test Suite (Test Writer Fixer)
```typescript
// src/__tests__/order-entry.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { OrderEntry } from '@/components/trading/OrderEntry';

describe('OrderEntry', () => {
  it('submits order with valid data', async () => {
    render(<OrderEntry />);
    
    await userEvent.type(screen.getByLabelText('Symbol'), 'AAPL');
    await userEvent.type(screen.getByLabelText('Quantity'), '100');
    await userEvent.click(screen.getByRole('button', { name: /buy/i }));
    
    expect(await screen.findByText('Order Submitted')).toBeInTheDocument();
  });
});
```
Commit: `test(dashboard): order entry and position table tests`

---

## Part 5: Deployment Handoff Spec

### 5.1 Environment Variables

```bash
# .env.local (frontend)
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_WS_URL=ws://localhost:8080/ws

# .env (backend)
JWT_SECRET=your-secret-key
API_PORT=8080
OMS_MODE=paper  # paper | live
REDIS_URL=redis://localhost:6379
```

### 5.2 Build Verification Checklist

- [ ] `npm run build` completes without errors
- [ ] `npm run type-check` passes
- [ ] `npm run lint` passes
- [ ] `npm run test` passes
- [ ] Docker build succeeds
- [ ] API health check responds 200
- [ ] WebSocket connects and receives messages

### 5.3 Deployment Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Nginx (SSL)                          │
│              ┌──────────┬──────────┐                   │
│              │          │          │                   │
│              ▼          ▼          ▼                   │
│         ┌────────┐ ┌────────┐ ┌────────┐               │
│         │ Next.js│ │ API    │ │ WS     │               │
│         │ 3000   │ │ 8080   │ │ 8080   │               │
│         └────────┘ └────────┘ └────────┘               │
│                          │                              │
│                          ▼                              │
│                    ┌──────────┐                        │
│                    │  OMS     │                        │
│                    │ Engine   │                        │
│                    └──────────┘                        │
└─────────────────────────────────────────────────────────┘
```

---

## Part 6: Updated Roadmap Checklist

### Completed ✅

- [x] **Phase 0**: Discovery & Planning
  - [x] Repository structure analysis
  - [x] Sentrux quality gate setup
  - [x] Phase 1 & 2 specs created

- [x] **Phase 1**: Fix Foundation
  - [x] Fix 22 cross_market test errors
  - [x] Fix 26 cargo warnings
  - [x] Library compiles with 0 errors (`cargo check --lib`)
  - [x] Committed: `1d46782`

- [x] **Phase 2**: Backend API Server
  - [x] API server module structure
  - [x] REST endpoints (orders, positions, market data, signals)
  - [x] JWT authentication middleware
  - [x] WebSocket real-time feed
  - [x] Error handling + types
  - [x] Committed: `ae3a2d7`

### In Progress 🔄

- [ ] **Phase 3**: Frontend Trading Dashboard
  - [ ] 3.0: Infrastructure (shadcn/ui, stores, API client)
  - [ ] 3.1: Core Components (order entry, position table)
  - [ ] 3.2: Real-Time Layer (WebSocket, price ticker)
  - [ ] 3.3: Chart Integration (TradingView charts)
  - [ ] 3.4: Dashboard Assembly (layout, portfolio summary)
  - [ ] 3.5: Polish & Testing (dark mode, mobile, tests)

### Next ⏭️

- [ ] **Phase 4**: Integration & E2E Testing
  - [ ] Frontend → API → OMS end-to-end
  - [ ] Paper trading workflow
  - [ ] Performance benchmarks

- [ ] **Phase 5**: Production Deployment
  - [ ] Docker containerization
  - [ ] Kubernetes manifests
  - [ ] Monitoring & alerting

---

## Part 7: Agent Coordination Summary

### Workflow Optimizer (You)
- ✅ Reviewed Phase 3 requirements
- ✅ Established HFT platform benchmarks
- ✅ Identified gaps and dependencies
- ✅ Created systematic chunking strategy
- ✅ Updated roadmap checklist

### AI Engineer Tasks
1. API client with JWT interceptor
2. Zustand stores (auth, orders, positions, market data, signals)
3. WebSocket client with auto-reconnect
4. Chart data integration

### Frontend Developer Tasks
1. shadcn/ui component setup
2. Dashboard layout (sidebar, header, grid)
3. Trading components (order entry, position table, market data)
4. Chart UI components
5. Dark mode + mobile responsive

### Test Writer Fixer Tasks
1. API client unit tests
2. Store unit tests
3. Component integration tests
4. WebSocket mock tests
5. Deployment handoff spec

### Backend Architect Tasks
1. API calibration (rate limiting, JWT expiry)
2. WebSocket performance optimization
3. Docker + deployment specs

### Rapid Prototyper Tasks
1. Project scaffolding
2. Route setup
3. Mock data generators
4. Component templates

---

## Execution Ready

**Status**: Phase 3 workflow orchestration complete. All agents have clear assignments with chunked execution plan.

**Next Action**: Begin Phase 3.0 Infrastructure setup with Rapid Prototyper initializing shadcn/ui project.
