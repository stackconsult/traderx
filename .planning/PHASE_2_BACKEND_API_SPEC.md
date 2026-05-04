# Phase 2: Backend API Server - Detailed Spec

## Objective
Build a production-ready REST API and WebSocket server that exposes the TraderX OMS engine functionality to the frontend. This is the critical bridge between the Rust backend and the React frontend.

## Success Criteria
- [ ] REST API server runs on port 8080
- [ ] WebSocket server for real-time market data
- [ ] JWT authentication middleware
- [ ] All core endpoints implemented (orders, positions, market data)
- [ ] Frontend can connect and place a paper trade
- [ ] End-to-end: Frontend → API → OMS → Exchange adapter → Response

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  Frontend (React)                                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │ Trading UI  │  │ Portfolio   │  │ Market Data Dashboard   │ │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘ │
│         │                │                     │                 │
│         └────────────────┼─────────────────────┘                 │
│                          │ HTTP/WebSocket                        │
└──────────────────────────┼─────────────────────────────────────┘
                           │
┌──────────────────────────┼─────────────────────────────────────┐
│  API Server (Axum)       │                                      │
│  ┌───────────────────────┴─────────────────────────────────────┐ │
│  │  Router                                                   │ │
│  │  ├── /api/v1/health          → Health check               │ │
│  │  ├── /api/v1/auth            → JWT auth                   │ │
│  │  ├── /api/v1/orders          → Order CRUD                 │ │
│  │  ├── /api/v1/positions       → Positions, P&L             │ │
│  │  ├── /api/v1/market-data     → Prices, order book        │ │
│  │  ├── /api/v1/signals         → Neural signals             │ │
│  │  └── /api/v1/benchmarks     → BM results                │ │
│  └──────────────────────────────────────────────────────────┘ │
│                          │                                      │
│  ┌───────────────────────┴─────────────────────────────────────┐ │
│  │  WebSocket (/ws)                                          │ │
│  │  ├── market_feed:实时价格推送                              │ │
│  │  ├── order_updates:订单状态变更                            │ │
│  │  └── portfolio_updates:仓位变动推送                          │ │
│  └──────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                           │
┌──────────────────────────┼─────────────────────────────────────┐
│  OMS Engine Integration  │                                      │
│  ┌───────────────────────┴─────────────────────────────────────┐ │
│  │  OmsEngineHandle                                            │ │
│  │  ├── submit_order()     → RiskBus → Exchange adapter       │ │
│  │  ├── get_positions()    → Portfolio state                  │ │
│  │  ├── get_market_data()  → Market fabric                   │ │
│  │  └── get_signals()      → Neural engine                    │ │
│  └──────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

---

## Task 2.1: Create API Server Module Structure

### Files to Create
```
src/api_server/
├── mod.rs           # Main module, route aggregation
├── server.rs        # Axum server setup, graceful shutdown
├── auth.rs          # JWT middleware, token validation
├── error.rs         # API error types (ApiError, Result)
├── state.rs         # Shared state (OmsEngineHandle, connections)
├── routes/
│   ├── mod.rs       # Route aggregation
│   ├── health.rs    # Health check endpoint
│   ├── orders.rs    # Order CRUD endpoints
│   ├── positions.rs # Position/P&L endpoints
│   ├── market_data.rs # Market data endpoints
│   ├── signals.rs   # Neural signal endpoints
│   └── auth.rs      # Authentication endpoints
├── websocket.rs     # WebSocket connection handler
└── types.rs         # Request/response DTOs
```

### Key Dependencies (Cargo.toml)
```toml
[dependencies]
axum = { version = "0.7", features = ["ws"] }
tokio = { version = "1.35", features = ["full"] }
tower = { version = "0.4", features = ["limit", "util"] }
tower-http = { version = "0.5", features = ["cors", "trace", "limit"] }
jsonwebtoken = "9"
serde_json = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
```

---

## Task 2.2: Implement Core Types

### Request/Response DTOs (types.rs)

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrderRequest {
    pub symbol: String,
    pub side: String,  // "buy" | "sell"
    pub order_type: String,  // "market" | "limit" | "iceberg"
    pub quantity: f64,
    pub price: Option<f64>,
    pub time_in_force: Option<String>, // "day" | "gtc" | "ioc"
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub status: String,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub filled_quantity: f64,
    pub avg_price: Option<f64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PositionResponse {
    pub symbol: String,
    pub quantity: f64,
    pub avg_entry_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub market_price: f64,
    pub market_value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketDataResponse {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last_price: f64,
    pub volume_24h: f64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NeuralSignalResponse {
    pub signal_id: String,
    pub symbol: String,
    pub direction: String,  // "long" | "short" | "neutral"
    pub confidence: f64,    // 0.0 - 1.0
    pub strategy: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkResponse {
    pub benchmark_name: String,
    pub latency_ns: u64,
    pub throughput_per_sec: f64,
    pub timestamp: String,
}
```

---

## Task 2.3: Implement REST API Endpoints

### Health Check (routes/health.rs)
```rust
pub async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": Utc::now().to_rfc3339()
    }))
}
```

### Orders API (routes/orders.rs)
```rust
// POST /api/v1/orders
pub async fn create_order(
    State(state): State<AppState>,
    Json(req): Json<CreateOrderRequest>,
) -> Result<Json<OrderResponse>, ApiError> {
    // 1. Validate request
    // 2. Convert to internal Order type
    // 3. Submit to OMS via state.oms.handle_order(order).await
    // 4. Return OrderResponse
}

// GET /api/v1/orders
pub async fn list_orders(
    State(state): State<AppState>,
    Query(params): Query<ListOrdersQuery>,
) -> Result<Json<Vec<OrderResponse>>, ApiError> {
    // Return orders from OMS state
}

// GET /api/v1/orders/:id
pub async fn get_order(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> Result<Json<OrderResponse>, ApiError> {
    // Return specific order
}

// DELETE /api/v1/orders/:id
pub async fn cancel_order(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    // Cancel order via OMS
}
```

### Positions API (routes/positions.rs)
```rust
// GET /api/v1/positions
pub async fn get_positions(
    State(state): State<AppState>,
) -> Result<Json<Vec<PositionResponse>>, ApiError> {
    // Get positions from Portfolio module
}

// GET /api/v1/portfolio/summary
pub async fn get_portfolio_summary(
    State(state): State<AppState>,
) -> Result<Json<PortfolioSummaryResponse>, ApiError> {
    // Total P&L, exposure, buying power, etc.
}
```

### Market Data API (routes/market_data.rs)
```rust
// GET /api/v1/market-data/:symbol
pub async fn get_market_data(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<Json<MarketDataResponse>, ApiError> {
    // Get from MarketFabric
}

// GET /api/v1/market-data/order-book/:symbol
pub async fn get_order_book(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(depth): Query<OrderBookQuery>,
) -> Result<Json<OrderBookResponse>, ApiError> {
    // Get order book from adapter
}
```

### Neural Signals API (routes/signals.rs)
```rust
// GET /api/v1/signals
pub async fn get_signals(
    State(state): State<AppState>,
    Query(params): Query<SignalQuery>,
) -> Result<Json<Vec<NeuralSignalResponse>>, ApiError> {
    // Get signals from neural engine
}
```

---

## Task 2.4: Implement WebSocket Server

### WebSocket Handler (websocket.rs)
```rust
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<WsQuery>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, params.token))
}

async fn handle_socket(
    mut socket: WebSocket,
    state: AppState,
    token: String,
) {
    // 1. Authenticate token
    // 2. Subscribe to market data feed
    // 3. Spawn task to forward OMS events to socket
    // 4. Handle incoming client messages
}

// Message types
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "market_data")]
    MarketData { symbol: String, bid: f64, ask: f64, last: f64 },
    
    #[serde(rename = "order_update")]
    OrderUpdate { order_id: String, status: String, filled_qty: f64 },
    
    #[serde(rename = "position_update")]
    PositionUpdate { symbol: String, quantity: f64, unrealized_pnl: f64 },
    
    #[serde(rename = "signal")]
    Signal { signal_id: String, symbol: String, direction: String, confidence: f64 },
    
    #[serde(rename = "error")]
    Error { message: String },
}
```

---

## Task 2.5: Implement Authentication

### JWT Middleware (auth.rs)
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub sub: String,  // user_id
    pub exp: usize,
    pub iat: usize,
}

pub async fn jwt_middleware<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // 1. Extract token from Authorization header
    // 2. Validate with jsonwebtoken
    // 3. Add claims to request extensions
    // 4. Call next handler
}

pub fn generate_token(user_id: &str, secret: &str) -> String {
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .unwrap()
}
```

---

## Task 2.6: Wire Everything Together

### Server Setup (server.rs)
```rust
pub async fn run_server(
    oms: Arc<OmsEngine>,
    config: ApiServerConfig,
) -> anyhow::Result<()> {
    let app_state = Arc::new(AppState {
        oms,
        jwt_secret: config.jwt_secret,
        market_feed: broadcast::channel(100).0,
    });

    let app = Router::new()
        // Health
        .route("/api/v1/health", get(health_check))
        // Auth
        .route("/api/v1/auth/login", post(login))
        // Orders (protected)
        .route("/api/v1/orders", get(list_orders).post(create_order))
        .route("/api/v1/orders/:id", get(get_order).delete(cancel_order))
        .layer(middleware::from_fn(jwt_middleware))
        // Positions (protected)
        .route("/api/v1/positions", get(get_positions))
        .route("/api/v1/portfolio/summary", get(get_portfolio_summary))
        .layer(middleware::from_fn(jwt_middleware))
        // Market Data (public or protected)
        .route("/api/v1/market-data/:symbol", get(get_market_data))
        // WebSocket
        .route("/ws", get(ws_handler))
        // State
        .with_state(app_state)
        // CORS
        .layer(CorsLayer::permissive())
        // Logging
        .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind(&config.bind_address).await?;
    info!("API server listening on {}", config.bind_address);
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

---

## Task 2.7: Integration with OMS Engine

### OmsEngineHandle (state.rs)
```rust
#[derive(Clone)]
pub struct AppState {
    pub oms: Arc<OmsEngine>,
    pub jwt_secret: String,
    pub market_feed: broadcast::Sender<WsMessage>,
}

impl AppState {
    pub async fn submit_order(&self, order: Order) -> Result<OrderResult, OmsError> {
        self.oms.submit_order(order).await
    }
    
    pub async fn get_positions(&self) -> Vec<Position> {
        self.oms.portfolio.get_positions().await
    }
    
    pub async fn get_market_data(&self, symbol: &str) -> Option<MarketData> {
        self.oms.market_fabric.get_data(symbol).await
    }
}
```

---

## Execution Order

```
Step 1: Create module structure
  ├─ Create api_server/ directory
  ├─ Create all .rs files with stubs
  └─ Add pub mod api_server to lib.rs

Step 2: Implement types.rs
  ├─ Define all DTOs
  └─ Define AppState struct

Step 3: Implement error.rs
  ├─ Define ApiError enum
  └─ Implement IntoResponse for ApiError

Step 4: Implement auth.rs
  ├─ JWT claims and validation
  └─ jwt_middleware function

Step 5: Implement routes
  ├─ health.rs (simplest)
  ├─ orders.rs (most critical)
  ├─ positions.rs
  ├─ market_data.rs
  └─ signals.rs

Step 6: Implement websocket.rs
  ├─ WsMessage enum
  ├─ Connection handler
  └─ Message broadcasting

Step 7: Implement server.rs
  ├─ Router setup
  ├─ Middleware stack
  └─ run_server function

Step 8: Create bin/api_server.rs
  ├─ Main entry point
  ├─ Config from env
  └─ Start OMS + API server

Step 9: Wire to frontend
  ├─ Test with curl
  ├─ Create frontend API client
  └─ Test end-to-end
```

---

## Testing Commands

```bash
# Start API server
cargo run --bin api_server

# Health check
curl http://localhost:8080/api/v1/health

# Login
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"test","password":"test"}'

# Create order (with JWT token)
curl -X POST http://localhost:8080/api/v1/orders \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "AAPL",
    "side": "buy",
    "order_type": "market",
    "quantity": 100
  }'

# Get positions
curl http://localhost:8080/api/v1/positions \
  -H "Authorization: Bearer $TOKEN"

# WebSocket connection
wscat -c ws://localhost:8080/ws?token=$TOKEN
```

---

## Verification Steps

1. **Unit Tests**
   - Test each route handler with mock state
   - Test JWT encoding/decoding
   - Test WebSocket message serialization

2. **Integration Tests**
   - Start server, make HTTP requests
   - Connect WebSocket, verify message flow
   - Test error handling (401, 404, 500)

3. **End-to-End**
   - Frontend → API → OMS → Paper trading
   - Verify order lifecycle
   - Verify real-time updates

---

## Definition of Done

- [ ] All API endpoints respond correctly
- [ ] WebSocket broadcasts real-time updates
- [ ] JWT authentication protects private routes
- [ ] Frontend can place and monitor orders
- [ ] Error handling is consistent
- [ ] Logging is comprehensive
- [ ] Documentation is complete

---

## Next Phase

Phase 3: Frontend Trading Dashboard (after API is ready)
