# Phase 4: Next Stage Roadmap Checklist
**Prepared by**: Workflow Optimizer + Engineering Team  
**Follows**: Phase 3 completion (commit `1685aed`)  
**Status**: Ready to plan

---

## PHASE 3 COMPLETION AUDIT

### ✅ Completed This Session

| Chunk | Component | Agent | Status |
|-------|-----------|-------|--------|
| 3.0 | Infrastructure, Next.js, Tailwind, stores | Rapid Prototyper | ✅ |
| 3.1 | Axios API client, WebSocket client | AI Engineer | ✅ |
| 3.2 | Layout, Order Entry, UI primitives | Frontend Developer | ✅ |
| 3.3 | PriceTicker, Watchlist, SymbolSearch | Frontend Dev + AI Eng | ✅ |
| 3.4 | TradingViewChart (Lightweight), useChartData | AI Engineer | ✅ |
| 3.5 | SignalCard, SignalFeed, Dashboard Assembly | AI Engineer + Frontend Dev | ✅ |
| 3.6 | 23 unit tests, Jest config, CI/CD pipeline | Test Writer Fixer + DevOps | ✅ |

### 🔧 Known Tech Debt (Pre-`npm install`)
These IDE errors disappear after `npm ci` in `packages/dashboard`:
- `Cannot find module 'react'` → resolved by `@types/react` install
- `Cannot find module 'lucide-react'` → resolved by `lucide-react` install  
- `Cannot find module 'lightweight-charts'` → resolved by `lightweight-charts` install
- `Parameter 'state' implicitly has 'any'` → resolved by Zustand types with `strict` mode
- **Root cause**: Zustand store callbacks need explicit state types in strict mode — add typed selectors in Phase 4

---

## PHASE 4: BACKEND INTEGRATION & LIVE DATA

### 4.0 Pre-flight (DevOps Automator)

- [ ] **4.0.1** — Run `npm ci` in `packages/dashboard`, verify 0 TS errors
- [ ] **4.0.2** — Run `npm test` — 23 tests must pass green
- [ ] **4.0.3** — Run `npm run build` — Next.js build must succeed
- [ ] **4.0.4** — Verify `.github/workflows/dashboard.yml` triggers on PR
- [ ] **4.0.5** — Confirm OMS API server (`packages/oms-engine/src/api_server`) is running on `:8080`

**Command**:
```bash
cd packages/dashboard && npm ci && npm test && npm run build
```

---

### 4.1 Fix Zustand Type Errors (AI Engineer)
**Agent**: AI Engineer  
**Skill**: `/.ai/engineering/ai-engineer.md`  
**Guard**: No `any` in store selectors, strict Zustand generics

- [ ] Add explicit state type to all `useXxxStore((state) => ...)` selectors
- [ ] Export store state types from `src/types/store.ts`
- [ ] Add `StoreApi` typed selectors pattern
- [ ] Fix `parameter 'state' implicitly has 'any'` errors across all 5 stores

**Files**:
- `src/store/market-data-store.ts` — add `MarketDataState` type
- `src/store/order-store.ts` — add `OrderState` type
- `src/store/auth-store.ts` — add `AuthState` type
- `src/store/signal-store.ts` — add `SignalState` type
- `src/store/position-store.ts` — add `PositionState` type

---

### 4.2 Real Position Table (Frontend Developer)
**Agent**: Frontend Developer  
**Skill**: `/.ai/engineering/frontend-developer.md`  
**Guard**: Sortable columns, P&L color coding, <50ms render

- [ ] `src/components/trading/position-table.tsx` — sortable data table
- [ ] Columns: Symbol, Qty, Avg Price, Current Price, P&L $, P&L %, Market Value
- [ ] Color: positive P&L = `text-up`, negative = `text-down`
- [ ] Close position button (triggers sell order flow)
- [ ] Real-time mark price from `market-data-store`
- [ ] Wire into `positions/page.tsx` and dashboard main area

---

### 4.3 Order History Table (Frontend Developer)
**Agent**: Frontend Developer  
**Guard**: Paginated, filterable by status/symbol

- [ ] `src/components/trading/order-history-table.tsx`
- [ ] Filter: All / Pending / Filled / Cancelled
- [ ] Cancel pending order action
- [ ] Wire into `orders/page.tsx`

---

### 4.4 WebSocket Live Wire (AI Engineer + Backend Architect)
**Agent**: AI Engineer  
**Skill**: `/.ai/engineering/ai-engineer.md` + `/.ai/engineering/backend-architect.md`  
**Guard**: All RiskBus checks pass before order, no mock data in production paths

- [ ] Connect `TradingWebSocket` to running OMS API at `ws://localhost:8080/ws`
- [ ] Replace mock price data in `Watchlist` with live `market-data-store` updates
- [ ] Replace mock signals in `SignalFeed` with live `signal-store` updates
- [ ] Wire chart candle updates from `market-data-store.candles`
- [ ] Wire order submit in `OrderEntry` to `POST /api/orders` via `api.ts`
- [ ] Wire position updates from `usePositionStore`

**Backend Architect Handoff**:
```yaml
required_endpoints:
  - POST /api/auth/login
  - GET  /api/orders
  - POST /api/orders
  - DELETE /api/orders/:id
  - GET  /api/positions
  - GET  /api/market-data/:symbol
  - WS   /ws  (market.price, order.update, position.update, signal.new)
```

---

### 4.5 Authentication Flow (AI Engineer)
**Agent**: AI Engineer  
**Guard**: JWT stored in localStorage via auth-store persist, no tokens in URL

- [ ] `src/app/login/page.tsx` — login form
- [ ] `src/middleware.ts` — Next.js route protection (redirect unauthenticated)
- [ ] `src/app/api/auth/[...nextauth]/route.ts` OR custom JWT flow via `api.ts`
- [ ] Logout button in Header wired to `auth-store.logout()`

---

### 4.6 Error Boundaries & Loading States (Frontend Developer)
**Agent**: Frontend Developer  
**Guard**: No unhandled errors reach user; loading skeletons for all data panels

- [ ] `src/components/ui/skeleton.tsx` — loading skeleton primitive
- [ ] `src/components/error-boundary.tsx` — React error boundary
- [ ] Add `<Suspense>` + skeleton to: Chart, Positions, Orders
- [ ] Add `isLoading` state display to OrderEntry submit button
- [ ] Toast notifications for: order submitted, order filled, order rejected

---

## PHASE 5: MOBILE & DEPLOYMENT

### 5.1 Mobile App (Mobile App Builder)
**Agent**: Mobile App Builder  
**Skill**: `/.ai/engineering/mobile-app-builder.md`  
**Guard**: Same store layer, no duplication of business logic

- [ ] Evaluate React Native + Expo vs PWA approach
- [ ] PWA manifest + service worker for offline
- [ ] Touch-optimized order entry (large buttons, swipe to cancel)
- [ ] Push notifications for fills and signals

### 5.2 Production Deployment (DevOps Automator)
**Agent**: DevOps Automator  
**Skill**: `/.ai/engineering/devops-automator.md`  
**Guard**: Secrets in env vars only, never committed

- [ ] `packages/dashboard/.env.production` template (no real values)
- [ ] Docker: `packages/dashboard/Dockerfile` for Next.js
- [ ] `docker-compose.yml` at root: next + oms-engine + redis
- [ ] Vercel/Netlify deployment config OR k8s manifest
- [ ] Environment secrets: `NEXT_PUBLIC_API_URL`, `NEXT_PUBLIC_WS_URL`, `JWT_SECRET`
- [ ] Health check endpoint `/api/health`

### 5.3 Performance Validation (Performance Benchmarker)
**Agent**: Performance Benchmarker  
**Skill**: `/.ai/testing/performance-benchmarker.md`

- [ ] Chart render: <100ms on price update
- [ ] Order submit: <1.5s end-to-end
- [ ] WebSocket reconnect: <3s
- [ ] Dashboard LCP: <2.5s on 3G
- [ ] Lighthouse score: >85 on mobile

---

## PHASE 6: PRODUCTION HARDENING

### 6.1 E2E Tests (Test Writer Fixer)
**Agent**: Test Writer Fixer  
**Skill**: `/.ai/engineering/test-writer-fixer.md`  
**Guard**: Tests use real WebSocket messages, not mocks

- [ ] Playwright E2E: login → place order → verify position
- [ ] E2E: signal appears → click Trade Signal → order submitted
- [ ] E2E: price flash on market data update
- [ ] CI integration: E2E in separate `e2e` job after `build`

### 6.2 Monitoring & Observability (DevOps Automator)
- [ ] Sentry error tracking in Next.js
- [ ] WebSocket message latency metrics
- [ ] Order submit success rate dashboard
- [ ] Alert on: WS disconnect >10s, order failure rate >5%

---

## EXECUTION ORDER FOR PHASE 4

```
Week 1
├── 4.0 Pre-flight validation (DevOps) — Day 1
├── 4.1 Fix Zustand type errors (AI Engineer) — Day 1-2
├── 4.2 Position Table (Frontend Dev) — Day 2-3
└── 4.3 Order History Table (Frontend Dev) — Day 3

Week 2
├── 4.4 WebSocket Live Wire (AI Eng + Backend Arch) — Day 4-6
├── 4.5 Auth Flow (AI Engineer) — Day 6-7
└── 4.6 Error Boundaries (Frontend Dev) — Day 7

Week 3
├── Phase 5.1 PWA + Mobile (Mobile App Builder)
├── Phase 5.2 Docker + Deploy (DevOps Automator)
└── Phase 5.3 Performance validation (Perf Benchmarker)

Week 4
├── Phase 6.1 E2E Tests (Test Writer Fixer)
└── Phase 6.2 Monitoring (DevOps Automator)
```

---

## CURRENT BRANCH STATE
```
branch:  feature/github-mcp-setup
commit:  1685aed
files:   packages/dashboard/ — 40+ files
tests:   23 unit tests written
ci/cd:   .github/workflows/dashboard.yml active
```

## FIRST ACTION NEXT SESSION
```bash
cd packages/dashboard
npm ci
npm test
npm run build
```
All three must pass before any Phase 4 coding begins.
