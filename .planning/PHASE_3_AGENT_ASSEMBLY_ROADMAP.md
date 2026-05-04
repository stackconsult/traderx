# Phase 3: Agent Assembly & Build Roadmap
**Status**: Ready for Review | **Do Not Execute Until Confirmed**

---

## 1. AGENT SKILL MAPPING

### Core Agents Assigned

| Agent | Skill File | Role | Chunks |
|-------|-----------|------|--------|
| **Rapid Prototyper** | `/.ai/engineering/rapid-prototyper.md` | Scaffold, mock data | 1.1, 1.2 |
| **Frontend Developer** | `/.ai/engineering/frontend-developer.md` | UI/UX, components | 3.1-3.3, 6.1, 7.1-7.2 |
| **AI Engineer** | `/.ai/engineering/ai-engineer.md` | API, WebSocket, state | 1.3, 2.1-2.2, 4.1-4.2, 5.1-5.2 |
| **Test Writer Fixer** | `/.ai/engineering/test-writer-fixer.md` | Tests, validation | 7.3, all validation gates |
| **Backend Architect** | `/.ai/engineering/backend-architect.md` | API calibration, deploy | 2.2 handoff, Phase 4-5 |
| **UI Designer** | `/.ai/design/ui-designer.md` | Design system, polish | 1.2, 3.2, 7.1 |
| **UX Researcher** | `/.ai/design/ux-researcher.md` | HFT benchmarks, mobile | 3.2, 7.2 |
| **Performance Benchmarker** | `/.ai/testing/performance-benchmarker.md` | Latency, fps testing | 7.3 |

---

## 2. PHASE 3 BUILD CHUNKS (Agent-Assigned)

### Phase 3.0: Infrastructure (Day 1)

**Chunk 1.1: Project Scaffolding**
- **Agent**: Rapid Prototyper
- **Skills**: `project-scaffolding`, `boilerplate-generation`
- **Inputs**: `/.ai/engineering/rapid-prototyper.md` lines 45-78
- **Outputs**:
  - `packages/dashboard/package.json` - dependencies
  - `packages/dashboard/tsconfig.json` - strict TypeScript
  - `packages/dashboard/tailwind.config.ts` - trading theme
  - `packages/dashboard/next.config.js` - API rewrites
  - `packages/dashboard/postcss.config.js` - tailwind
- **Guard-Guidelines**:
  - Use Next.js 14.2.0 exactly
  - Dependencies: zustand, axios, lightweight-charts, next-themes, @radix-ui/*
  - Tailwind: extend colors (up: #00C805, down: #FF5000)
- **Validation Gate**: `npm install` passes, `npm run type-check` 0 errors

**Chunk 1.2: Design System & Theme**
- **Agent**: UI Designer (primary), Frontend Developer (support)
- **Skills**: `design-system-creation`, `color-theory-application`, `dark-mode-implementation`
- **Inputs**: `/.ai/design/ui-designer.md` lines 32-89
- **Outputs**:
  - `packages/dashboard/src/app/globals.css` - CSS variables, flash animations
  - `packages/dashboard/src/components/theme-provider.tsx` - next-themes wrapper
  - `packages/dashboard/src/components/theme-toggle.tsx` - toggle button
  - `packages/dashboard/src/components/ui/button.tsx` - shadcn button
  - `packages/dashboard/src/components/ui/card.tsx` - shadcn card
- **Guard-Guidelines**:
  - Dark mode default, system preference detection
  - No !important in CSS
  - CSS variables for all colors
- **Validation Gate**: Toggle works, colors match Robin Hood spec

**Chunk 1.3: State Management Architecture**
- **Agent**: AI Engineer
- **Skills**: `state-management-pattern`, `zustand-slice-pattern`
- **Inputs**: `/.ai/engineering/ai-engineer.md` lines 67-134
- **Outputs**:
  - `packages/dashboard/src/types/store.ts` - shared interfaces
  - `packages/dashboard/src/store/auth-store.ts` - JWT, persist
  - `packages/dashboard/src/store/order-store.ts` - CRUD, optimistic
  - `packages/dashboard/src/store/position-store.ts` - P&L calc
  - `packages/dashboard/src/store/market-data-store.ts` - price cache
  - `packages/dashboard/src/store/signal-store.ts` - confidence filter
  - `packages/dashboard/src/store/index.ts` - exports
- **Guard-Guidelines**:
  - Redux DevTools enabled
  - localStorage persist for auth only
  - No circular dependencies
- **Validation Gate**: Stores import without errors, devtools connected

---

### Phase 3.1: API & WebSocket Layer (Days 2-3)

**Chunk 2.1: HTTP API Client**
- **Agent**: AI Engineer
- **Skills**: `http-client-design`, `error-handling-patterns`, `retry-logic`
- **Inputs**: `/.ai/engineering/ai-engineer.md` lines 135-198
- **Outputs**:
  - `packages/dashboard/src/lib/api.ts` - axios with interceptors
  - `packages/dashboard/src/lib/api-types.ts` - request/response types
- **Guard-Guidelines**:
  - 3 retry attempts with exponential backoff
  - 401 redirects to /login
  - 429 rate limit warnings
  - 10s timeout
- **Validation Gate**: Mock API calls work, errors handled

**Chunk 2.2: WebSocket Client**
- **Agent**: AI Engineer (primary), Backend Architect (calibration)
- **Skills**: `websocket-client-design`, `reconnection-logic`, `api-calibration`
- **Inputs**: `/.ai/engineering/ai-engineer.md` lines 199-245, `/.ai/engineering/backend-architect.md` lines 78-112
- **Outputs**:
  - `packages/dashboard/src/lib/websocket.ts` - TradingWebSocket class
  - `packages/dashboard/src/hooks/use-websocket.ts` - React hook
- **Guard-Guidelines**:
  - 5 max reconnect attempts
  - 30s heartbeat
  - Message queue when disconnected
  - 1000 msg/sec rate limit
- **Validation Gate**: Connects to ws://localhost:8080/ws, reconnects on disconnect
- **Handoff to Backend Architect**: WebSocket performance specs for calibration

---

### Phase 3.2: Core Trading Components (Days 4-5)

**Chunk 3.1: Layout & Navigation**
- **Agent**: Frontend Developer (primary), Rapid Prototyper (routes)
- **Skills**: `responsive-layout`, `navigation-patterns`, `route-setup`
- **Inputs**: `/.ai/engineering/frontend-developer.md` lines 45-89
- **Outputs**:
  - `packages/dashboard/src/components/layout/dashboard-layout.tsx` - shell
  - `packages/dashboard/src/components/layout/sidebar.tsx` - desktop nav
  - `packages/dashboard/src/components/layout/header.tsx` - top bar
  - `packages/dashboard/src/components/layout/mobile-nav.tsx` - bottom nav
  - `packages/dashboard/src/app/dashboard/page.tsx` - main view
  - `packages/dashboard/src/app/trading/page.tsx` - trading view
  - `packages/dashboard/src/app/positions/page.tsx` - positions
  - `packages/dashboard/src/app/orders/page.tsx` - orders
  - `packages/dashboard/src/app/settings/page.tsx` - settings
- **Guard-Guidelines**:
  - Mobile-first, 44px tap targets
  - Active state highlighting
  - Sheet component for mobile menu
- **Validation Gate**: All routes render, responsive at 320px, 768px, 1024px+

**Chunk 3.2: Order Entry Form**
- **Agent**: Frontend Developer (primary), UX Researcher (safety patterns)
- **Skills**: `form-validation`, `react-hook-form`, `error-prevention`
- **Inputs**: `/.ai/engineering/frontend-developer.md` lines 90-156, `/.ai/design/ux-researcher.md` lines 67-89
- **Outputs**:
  - `packages/dashboard/src/components/trading/order-entry.tsx` - main form
  - `packages/dashboard/src/lib/validation/order-schema.ts` - zod schema
  - `packages/dashboard/src/components/ui/label.tsx` - shadcn label
  - `packages/dashboard/src/components/ui/select.tsx` - shadcn select
  - `packages/dashboard/src/components/ui/toggle.tsx` - shadcn toggle
  - `packages/dashboard/src/components/ui/input.tsx` - shadcn input
  - `packages/dashboard/src/components/ui/separator.tsx` - shadcn separator
  - `packages/dashboard/src/components/ui/sheet.tsx` - shadcn sheet
- **Guard-Guidelines**:
  - Buy=green (#00C805), Sell=red (#FF5000)
  - Order types: market, limit, stop, stop_limit, iceberg
  - Time in Force: day, gtc, ioc, fok
  - Warning if >10% buying power
  - Inline validation, no alerts
- **Validation Gate**: Form submits, validation works, cost calculates correctly

**Chunk 3.3: Position Table (Stub)**
- **Agent**: Frontend Developer
- **Skills**: `data-table-implementation`
- **Outputs**:
  - `packages/dashboard/src/components/dashboard/portfolio-summary.tsx` - cards
  - `packages/dashboard/src/components/dashboard/key-metrics.tsx` - metrics
- **Guard-Guidelines**: Mock data only, ready for real data in Phase 3.4
- **Validation Gate**: Cards render with mock P&L

---

### Phase 3.3: Real-Time Layer (Days 6-7)

**Chunk 4.1: Price Ticker Component**
- **Agent**: Frontend Developer (primary), AI Engineer (data flow)
- **Skills**: `real-time-ui-updates`, `animation-optimization`
- **Outputs**:
  - `packages/dashboard/src/components/market/price-ticker.tsx`
  - `packages/dashboard/src/hooks/use-price-flash.ts`
- **Guard-Guidelines**:
  - Flash animation on price change (flash-up/down CSS)
  - 100ms batch updates
  - Color coding: green up, red down
- **Validation Gate**: Price updates flash, no jitter

**Chunk 4.2: Market Data Panel**
- **Agent**: Frontend Developer, AI Engineer (WebSocket wiring)
- **Skills**: `watchlist-implementation`, `websocket-integration`
- **Outputs**:
  - `packages/dashboard/src/components/market/watchlist.tsx`
  - `packages/dashboard/src/components/market/symbol-search.tsx`
- **Guard-Guidelines**:
  - Add/remove symbols
  - Real-time price updates
  - Search autocomplete
- **Validation Gate**: Watchlist updates from WebSocket

---

### Phase 3.4: Chart Integration (Days 8-9)

**Chunk 5.1: TradingView Chart Component**
- **Agent**: AI Engineer (primary), Frontend Developer (controls)
- **Skills**: `charting-library-integration`, `real-time-data-feed`
- **Outputs**:
  - `packages/dashboard/src/components/charts/trading-view-chart.tsx`
  - `packages/dashboard/src/components/charts/timeframe-selector.tsx`
  - `packages/dashboard/src/hooks/use-chart-data.ts`
- **Guard-Guidelines**:
  - Lightweight Charts library
  - Timeframes: 1m, 5m, 15m, 1h, 4h, 1d
  - Candlestick + volume
  - Real-time candle updates
- **Validation Gate**: Chart renders, data flows from WebSocket

**Chunk 5.2: Signal Feed Integration**
- **Agent**: AI Engineer
- **Skills**: `signal-processing`, `alert-system`
- **Outputs**:
  - `packages/dashboard/src/components/signals/signal-feed.tsx`
  - `packages/dashboard/src/components/signals/signal-card.tsx`
- **Guard-Guidelines**:
  - Confidence filter (default 0.7)
  - One-click order from signal
  - Signal metadata display
- **Validation Gate**: Signals appear, filter works

---

### Phase 3.5: Dashboard Assembly (Day 9)

**Chunk 6.1: Main Dashboard Page**
- **Agent**: Frontend Developer
- **Skills**: `page-composition`, `responsive-grid`
- **Outputs**:
  - Final `packages/dashboard/src/app/dashboard/page.tsx`
  - `packages/dashboard/src/components/dashboard/dashboard-grid.tsx`
- **Guard-Guidelines**:
  - Desktop: 3-column grid
  - Tablet: 2-column
  - Mobile: 1-column stack
  - 16px gaps, 24px padding
- **Validation Gate**: All breakpoints render correctly

---

### Phase 3.6: Polish & Testing (Day 10)

**Chunk 7.1: Dark Mode Polish**
- **Agent**: Frontend Developer, UI Designer
- **Skills**: `dark-mode-implementation`, `visual-hierarchy`
- **Outputs**: Dark mode refinements across all components
- **Guard-Guidelines**:
  - All components have dark: variants
  - Charts use dark theme
  - No hardcoded colors

**Chunk 7.2: Mobile Responsive**
- **Agent**: Frontend Developer, UX Researcher
- **Skills**: `mobile-first-responsive`, `touch-optimization`
- **Outputs**: Mobile-optimized trading form, tables
- **Guard-Guidelines**:
  - Min 44px tap targets
  - 16px font minimum
  - Swipe gestures where appropriate

**Chunk 7.3: Test Suite**
- **Agent**: Test Writer Fixer, Performance Benchmarker
- **Skills**: `unit-test-creation`, `integration-test-setup`, `performance-testing`
- **Inputs**: `/.ai/engineering/test-writer-fixer.md` lines 45-89, `/.ai/testing/performance-benchmarker.md` lines 34-67
- **Outputs**:
  - `packages/dashboard/src/__tests__/api-client.test.ts`
  - `packages/dashboard/src/__tests__/order-entry.test.tsx`
  - `packages/dashboard/src/__tests__/performance/dashboard.perf.test.ts`
- **Guard-Guidelines**:
  - Unit tests: stores, utils
  - Integration: order flow
  - Performance: <100ms chart update, <1.5s order submit
- **Validation Gate**: All tests pass

---

## 3. SENTRUX RULES ENFORCEMENT

From `/.sentrux/rules.toml`:

```toml
[[rules]]
id = "dashboard-no-todo"
pattern = "TODO|FIXME|XXX"
severity = "error"
categories = ["code-quality"]

[[rules]]
id = "dashboard-type-strict"
pattern = "any"
severity = "error"
files = ["*.ts", "*.tsx"]

[[rules]]
id = "dashboard-api-timeout"
pattern = "timeout.*[0-9]{5,}"
message = "API timeout must be <10s"
severity = "warning"

[[rules]]
id = "dashboard-memory-cleanup"
pattern = "setInterval|setTimeout"
message = "Must have cleanup in useEffect return"
severity = "error"
```

**Pre-Commit Check**:
```bash
cd packages/dashboard
npm run type-check  # 0 errors
npm run lint        # 0 errors
npm run test:unit   # all pass
```

---

## 4. HANDOFF SPECIFICATIONS

### To Backend Architect (from AI Engineer)
```yaml
webSocketSpecs:
  connectionLimit: 100
  messageRate: 1000  # msg/sec per client
  batchUpdates: 100  # ms
  jwtExpiry: 24h
  rateLimit: 100  # req/min
  corsOrigins: ["http://localhost:3000"]
```

### To Test Writer Fixer (from Frontend Developer)
```yaml
testRequirements:
  - order entry form validation
  - position table sorting/filtering
  - price ticker real-time updates
  - chart data synchronization
  - webSocket reconnection
  - performance: <100ms chart update
```

---

## 5. EXECUTION SEQUENCE

```
Phase 3.0 (Day 1)
├── Chunk 1.1 → Rapid Prototyper
├── Chunk 1.2 → UI Designer + Frontend Developer
└── Chunk 1.3 → AI Engineer

Phase 3.1 (Days 2-3)
├── Chunk 2.1 → AI Engineer
└── Chunk 2.2 → AI Engineer → Backend Architect (handoff)

Phase 3.2 (Days 4-5)
├── Chunk 3.1 → Frontend Developer
├── Chunk 3.2 → Frontend Developer + UX Researcher
└── Chunk 3.3 → Frontend Developer

Phase 3.3 (Days 6-7)
├── Chunk 4.1 → Frontend Developer + AI Engineer
└── Chunk 4.2 → Frontend Developer + AI Engineer

Phase 3.4 (Days 8-9)
├── Chunk 5.1 → AI Engineer + Frontend Developer
└── Chunk 5.2 → AI Engineer

Phase 3.5 (Day 9)
└── Chunk 6.1 → Frontend Developer

Phase 3.6 (Day 10)
├── Chunk 7.1 → Frontend Developer + UI Designer
├── Chunk 7.2 → Frontend Developer + UX Researcher
└── Chunk 7.3 → Test Writer Fixer + Performance Benchmarker
```

---

## 6. REVIEW CHECKLIST

Before execution, confirm:
- [ ] Agent assignments clear for each chunk
- [ ] Guard-guidelines defined for each chunk
- [ ] Validation gates specified
- [ ] Handoff specs included
- [ ] Sentrux rules referenced
- [ ] Execution sequence logical
- [ ] No TODO/FIXME allowed in committed code
- [ ] Max 3 files per commit rule acknowledged

**Status**: ⏸️ PAUSED FOR REVIEW

**Next Action**: User reviews this roadmap, confirms or requests changes, then says "execute" to begin Phase 3.0 Chunk 1.1.
