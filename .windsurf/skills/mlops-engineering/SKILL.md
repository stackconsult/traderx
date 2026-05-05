# MLOps Engineering Skill

## When to activate
Load when: model serving, A/B testing strategies, model versioning, drift detection,
feature stores, production ML deployment, or local Ollama model management.

## Model Serving Architecture — This System

```
Strategy Signals (Python/Rust)
    ↓
┌─────────────────────────────────────────┐
│  Model Router                           │
│  fast (< 100ms)  → Ollama gemma3:1b     │
│  code  (< 500ms) → Ollama qwen2.5-coder │
│  think (< 5s)    → Gemini Pro           │
│  embed           → nomic-embed-text     │
└─────────────────────────────────────────┘
    ↓
mem0 (short-term cache) ←→ pgvector (long-term)
    ↓
Response → n8n workflow trigger (if action needed)
```

## Local Model Management (Ollama)

```bash
# Check what's available
curl -s http://127.0.0.1:11434/api/tags | python3 -c \
  "import json,sys; [print(m['name'], m['size']//1e9, 'GB') for m in json.load(sys.stdin)['models']]"

# Pull a model (background)
OLLAMA_MODELS=/Volumes/GenesisModels/ollama-models ollama pull qwen2.5-coder:1.5b &

# Test inference latency
time curl -s http://127.0.0.1:11434/api/generate \
  -d '{"model":"gemma3:1b","prompt":"hello","stream":false}' | python3 -c \
  "import json,sys; d=json.load(sys.stdin); print('tokens/s:', d.get('eval_count',0)/max(d.get('eval_duration',1)/1e9,0.001))"

# Model benchmark — pick best for task
for model in gemma3:1b qwen2.5-coder:1.5b; do
  echo "=== $model ===" && time ollama run $model "explain tokio::sync::Mutex in one sentence"
done
```

## A/B Testing for Strategies

```python
# Route 10% of signals to new strategy, 90% to baseline
import random

def route_signal(signal: dict) -> str:
    if random.random() < 0.10:
        return "strategy_v2"  # challenger
    return "strategy_v1"      # baseline

# Track metrics per variant in pgvector
def log_result(variant: str, pnl: float, latency_ns: int):
    db.execute("""
        INSERT INTO strategy_ab_results (variant, pnl, latency_ns, ts)
        VALUES ($1, $2, $3, now())
    """, variant, pnl, latency_ns)
```

## Model Drift Detection

```python
# Compare embedding distributions over time
from scipy.spatial.distance import cosine

def detect_drift(baseline_embeddings, current_embeddings, threshold=0.15):
    baseline_centroid = np.mean(baseline_embeddings, axis=0)
    current_centroid = np.mean(current_embeddings, axis=0)
    drift = cosine(baseline_centroid, current_centroid)
    if drift > threshold:
        # Log to GENESIS_ROADMAP.md + trigger re-evaluation
        return True, drift
    return False, drift
```

## Feature Store Pattern

```sql
-- pgvector feature store
CREATE TABLE features (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol TEXT NOT NULL,
    timestamp_ns BIGINT NOT NULL,
    embedding VECTOR(768),  -- nomic-embed-text dimensions
    feature_json JSONB,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX ON features USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 100);
```

## n8n MLOps Workflows

Genesis agent uses n8n for ML lifecycle automation:
- **Nightly retraining**: n8n cron → fetch new fills → retrain → evaluate → promote if better
- **Drift alerts**: n8n monitor embedding distances → Slack alert if drift > threshold
- **Model registry**: n8n workflow tracks model versions + performance in Notion/Airtable
- **A/B result analysis**: n8n aggregates variant metrics → generate report → email

## Validation Gate

Before any model change goes to production:
```bash
# Run strategy backtest
cargo test --package oms-engine backtest_strategy -- --nocapture

# Compare Sharpe ratios
python3 src/evaluate_strategy.py --model new --baseline current --min-sharpe 1.5
```
