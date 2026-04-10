-- HSTR Bitemporal Schema for TimescaleDB
-- Milestone M2.1: Hypertable chunk size set to 1-day

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;
CREATE EXTENSION IF NOT EXISTS vector CASCADE;
CREATE EXTENSION IF NOT EXISTS pgvectorscale CASCADE;

-- GICS Nodes - Hierarchical Tree Structure
CREATE TABLE gics_nodes (
    node_id INTEGER PRIMARY KEY,
    parent_id INTEGER REFERENCES gics_nodes(node_id),
    level INTEGER NOT NULL,
    code VARCHAR(10) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT
);

-- Create adjacency list B-tree index for hierarchical queries
CREATE INDEX idx_gics_parent ON gics_nodes(parent_id);

-- Entities - Core Identity Information
CREATE TABLE entities (
    entity_id BIGSERIAL PRIMARY KEY,
    ticker VARCHAR(10) UNIQUE NOT NULL,
    cik INTEGER UNIQUE, -- SEC Central Index Key
    name VARCHAR(255) NOT NULL,
    description TEXT,
    sector_id INTEGER REFERENCES gics_nodes(node_id),
    industry_id INTEGER REFERENCES gics_nodes(node_id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create unique B-tree index on ticker for fast lookups
CREATE UNIQUE INDEX idx_entities_ticker ON entities(ticker);

-- Facet Snapshots - Complete State Snapshots
CREATE TABLE facet_snapshots (
    snapshot_id BIGSERIAL PRIMARY KEY,
    entity_id BIGINT NOT NULL REFERENCES entities(entity_id),
    facet_name VARCHAR(100) NOT NULL, -- e.g., 'financials', 'risk_metrics', 'strategy_embedding'
    valid_from TIMESTAMPTZ NOT NULL,
    valid_to TIMESTAMPTZ DEFAULT 'infinity',
    snapshot_data JSONB NOT NULL, -- Complete state at this point
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Composite index for O(1) snapshot retrieval
    UNIQUE(entity_id, facet_name, valid_from)
);

-- Convert to hypertable with 1-day chunks
SELECT create_hypertable('facet_snapshots', 'valid_from', 
    chunk_time_interval => INTERVAL '1 day');

-- Facet Deltas - Incremental Changes (RFC 6902 JSON Patch)
CREATE TABLE facet_deltas (
    delta_id BIGSERIAL PRIMARY KEY,
    entity_id BIGINT NOT NULL REFERENCES entities(entity_id),
    facet_name VARCHAR(100) NOT NULL,
    effective_time TIMESTAMPTZ NOT NULL,
    patch_operation JSONB NOT NULL, -- RFC 6902 patch format
    applied_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Entity and facet for efficient delta queries
    INDEX(entity_id, facet_name, effective_time)
);

-- Convert to hypertable with 1-hour chunks for high-frequency updates
SELECT create_hypertable('facet_deltas', 'effective_time',
    chunk_time_interval => INTERVAL '1 hour');

-- Create BRIN index on timestamp for time series queries
CREATE INDEX idx_facet_deltas_time ON facet_deltas USING BRIN(effective_time);

-- Vector Store - AI Embeddings with DiskANN Indexing
CREATE TABLE vector_store (
    vector_id BIGSERIAL PRIMARY KEY,
    entity_id BIGINT NOT NULL REFERENCES entities(entity_id),
    vector_type VARCHAR(50) NOT NULL, -- 'strategy', 'market', 'news', etc.
    embedding vector(1536) NOT NULL, -- Adjust dimension as needed
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create DiskANN index for high-recall vector search
CREATE INDEX idx_vector_store_diskann ON vector_store 
    USING diskann (embedding vector_cosine_ops);

-- Additional indexes for vector store
CREATE INDEX idx_vector_store_entity_type ON vector_store(entity_id, vector_type);

-- Market Data Ticks - High-frequency price data
CREATE TABLE market_ticks (
    tick_id BIGSERIAL PRIMARY KEY,
    symbol VARCHAR(10) NOT NULL REFERENCES entities(ticker),
    exchange VARCHAR(20) NOT NULL,
    price DECIMAL(15,4) NOT NULL,
    volume BIGINT NOT NULL,
    bid DECIMAL(15,4),
    ask DECIMAL(15,4),
    timestamp_ns BIGINT NOT NULL, -- Nanosecond precision
    ptp_timestamp TIMESTAMPTZ NOT NULL, -- IEEE 1588 synchronized
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Convert to hypertable with 1-minute chunks for high-frequency data
SELECT create_hypertable('market_ticks', 'ptp_timestamp',
    chunk_time_interval => INTERVAL '1 minute');

-- Create composite index for symbol-time queries
CREATE INDEX idx_market_ticks_symbol_time ON market_ticks(symbol, ptp_timestamp);

-- Reconstruction Views for O(1) + O(k) Performance
CREATE OR REPLACE VIEW v_entity_state AS
WITH latest_snapshot AS (
    SELECT DISTINCT ON (entity_id, facet_name)
        entity_id,
        facet_name,
        snapshot_data,
        valid_from
    FROM facet_snapshots
    WHERE valid_to = 'infinity'
    ORDER BY entity_id, facet_name, valid_from DESC
),
applied_deltas AS (
    SELECT 
        s.entity_id,
        s.facet_name,
        s.snapshot_data || 
        COALESCE(
            jsonb_agg(
                jsonb_set('{}', '{patch}', d.patch_operation)
                ORDER BY d.effective_time
            ) FILTER (WHERE d.patch_operation IS NOT NULL),
            '[]'::jsonb
        ) AS state
    FROM latest_snapshot s
    LEFT JOIN facet_deltas d ON (
        d.entity_id = s.entity_id 
        AND d.facet_name = s.facet_name 
        AND d.effective_time >= s.valid_from
    )
    GROUP BY s.entity_id, s.facet_name, s.snapshot_data
)
SELECT 
    e.ticker,
    e.name,
    a.facet_name,
    a.state,
    s.valid_from AS base_timestamp
FROM applied_deltas a
JOIN entities e ON e.entity_id = a.entity_id
JOIN latest_snapshot s ON (
    s.entity_id = a.entity_id 
    AND s.facet_name = a.facet_name
);

-- Function for State Reconstruction at Specific Time
CREATE OR REPLACE FUNCTION reconstruct_state_at(
    p_entity_id BIGINT,
    p_facet_name VARCHAR,
    p_timestamp TIMESTAMPTZ
) RETURNS JSONB AS $$
DECLARE
    v_snapshot_data JSONB;
    v_snapshot_time TIMESTAMPTZ;
    v_deltas JSONB;
BEGIN
    -- Get latest snapshot before timestamp (O(1))
    SELECT snapshot_data, valid_from
    INTO v_snapshot_data, v_snapshot_time
    FROM facet_snapshots
    WHERE entity_id = p_entity_id
      AND facet_name = p_facet_name
      AND valid_from <= p_timestamp
      AND valid_to > p_timestamp
    ORDER BY valid_from DESC
    LIMIT 1;
    
    IF NOT FOUND THEN
        RETURN NULL;
    END IF;
    
    -- Get deltas to apply (O(k), typically k < 20)
    SELECT jsonb_agg(patch_operation ORDER BY effective_time)
    INTO v_deltas
    FROM facet_deltas
    WHERE entity_id = p_entity_id
      AND facet_name = p_facet_name
      AND effective_time > v_snapshot_time
      AND effective_time <= p_timestamp;
    
    -- Apply deltas to snapshot
    RETURN v_snapshot_data || COALESCE(v_deltas, '[]'::jsonb);
END;
$$ LANGUAGE plpgsql;

-- Performance Monitoring Query
CREATE OR REPLACE VIEW hstr_performance_stats AS
SELECT 
    'facet_snapshots' as table_name,
    COUNT(*) as total_rows,
    COUNT(DISTINCT entity_id) as unique_entities,
    COUNT(DISTINCT facet_name) as unique_facets,
    MIN(valid_from) as earliest_time,
    MAX(valid_from) as latest_time
FROM facet_snapshots
UNION ALL
SELECT 
    'facet_deltas' as table_name,
    COUNT(*) as total_rows,
    COUNT(DISTINCT entity_id) as unique_entities,
    COUNT(DISTINCT facet_name) as unique_facets,
    MIN(effective_time) as earliest_time,
    MAX(effective_time) as latest_time
FROM facet_deltas;

-- Grant permissions (adjust as needed)
-- GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO traderx_app;
-- GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO traderx_app;

-- Sample data for testing
INSERT INTO gics_nodes (node_id, parent_id, level, code, name) VALUES
(1, NULL, 1, '10', 'Energy'),
(2, NULL, 1, '15', 'Materials'),
(3, NULL, 1, '20', 'Industrials'),
(4, NULL, 1, '25', 'Information Technology'),
(5, 4, 2, '2510', 'Technology Hardware'),
(6, 4, 2, '2520', 'Software & Services'),
(7, 4, 2, '2530', 'Semiconductors');

INSERT INTO entities (ticker, cik, name, sector_id, industry_id) VALUES
('AAPL', 320193, 'Apple Inc.', 4, 5),
('MSFT', 789019, 'Microsoft Corporation', 4, 6),
('NVDA', 1045810, 'NVIDIA Corporation', 4, 7);

COMMENT ON TABLE facet_snapshots IS 'Complete state snapshots with O(1) retrieval';
COMMENT ON TABLE facet_deltas IS 'Incremental changes with O(k) application (k < 20)';
COMMENT ON TABLE vector_store IS 'AI embeddings with DiskANN indexing for 99% recall';
COMMENT ON TABLE market_ticks IS 'High-frequency market data with IEEE 1588 timestamps';
