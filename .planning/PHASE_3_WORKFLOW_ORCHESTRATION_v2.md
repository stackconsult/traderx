# Phase 3: Frontend Trading Dashboard - Agent Skill Mapped Workflow

## Executive Summary

**Mission**: Build React dashboard meeting HFT platform benchmarks (Robin Hood, IBKR, TradingView) with systematic agent-orchestrated execution.

**Agent Team with Skill Mappings**:
- **Workflow Optimizer** (`/.ai/testing/workflow-optimizer.md`) → Chunk planning, validation gates
- **AI Engineer** (`/.ai/engineering/ai-engineer.md`) → API integration, WebSocket, state management
- **Frontend Developer** (`/.ai/engineering/frontend-developer.md`) → UI/UX, components, styling
- **Test Writer Fixer** (`/.ai/engineering/test-writer-fixer.md`) → Test specs, validation, deployment handoff
- **Backend Architect** (`/.ai/engineering/backend-architect.md`) → API calibration, WebSocket perf, deployment
- **Rapid Prototyper** (`/.ai/engineering/rapid-prototyper.md`) → UI shell, scaffolding, mock data
- **UI Designer** (`/.ai/design/ui-designer.md`) → Design system, component patterns
- **UX Researcher** (`/.ai/design/ux-researcher.md`) → HFT platform UX analysis
- **Performance Benchmarker** (`/.ai/testing/performance-benchmarker.md`) → Latency, fps, load testing

---

## Part 1: Agent Skill Matrix by Build Chunk

### Phase 3.0: Infrastructure Setup (Day 1)

#### Chunk 1.1: Project Scaffolding
**Primary Agent**: Rapid Prototyper (`/.ai/engineering/rapid-prototyper.md`)
**Supporting**: Frontend Developer (`/.ai/engineering/frontend-developer.md`)

**Skill Workflows**:
```yaml
rapid-prototyper:
  skills:
    - project-scaffolding
    - boilerplate-generation
    - quick-mock-creation
  workflow:
    - initialize: "npx shadcn-ui@latest init"
    - template: "next-app-template"
    - dependencies: ["zustand", "axios", "lightweight-charts"]
  outputs:
    - packages/dashboard/package.json
    - packages/dashboard/tsconfig.json
    - packages/dashboard/tailwind.config.ts
    - packages/dashboard/src/app/layout.tsx (shell)
    - packages/dashboard/src/app/page.tsx (placeholder)

frontend-developer:
  skills:
    - component-architecture
    - tailwind-configuration
    - responsive-design
  workflow:
    - configure: "tailwind custom colors"
    - extend: "theme tokens for trading UI"
  outputs:
    - tailwind.config.ts (extended)
    - src/lib/utils.ts (cn helper)
```

**Commit**: `feat(dashboard): initialize Next.js with shadcn/ui`

---

#### Chunk 1.2: Design System & Theme
**Primary Agent**: UI Designer (`/.ai/design/ui-designer.md`)
**Supporting**: Frontend Developer (`/.ai/engineering/frontend-developer.md`)

**Skill Workflows**:
```yaml
ui-designer:
  skills:
    - design-system-creation
    - component-pattern-library
    - color-theory-application
  workflow:
    - palette: "Robin Hood + IBKR hybrid"
    - tokens:
        colors:
          up: "#00C805"        # Robin Hood green
          down: "#FF5000"      # Robin Hood red
          primary: "#000000"   # Black text
          background: "#FFFFFF"
          card: "#F5F5F5"
          dark-bg: "#0a0a0a"
          dark-card: "#1a1a1a"
    - components: ["button", "input", "select", "card", "table", "tabs"]
  outputs:
    - src/styles/tokens.ts
    - src/components/ui/button.tsx
    - src/components/ui/input.tsx
    - src/components/ui/select.tsx
    - src/components/ui/card.tsx
    - src/components/ui/table.tsx
    - src/components/ui/tabs.tsx

frontend-developer:
  skills:
    - dark-mode-implementation
    - theme-provider-setup
  workflow:
    - implement: "next-themes integration"
    - persist: "theme preference in localStorage"
  outputs:
    - src/components/theme-provider.tsx
    - src/hooks/use-theme.ts
```

**Commit**: `feat(ui): design system with shadcn components + dark mode`

---

#### Chunk 1.3: State Management Architecture
**Primary Agent**: AI Engineer (`/.ai/engineering/ai-engineer.md`)
**Supporting**: Frontend Developer (`/.ai/engineering/frontend-developer.md`)

**Skill Workflows**:
```yaml
ai-engineer:
  skills:
    - state-management-pattern
    - api-client-design
    - websocket-integration
    - type-safe-architecture
  workflow:
    pattern: "Zustand with slice pattern"
    structure:
      - auth-store.ts: "JWT, user, login/logout"
      - order-store.ts: "CRUD, optimistic updates"
      - position-store.ts: "P&L calculations, exposure"
      - market-data-store.ts: "price cache, tick history"
      - signal-store.ts: "neural signals, filtering"
    middleware:
      - devtools: "Redux DevTools integration"
      - persist: "localStorage for auth"
  outputs:
    - src/store/index.ts
    - src/store/auth-store.ts
    - src/store/order-store.ts
    - src/store/position-store.ts
    - src/store/market-data-store.ts
    - src/store/signal-store.ts
    - src/types/store.ts (shared types)
```

**Commit**: `feat(store): zustand stores with slice pattern`

---

### Phase 3.1: API & WebSocket Layer (Days 2-3)

#### Chunk 2.1: HTTP API Client
**Primary Agent**: AI Engineer (`/.ai/engineering/ai-engineer.md`)
**Supporting**: Test Writer Fixer (`/.ai/engineering/test-writer-fixer.md`)

**Skill Workflows**:
```yaml
ai-engineer:
  skills:
    - http-client-design
    - error-handling-patterns
    - retry-logic
    - interceptor-pattern
  workflow:
    client: "axios with custom config"
    interceptors:
      request:
        - attach-jwt: "from auth-store"
        - add-timestamp: "for debugging"
      response:
        - handle-401: "redirect to login"
        - handle-429: "rate limit backoff"
        - exponential-retry: "3 attempts"
    error-types:
      - NetworkError
      - TimeoutError
      - AuthError
      - RateLimitError
  outputs:
    - src/lib/api.ts
    - src/lib/api-types.ts
    - src/lib/api-errors.ts
    - src/lib/api-interceptors.ts

test-writer-fixer:
  skills:
    - unit-test-creation
    - mock-server-setup
    - error-scenario-testing
  workflow:
    tests:
      - api-client.test.ts: "mock axios, test interceptors"
      - error-handling.test.ts: "test retry logic"
  outputs:
    - src/__tests__/api-client.test.ts
```

**Commit**: `feat(api): axios client with JWT interceptor + error handling`

---

#### Chunk 2.2: WebSocket Client
**Primary Agent**: AI Engineer (`/.ai/engineering/ai-engineer.md`)
**Supporting**: Backend Architect (`/.ai/engineering/backend-architect.md`)

**Skill Workflows**:
```yaml
ai-engineer:
  skills:
    - websocket-client-design
    - reconnection-logic
    - message-handler-architecture
    - heartbeat-pattern
  workflow:
    features:
      - auto-reconnect: "exponential backoff"
      - heartbeat: "ping every 30s"
      - message-queue: "buffer when disconnected"
    handlers:
      market_data: "update price store"
      order_update: "refresh order store"
      position_update: "recalculate P&L"
      signal: "add to signal feed"
  outputs:
    - src/lib/websocket.ts
    - src/lib/websocket-types.ts
    - src/hooks/use-websocket.ts
    - src/hooks/use-market-data.ts

backend-architect:
  skills:
    - api-calibration
    - performance-optimization
    - connection-management
  workflow:
    validate:
      - connection-limit: "max 100 concurrent"
      - message-rate: "1000 msg/sec per client"
      - batch-updates: "100ms batching"
  outputs:
    - src/lib/websocket-config.ts
```

**Commit**: `feat(websocket): auto-reconnecting WebSocket client + hooks`

---

### Phase 3.2: Core Trading Components (Days 4-5)

#### Chunk 3.1: Layout & Navigation
**Primary Agent**: Frontend Developer (`/.ai/engineering/frontend-developer.md`)
**Supporting**: Rapid Prototyper (`/.ai/engineering/rapid-prototyper.md`)

**Skill Workflows**:
```yaml
frontend-developer:
  skills:
    - responsive-layout
    - grid-system-implementation
    - navigation-patterns
    - accessibility-compliance
  workflow:
    layout:
      pattern: "sidebar + header + main"
      responsive: "mobile-first"
      grid: "12-column with breakpoints"
    components:
      - DashboardLayout: "shell with slots"
      - Header: "logo, nav, user, theme"
      - Sidebar: "route links, active state"
      - MainContent: "flexible content area"
  outputs:
    - src/components/layout/dashboard-layout.tsx
    - src/components/layout/header.tsx
    - src/components/layout/sidebar.tsx
    - src/components/layout/mobile-nav.tsx

rapid-prototyper:
  skills:
    - quick-component-scaffolding
    - placeholder-content
    - route-setup
  workflow:
    routes:
      - /dashboard: "main trading view"
      - /orders: "order history"
      - /positions: "portfolio view"
      - /settings: "user preferences"
  outputs:
    - src/app/dashboard/page.tsx
    - src/app/orders/page.tsx
    - src/app/positions/page.tsx
    - src/app/settings/page.tsx
```

**Commit**: `feat(layout): responsive dashboard shell with navigation`

---

#### Chunk 3.2: Order Entry Form
**Primary Agent**: Frontend Developer (`/.ai/engineering/frontend-developer.md`)
**Supporting**: UX Researcher (`/.ai/design/ux-researcher.md`)

**Skill Workflows**:
```yaml
frontend-developer:
  skills:
    - form-validation
    - react-hook-form
    - zod-schema-validation
    - optimistic-updates
  workflow:
    form:
      fields:
        - symbol: "search with autocomplete"
        - side: "buy/sell toggle"
        - quantity: "numeric with step"
        - orderType: "market/limit/stop"
        - price: "conditional on type"
        - timeInForce: "day/gtc/ioc"
      validation: "zod schema"
      submission: "optimistic update + toast"
    preview:
      - estimated-cost: "quantity * price"
      - buying-power-impact: "post-trade available"
      - risk-warning: "if >10% portfolio"
  outputs:
    - src/components/trading/order-entry.tsx
    - src/components/trading/order-form.tsx
    - src/components/trading/order-preview.tsx
    - src/lib/validation/order-schema.ts

ux-researcher:
  skills:
    - hft-platform-analysis
    - interaction-design
    - error-prevention
  workflow:
    patterns:
      - robin-hood: "simple, clean, mobile-first"
      - ibkr: "advanced order types accessible"
    safety:
      - confirm-dialog: "for large orders"
      - clear-errors: "inline validation"
      - undo: "cancel within 5s"
```

**Commit**: `feat(trading): order entry form with validation + preview`

---

#### Chunk 3.3: Position Table
**Primary Agent**: Frontend Developer (`/.ai/engineering/frontend-developer.md`)
**Supporting**: UI Designer (`/.ai/design/ui-designer.md`)

**Skill Workflows**:
```yaml
frontend-developer:
  skills:
    - data-table-implementation
    - tanstack-table
    - sorting-filtering
    - virtualization
  workflow:
    table:
      columns:
        - symbol: "with mini chart spark"
        - quantity: "with side indicator"
        - avg-entry: "entry price"
        - current-price: "real-time"
        - market-value: "qty * price"
        - unrealized-pnl: "with color"
        - realized-pnl: "lifetime"
        - actions: "close, add"
      features:
        - sort: "by any column"
        - filter: "symbol search"
        - pagination: "if >50 rows"
        - real-time: "price updates flash"
  outputs:
    - src/components/trading/position-table.tsx
    - src/components/trading/position-columns.tsx
    - src/components/trading/position-row.tsx
    - src/hooks/use-positions.ts

ui-designer:
  skills:
    - data-visualization
    - color-coding
    - micro-interactions
  workflow:
    design:
      pnl-positive: "green with up arrow"
      pnl-negative: "red with down arrow"
      price-change: "flash effect on update"
      hover: "row highlight + action buttons"
```

**Commit**: `feat(trading): sortable position table with real-time P&L`

---

### Phase 3.3: Real-Time Layer (Days 6-7)

#### Chunk 4.1: Price Ticker Component
**Primary Agent**: Frontend Developer (`/.ai/engineering/frontend-developer.md`)
**Supporting**: AI Engineer (`/.ai/engineering/ai-engineer.md`)

**Skill Workflows**:
```yaml
frontend-developer:
  skills:
    - real-time-ui-updates
    - animation-optimization
    - color-transitions
  workflow:
    component:
      display: "symbol + price + change"
      animation: "color flash on change"
      format: "2 decimal places"
    variants:
      - compact: "for watchlist"
      - detailed: "with volume, spread"
      - hero: "large for main display"
  outputs:
    - src/components/market/price-ticker.tsx
    - src/components/market/price-change.tsx
    - src/hooks/use-price-flash.ts

ai-engineer:
  skills:
    - websocket-data-flow
    - state-synchronization
    - performance-optimization
  workflow:
    optimization:
      - batch-updates: "100ms throttle"
      - selective-render: "only changed symbols"
      - memory-management: "limit tick history"
```

**Commit**: `feat(market): real-time price ticker with flash animation`

---

#### Chunk 4.2: Market Data Panel
**Primary Agent**: Frontend Developer (`/.ai/engineering/frontend-developer.md`)
**Supporting**: Rapid Prototyper (`/.ai/engineering/rapid-prototyper.md`)

**Skill Workflows**:
```yaml
frontend-developer:
  skills:
    - watchlist-implementation
    - search-autocomplete
    - websocket-integration
  workflow:
    watchlist:
      - add-symbol: "search + add"
      - remove: "swipe on mobile"
      - reorder: "drag on desktop"
      - columns: "customizable"
    order-book:
      - depth: "5-10 levels"
      - visualization: "depth chart"
      - spread: "highlight if wide"
  outputs:
    - src/components/market/watchlist.tsx
    - src/components/market/order-book.tsx
    - src/components/market/market-depth.tsx
    - src/components/market/symbol-search.tsx
```

**Commit**: `feat(market): watchlist + order book with depth visualization`

---

### Phase 3.4: Chart Integration (Days 8-9)

#### Chunk 5.1: TradingView Chart Component
**Primary Agent**: AI Engineer (`/.ai/engineering/ai-engineer.md`)
**Supporting**: Frontend Developer (`/.ai/engineering/frontend-developer.md`)

**Skill Workflows**:
```yaml
ai-engineer:
  skills:
    - charting-library-integration
    - real-time-data-feed
    - technical-indicators
  workflow:
    library: "TradingView Lightweight Charts"
    features:
      - candlestick: "1m, 5m, 15m, 1h, 1d"
      - volume: "histogram below"
      - studies: "SMA, EMA, RSI, MACD"
      - drawings: "trend lines, support/resistance"
    data:
      - historical: "REST API fetch"
      - realtime: "WebSocket updates"
      - merge: "update last candle"
  outputs:
    - src/components/charts/trading-view-chart.tsx
    - src/components/charts/chart-container.tsx
    - src/components/charts/study-panel.tsx
    - src/hooks/use-chart-data.ts
    - src/lib/chart-config.ts

frontend-developer:
  skills:
    - chart-ui-controls
    - timeframe-selector
    - drawing-tools-ui
  workflow:
    controls:
      - timeframe: "buttons: 1m 5m 15m 1h 4h 1d"
      - chart-type: "candles, line, area"
      - studies: "add/remove indicators"
      - fullscreen: "expand to fill"
  outputs:
    - src/components/charts/timeframe-selector.tsx
    - src/components/charts/chart-controls.tsx
    - src/components/charts/study-selector.tsx
```

**Commit**: `feat(charts): TradingView Lightweight Charts + real-time updates`

---

#### Chunk 5.2: Signal Feed Integration
**Primary Agent**: AI Engineer (`/.ai/engineering/ai-engineer.md`)
**Supporting**: Frontend Developer (`/.ai/engineering/frontend-developer.md`)

**Skill Workflows**:
```yaml
ai-engineer:
  skills:
    - signal-processing
    - confidence-filtering
    - alert-system
  workflow:
    feed:
      - sources: "neural signals from backend"
      - filter: "min confidence 0.7"
      - sort: "newest first"
      - limit: "last 50 signals"
    actions:
      - one-click: "create order from signal"
      - dismiss: "remove from feed"
      - details: "view signal metadata"
  outputs:
    - src/components/signals/signal-feed.tsx
    - src/components/signals/signal-card.tsx
    - src/components/signals/signal-filter.tsx
    - src/hooks/use-signals.ts
```

**Commit**: `feat(signals): neural signal feed with one-click order`

---

### Phase 3.5: Dashboard Assembly (Day 9)

#### Chunk 6.1: Main Dashboard Page
**Primary Agent**: Frontend Developer (`/.ai/engineering/frontend-developer.md`)
**Supporting**: UI Designer (`/.ai/design/ui-designer.md`)

**Skill Workflows**:
```yaml
frontend-developer:
  skills:
    - page-composition
    - responsive-grid
    - performance-optimization
  workflow:
    layout:
      desktop: "3-column grid"
      tablet: "2-column grid"
      mobile: "1-column stack"
    sections:
      - top: "PortfolioSummary + KeyMetrics"
      - main: "Chart (60%) + OrderEntry (40%)"
      - side: "PositionTable + Watchlist + SignalFeed"
  outputs:
    - src/app/dashboard/page.tsx
    - src/components/dashboard/portfolio-summary.tsx
    - src/components/dashboard/key-metrics.tsx
    - src/components/dashboard/dashboard-grid.tsx

ui-designer:
  skills:
    - visual-hierarchy
    - spacing-system
    - component-spacing
  workflow:
    spacing:
      - gap: "16px between cards"
      - padding: "24px container"
      - card-padding: "16px internal"
```

**Commit**: `feat(dashboard): main dashboard layout with responsive grid`

---

### Phase 3.6: Polish & Testing (Day 10)

#### Chunk 7.1: Dark Mode Polish
**Primary Agent**: Frontend Developer (`/.ai/engineering/frontend-developer.md`)
**Supporting**: UI Designer (`/.ai/design/ui-designer.md`)

**Skill Workflows**:
```yaml
frontend-developer:
  skills:
    - dark-mode-implementation
    - theme-persistence
    - system-preference-detection
  workflow:
    implementation:
      - provider: "next-themes"
      - toggle: "in header"
      - persist: "localStorage"
      - sync: "with system preference"
    coverage:
      - all-components: "dark: variants"
      - charts: "dark theme config"
      - syntax: "consistent dark: prefix"
  outputs:
    - src/components/theme-toggle.tsx
    - src/hooks/use-theme.ts (updated)
    - src/styles/dark-mode.css
```

**Commit**: `feat(ui): dark mode with persistence + system sync`

---

#### Chunk 7.2: Mobile Responsive
**Primary Agent**: Frontend Developer (`/.ai/engineering/frontend-developer.md`)
**Supporting**: UX Researcher (`/.ai/design/ux-researcher.md`)

**Skill Workflows**:
```yaml
frontend-developer:
  skills:
    - mobile-first-responsive
    - touch-optimization
    - hamburger-navigation
  workflow:
    breakpoints:
      - mobile: "< 640px"
      - tablet: "640px - 1024px"
      - desktop: "> 1024px"
    optimizations:
      - tap-targets: "min 44px"
      - font-sizes: "16px min on mobile"
      - tables: "horizontal scroll"
      - charts: "simplify on mobile"
  outputs:
    - src/components/layout/mobile-nav.tsx
    - src/components/trading/order-entry-mobile.tsx
    - src/hooks/use-breakpoint.ts

ux-researcher:
  skills:
    - mobile-ux-patterns
    - hft-mobile-analysis
  workflow:
    patterns:
      - robin-hood-mobile: "clean, thumb-friendly"
      - gestures: "swipe to dismiss"
      - quick-actions: "FAB for order"
```

**Commit**: `feat(ui): mobile responsive with touch optimization`

---

#### Chunk 7.3: Test Suite
**Primary Agent**: Test Writer Fixer (`/.ai/engineering/test-writer-fixer.md`)
**Supporting**: Performance Benchmarker (`/.ai/testing/performance-benchmarker.md`)

**Skill Workflows**:
```yaml
test-writer-fixer:
  skills:
    - unit-test-creation
    - integration-test-setup
    - component-testing
    - mock-implementation
  workflow:
    unit-tests:
      - api-client: "mock axios"
      - stores: "mock zustand"
      - utils: "pure function tests"
    integration:
      - order-flow: "login → place order → verify"
      - websocket: "connect → receive → update"
    component:
      - order-entry: "render, fill, submit"
      - position-table: "sort, filter, update"
  outputs:
    - src/__tests__/api-client.test.ts
    - src/__tests__/auth-store.test.ts
    - src/__tests__/order-store.test.ts
    - src/__tests__/order-entry.test.tsx
    - src/__tests__/position-table.test.tsx
    - src/__tests__/websocket.test.ts

performance-benchmarker:
  skills:
    - performance-testing
    - load-testing
    - benchmark-automation
  workflow:
    benchmarks:
      - render-time: "Dashboard < 2s"
      - table-scroll: "60fps with 1000 rows"
      - chart-update: "< 100ms per tick"
      - order-submit: "< 1.5s end-to-end"
  outputs:
    - src/__tests__/performance/dashboard.perf.test.ts
    - src/__tests__/performance/table.perf.test.ts
    - src/__tests__/performance/chart.perf.test.ts
```

**Commit**: `test(dashboard): comprehensive test suite + performance benchmarks`

---

## Part 2: Execution Command Reference

### Validation Commands (Per Chunk)

```bash
# Before EVERY commit:
npm run lint              # ESLint
npm run type-check        # TypeScript
npm run build             # Next.js build
npm run test:unit         # Jest unit tests

# Performance validation:
npm run test:perf         # Performance benchmarks
```

### Commit Message Pattern

```bash
# Format: type(scope): description

feat(dashboard): initialize Next.js with shadcn/ui
feat(ui): design system with shadcn components + dark mode
feat(store): zustand stores with slice pattern
feat(api): axios client with JWT interceptor
feat(websocket): auto-reconnecting WebSocket client
feat(layout): responsive dashboard shell
feat(trading): order entry form with validation
feat(trading): sortable position table
feat(market): real-time price ticker
feat(charts): TradingView charts integration
feat(signals): neural signal feed
feat(dashboard): main dashboard layout
test(dashboard): comprehensive test suite
```

---

## Part 3: Handoff Specifications

### To AI Engineer + Frontend Developer

**From Rapid Prototyper**:
```markdown
Handoff Package:
- [ ] Project initialized with Next.js + shadcn/ui
- [ ] Tailwind configured with custom theme
- [ ] All dependencies installed (zustand, axios, charts)
- [ ] TypeScript strict mode enabled
- [ ] Placeholder pages for all routes
- [ ] Mock data generators available
```

### To Backend Architect

**From AI Engineer**:
```markdown
API Calibration Request:
- [ ] WebSocket message rate: 1000 msg/sec
- [ ] Connection limit: 100 concurrent
- [ ] JWT expiry: 24 hours
- [ ] Rate limiting: 100 req/min per IP
- [ ] CORS origins: http://localhost:3000
```

### To Test Writer Fixer

**From Frontend Developer**:
```markdown
Test Requirements:
- [ ] Order entry form validation
- [ ] Position table sorting/filtering
- [ ] Price ticker real-time updates
- [ ] Chart data synchronization
- [ ] WebSocket reconnection
- [ ] Performance: <100ms chart update
```

---

## Part 4: Risk Mitigation Checklist

### Per Chunk Validation

- [ ] Max 3 files changed per commit
- [ ] All tests pass before commit
- [ ] No TypeScript errors
- [ ] ESLint warnings resolved
- [ ] Feature flagged if incomplete
- [ ] Mobile responsive tested
- [ ] Dark mode verified

### Build Safety

- [ ] No large dependency additions (>5MB)
- [ ] WebSocket connection pooling
- [ ] Chart memory cleanup
- [ ] Table virtualization for large lists
- [ ] Image optimization

---

## Part 5: Next Phase Preview

### Phase 4: Integration & Security Hardening

**Agents**: Backend Architect, Test Writer Fixer, AI Engineer
**Focus**: E2E tests, security audit, performance optimization
**From Original Roadmap**: Merge Phase 7 (Security Engineering)

### Phase 5: Production Deployment

**Agents**: DevOps Automator, Backend Architect, Project Shipper
**Focus**: CI/CD, Docker, Kubernetes, monitoring
**From Original Roadmap**: Merge Phase 3 (CI/CD Enforcement)

---

**Ready for Execution**: All agent skills mapped, chunks defined, validation gates set.

**Next Action**: Begin Chunk 1.1 - Rapid Prototyper initializes dashboard project.
