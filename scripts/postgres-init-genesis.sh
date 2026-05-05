#!/bin/bash
# Create required schemas for Genesis services in the traderx postgres database.
# Run once after postgres is started: bash scripts/postgres-init-genesis.sh
# Safe to re-run (IF NOT EXISTS).

set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Load .env
if [ -f "$REPO/.env" ]; then
  set -a; source "$REPO/.env"; set +a
fi

PGCONN="postgresql://${POSTGRES_USER:-traderx}:${POSTGRES_PASSWORD:-}@${POSTGRES_HOST:-localhost}:${POSTGRES_PORT:-5432}/${POSTGRES_DB:-traderx}"

echo "Connecting to: ${PGCONN//:*@/:***@}"

psql "$PGCONN" <<'SQL'

-- n8n workflow automation schema
CREATE SCHEMA IF NOT EXISTS n8n;

-- pgvector extension (for mem0 embeddings)
CREATE EXTENSION IF NOT EXISTS vector;

-- mem0 / feature store table
CREATE TABLE IF NOT EXISTS public.memory_embeddings (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id    TEXT NOT NULL,
    content     TEXT NOT NULL,
    embedding   VECTOR(768),
    metadata    JSONB DEFAULT '{}',
    created_at  TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_memory_embeddings_agent
    ON public.memory_embeddings(agent_id);

CREATE INDEX IF NOT EXISTS idx_memory_embeddings_vec
    ON public.memory_embeddings USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 50);

-- Genesis telemetry log
CREATE TABLE IF NOT EXISTS public.genesis_telemetry (
    id          BIGSERIAL PRIMARY KEY,
    session_id  UUID,
    event_type  TEXT NOT NULL,
    agent_name  TEXT,
    payload     JSONB DEFAULT '{}',
    ts          TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_genesis_telemetry_ts
    ON public.genesis_telemetry(ts DESC);

SELECT 'Genesis DB schemas initialized OK' AS status;

SQL

echo "✅ Postgres Genesis schemas ready"
