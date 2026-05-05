# Data Pipeline Engineering Skill

## When to activate
Load when: market data ingestion, stream processing, ETL/ELT, Kafka/Pulsar,
data quality validation, schema evolution, or real-time feed handling.

## Pipeline Architecture — This System

```
Market Data Feed (WebSocket/FIX)
    ↓
Ingest Layer (Rust: tokio + tungstenite)
    ↓
Normalisation + Quality Gate
    ↓
┌──────────────────────────────────┐
│  Stream Router                   │
│  ├── Hot path → OMS (mpsc)       │
│  ├── Analytics → TimescaleDB     │
│  ├── ML features → pgvector      │
│  └── Archive → WAL               │
└──────────────────────────────────┘
    ↓
Strategy Signal Engine (Python/Rust)
    ↓
n8n Workflow Automation (via MCP)
```

## Data Quality Rules (enforce at every boundary)

```rust
// Validate before processing — never trust external data
fn validate_price_update(p: &PriceUpdate) -> Result<(), DataError> {
    if p.price_usd <= 0.0 { return Err(DataError::NegativePrice); }
    if p.price_usd > 1_000_000.0 { return Err(DataError::PriceOutOfRange); }
    if p.timestamp_ns < MIN_VALID_TS { return Err(DataError::StaleTimestamp); }
    Ok(())
}
```

## Schema Evolution Rules

```rust
// Always additive — never remove fields from serialised structs
#[derive(Serialize, Deserialize)]
struct PriceUpdate {
    pub symbol: String,
    pub price_usd: f64,
    pub timestamp_ns: i64,
    // New field: optional for backward compat
    #[serde(default)]
    pub exchange: Option<String>,
}
```

## Backpressure Management

```rust
// Bounded channels = natural backpressure
let (tx, rx) = mpsc::channel::<PriceUpdate>(1024);  // drop or block at 1024

// If sender fills up: log + drop (never block OMS)
match tx.try_send(update) {
    Ok(_) => {},
    Err(TrySendError::Full(_)) => {
        warn!("Price update channel full — dropping update");
        metrics::DROPPED_UPDATES.fetch_add(1, Ordering::Relaxed);
    }
    Err(TrySendError::Closed(_)) => error!("Price channel closed"),
}
```

## n8n Integration for Data Pipelines

n8n handles: webhook triggers, API polling, data transformation, routing to 400+ services.
Genesis agent calls n8n workflows via MCP to:
- Pull earnings data → inject into strategy context
- Trigger reports on market close
- Route alerts to Slack/email
- Transform CSV/JSON data into structured signals

```
# Call n8n workflow from Genesis agent via MCP:
mcp.n8n.execute_workflow(id="market-data-etl", input={symbol, date_range})
```

## Exactly-Once Processing Pattern

```rust
// Track processed message IDs in DashMap
let processed: DashMap<Uuid, ()> = DashMap::new();

fn process_idempotent(msg: &Message) -> Result<()> {
    if processed.contains_key(&msg.id) {
        debug!("Duplicate message {} — skipping", msg.id);
        return Ok(());
    }
    // Process...
    processed.insert(msg.id, ());
    Ok(())
}
```

## Data Lineage Tracking

Every data transformation logs to `GENESIS_ROADMAP.md`:
```markdown
## Data Event — [timestamp]
Source: [feed name]
Transform: [what changed]
Destination: [where it went]
Records: [count]
Quality: [pass/fail ratio]
```
