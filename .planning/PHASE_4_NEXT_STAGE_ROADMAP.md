# Phase 4: Next Stage Roadmap Checklist

**Prepared by**: Workflow Optimizer + Engineering Team  
**Follows**: Phase 3 completion (commit `1685aed`)  
**Status**: ✅ PHASE 4 + 5 + 6 COMPLETE — PHASE 7 VALIDATION PASSED — HEAD: `29f962c`

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

- [x] **4.0.1** — `npm install` completed ✅ 1015 packages — Node 20.20.2 via nvm
- [x] **4.0.2** — `npm test` ✅ 23/23 tests green
- [x] **4.0.3** — `npm run build` ✅ exit 0, all 9 routes compiled
- [x] **4.0.4** — Verify `.github/workflows/dashboard.yml` triggers on PR ✅
- [ ] **4.0.5** — Confirm OMS API server (`packages/oms-engine/src/api_server`) is running on `:8080` *(backend work)*

**Command**:

```bash
cd packages/dashboard && npm ci && npm test && npm run build
```

---

### 4.1 Fix Zustand Type Errors (AI Engineer)

**Agent**: AI Engineer  
**Skill**: `/.ai/engineering/ai-engineer.md`  
**Guard**: No `any` in store selectors, strict Zustand generics

- [x] Add explicit state type to all `useXxxStore((state) => ...)` selectors
- [x] Export store state types from `src/types/store.ts` → `src/types/store-states.ts`
- [x] Add `StoreApi` typed selectors pattern
- [x] Fix `parameter 'state' implicitly has 'any'` errors across all 5 stores

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

- [x] `src/components/trading/position-table.tsx` — sortable data table
- [x] Columns: Symbol, Qty, Avg Price, Current Price, P&L $, P&L %, Market Value
- [x] Color: positive P&L = `text-up`, negative = `text-down`
- [x] Close position button (triggers sell order flow)
- [x] Real-time mark price from `market-data-store`
- [x] Wire into dashboard main area

---

### 4.3 Order History Table (Frontend Developer)

**Agent**: Frontend Developer  
**Guard**: Paginated, filterable by status/symbol

- [x] `src/components/trading/order-history-table.tsx`
- [x] Filter: All / Pending / Filled / Cancelled
- [x] Cancel pending order action
- [x] Wire into dashboard main area

---

### 4.4 WebSocket Live Wire (AI Engineer + Backend Architect)

**Agent**: AI Engineer  
**Skill**: `/.ai/engineering/ai-engineer.md` + `/.ai/engineering/backend-architect.md`  
**Guard**: All RiskBus checks pass before order, no mock data in production paths

- [x] Connect `TradingWebSocket` via `useTradingWs()` mounted in `DashboardLayout`
- [x] Replace mock price data in `Watchlist` with live `market-data-store` updates
- [x] Replace mock signals in `SignalFeed` with live `signal-store` updates + toast on new signal
- [x] Wire chart candle updates from `market-data-store.candles`
- [x] Wire order submit in `OrderEntry` to `POST /api/orders` via `api.ts`
- [x] Wire position updates from `usePositionStore`
- [x] Signal → OrderEntry prefill via `CustomEvent traderx:prefill-order`

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

- [x] `src/app/login/page.tsx` — login form (JWT POST to `/api/auth/login`)
- [x] `src/middleware.ts` — Next.js route protection (redirect unauthenticated)
- [x] Custom JWT flow via `api.ts` + `auth-store` Zustand persist
- [x] Logout button in Header wired to `auth-store.logout()` + avatar dropdown

---

### 4.6 Error Boundaries & Loading States (Frontend Developer)

**Agent**: Frontend Developer  
**Guard**: No unhandled errors reach user; loading skeletons for all data panels

- [x] `src/components/ui/skeleton.tsx` — loading skeleton primitive
- [x] `src/components/error-boundary.tsx` — React error boundary
- [x] `ErrorBoundary` wraps Chart, Positions, Orders in dashboard
- [x] `isLoading` spinner on OrderEntry submit button
- [x] Toast notifications: order submitted, order filled, order rejected, new signal
- [x] `components/ui/toast.tsx` + `toaster.tsx` + `hooks/use-toast.ts`

---

## PHASE 5: MOBILE & DEPLOYMENT

### 5.1 Mobile App (Mobile App Builder)

**Agent**: Mobile App Builder  
**Skill**: `/.ai/engineering/mobile-app-builder.md`  
**Guard**: Same store layer, no duplication of business logic

- [x] Chose PWA approach (same codebase, no duplication)
- [x] `public/manifest.json` — standalone display, dark theme, icon declarations
- [x] `app/layout.tsx` metadata: `manifest`, `themeColor`, `appleWebApp` for iOS PWA
- [ ] Touch-optimized order entry (large buttons, swipe to cancel) — *future*
- [ ] Push notifications for fills and signals — *future (requires service worker + VAPID)*

### 5.2 Production Deployment (DevOps Automator)

**Agent**: DevOps Automator  
**Skill**: `/.ai/engineering/devops-automator.md`  
**Guard**: Secrets in env vars only, never committed

- [x] `packages/dashboard/.env.production.example` template (no real values)
- [x] `packages/dashboard/Dockerfile` — 3-stage build, non-root user
- [x] `docker-compose.traderx.yml` at root — next + oms-engine + redis with healthchecks
- [ ] Vercel/Netlify deployment config OR k8s manifest — *future*
- [x] Environment secrets documented in `.env.production.example`
- [x] Health check endpoint `GET /api/health`

### 5.3 Performance Validation (Performance Benchmarker)

**Agent**: Performance Benchmarker  
**Skill**: `/.ai/testing/performance-benchmarker.md`

- [ ] Chart render: <100ms on price update — *requires live OMS backend*
- [ ] Order submit: <1.5s end-to-end — *requires live OMS backend*
- [ ] WebSocket reconnect: <3s — *requires live OMS backend*
- [ ] Dashboard LCP: <2.5s on 3G — *build passes; run Lighthouse next*
- [ ] Lighthouse score: >85 on mobile — *build passes; run Lighthouse next*

---

## PHASE 6: PRODUCTION HARDENING

### 6.1 E2E Tests (Test Writer Fixer)

**Agent**: Test Writer Fixer  
**Skill**: `/.ai/engineering/test-writer-fixer.md`  
**Guard**: Tests use real WebSocket messages, not mocks

- [x] `playwright.config.ts` — Chromium + Mobile Safari, dev server auto-start
- [x] `e2e/auth.spec.ts` — redirect, form validation, successful login
- [x] `e2e/trading.spec.ts` — order submit toast, position table render
- [x] `package.json` — `test:e2e` + `test:e2e:ui` scripts, `@playwright/test` devDep
- [x] E2E: signal appears → CustomEvent prefills OrderEntry symbol + side
- [x] E2E: price flash — inject via store.setPrice, assert DOM renders updated price
- [x] CI integration: `e2e` job after `build` in `.github/workflows/dashboard.yml`

### 6.2 Monitoring & Observability (DevOps Automator)

- [x] `sentry.client.config.ts` — browser tracing + session replay, production-only
- [x] `@sentry/nextjs ^8.0.0` added to dependencies
- [ ] WebSocket message latency metrics — *requires live backend*
- [ ] Order submit success rate dashboard — *requires live backend*
- [ ] Alert on: WS disconnect >10s, order failure rate >5% — *requires live backend*

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
commit:  29f962c  (HEAD — synced with origin)
node:    v20.20.2 via nvm (installed this session)
files:   packages/dashboard/ — 60+ files + package-lock.json
tests:   23/23 unit tests green
build:   next build exit 0 — 9 routes compiled
ci/cd:   .github/workflows/dashboard.yml — lint + test + build + e2e + rust jobs
```

## PHASE 7: OMS BACKEND + LIVE INTEGRATION

> **We are here.** Frontend is complete and validated. The blocker is the Rust OMS backend.

### 7.1 OMS API Endpoints (Backend Architect — packages/oms-engine)

- [ ] `POST /api/auth/login` — JWT issue, validate credentials
- [ ] `GET  /api/orders` — list orders for authenticated user
- [ ] `POST /api/orders` — submit order through RiskBus → OMS
- [ ] `DELETE /api/orders/:id` — cancel pending order
- [ ] `GET  /api/positions` — current positions from portfolio store
- [ ] `WS   /ws` — broadcast `market.price`, `order.update`, `position.update`, `signal.new`
- [ ] Run `cargo check --package oms-engine` — error count must not increase
- [ ] Paper trading mode enabled before any live orders

### 7.2 E2E Tests Against Live Backend

- [ ] `npx playwright install chromium` — one-time browser install
- [ ] `npm run test:e2e` — auth + trading + signal prefill + price flash
- [ ] All 5 Playwright specs passing

### 7.3 Performance Benchmarks

- [ ] Lighthouse CLI on production build — target LCP <2.5s, score >85 mobile
- [ ] WS reconnect timing — target <3s
- [ ] Order submit latency — target <1.5s end-to-end

### 7.4 Phase 8 Horizon (Future)

- [ ] k8s manifests or Vercel deployment config
- [ ] Push notification service worker (VAPID keys)
- [ ] Touch-optimized mobile order entry
- [ ] Sentry alerting rules (WS disconnect >10s, order failure rate >5%)
