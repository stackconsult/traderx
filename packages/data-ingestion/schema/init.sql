-- TraderX QuestDB Schema
-- Tick data — partitioned by DAY, WAL + DEDUP for idempotent ingestion

CREATE TABLE IF NOT EXISTS ticks (
  ts          TIMESTAMP,
  symbol      SYMBOL   CAPACITY 4096  CACHE,
  exchange    SYMBOL   CAPACITY 64    CACHE,
  price       DOUBLE,
  volume      DOUBLE,
  side        SYMBOL   CAPACITY 4     CACHE,
  bid         DOUBLE,
  ask         DOUBLE,
  bid_size    DOUBLE,
  ask_size    DOUBLE,
  sequence    LONG
) TIMESTAMP(ts)
  PARTITION BY DAY
  WAL
  DEDUP UPSERT KEYS(ts, symbol, exchange, sequence);

-- OHLCV 1-minute bars (materialized by ingestion service)
CREATE TABLE IF NOT EXISTS ohlcv_1m (
  ts          TIMESTAMP,
  symbol      SYMBOL   CAPACITY 4096  CACHE,
  exchange    SYMBOL   CAPACITY 64    CACHE,
  open        DOUBLE,
  high        DOUBLE,
  low         DOUBLE,
  close       DOUBLE,
  volume      DOUBLE,
  vwap        DOUBLE,
  trade_count LONG
) TIMESTAMP(ts)
  PARTITION BY MONTH
  WAL
  DEDUP UPSERT KEYS(ts, symbol, exchange);

-- OHLCV 1-hour bars
CREATE TABLE IF NOT EXISTS ohlcv_1h (
  ts          TIMESTAMP,
  symbol      SYMBOL   CAPACITY 4096  CACHE,
  exchange    SYMBOL   CAPACITY 64    CACHE,
  open        DOUBLE,
  high        DOUBLE,
  low         DOUBLE,
  close       DOUBLE,
  volume      DOUBLE,
  vwap        DOUBLE
) TIMESTAMP(ts)
  PARTITION BY YEAR
  WAL
  DEDUP UPSERT KEYS(ts, symbol, exchange);

-- Order book snapshots (top-10 depth)
CREATE TABLE IF NOT EXISTS order_book (
  ts          TIMESTAMP,
  symbol      SYMBOL   CAPACITY 4096  CACHE,
  exchange    SYMBOL   CAPACITY 64    CACHE,
  bid_1       DOUBLE,  bid_sz_1  DOUBLE,
  bid_2       DOUBLE,  bid_sz_2  DOUBLE,
  bid_3       DOUBLE,  bid_sz_3  DOUBLE,
  bid_4       DOUBLE,  bid_sz_4  DOUBLE,
  bid_5       DOUBLE,  bid_sz_5  DOUBLE,
  ask_1       DOUBLE,  ask_sz_1  DOUBLE,
  ask_2       DOUBLE,  ask_sz_2  DOUBLE,
  ask_3       DOUBLE,  ask_sz_3  DOUBLE,
  ask_4       DOUBLE,  ask_sz_4  DOUBLE,
  ask_5       DOUBLE,  ask_sz_5  DOUBLE,
  spread_bps  DOUBLE,
  mid_price   DOUBLE
) TIMESTAMP(ts)
  PARTITION BY DAY
  WAL
  DEDUP UPSERT KEYS(ts, symbol, exchange);

-- Computed features (written by feature-store pipeline)
CREATE TABLE IF NOT EXISTS features (
  ts              TIMESTAMP,
  symbol          SYMBOL   CAPACITY 4096  CACHE,
  returns_1m      DOUBLE,
  returns_5m      DOUBLE,
  returns_1h      DOUBLE,
  returns_1d      DOUBLE,
  log_returns_1m  DOUBLE,
  realized_vol_5m DOUBLE,
  realized_vol_1h DOUBLE,
  realized_vol_1d DOUBLE,
  rsi_14          DOUBLE,
  macd_signal     DOUBLE,
  bb_upper        DOUBLE,
  bb_lower        DOUBLE,
  bb_width        DOUBLE,
  vwap_dev        DOUBLE,
  volume_zscore   DOUBLE,
  ofi             DOUBLE,
  spread_bps      DOUBLE
) TIMESTAMP(ts)
  PARTITION BY DAY
  WAL
  DEDUP UPSERT KEYS(ts, symbol);

-- Trade executions from OMS (for attribution)
CREATE TABLE IF NOT EXISTS executions (
  ts            TIMESTAMP,
  order_id      STRING,
  symbol        SYMBOL   CAPACITY 4096  CACHE,
  exchange      SYMBOL   CAPACITY 64    CACHE,
  side          SYMBOL   CAPACITY 4     CACHE,
  quantity      DOUBLE,
  fill_price    DOUBLE,
  commission    DOUBLE,
  slippage_bps  DOUBLE,
  strategy_id   SYMBOL   CAPACITY 256   CACHE,
  agent_id      SYMBOL   CAPACITY 256   CACHE
) TIMESTAMP(ts)
  PARTITION BY MONTH
  WAL;
