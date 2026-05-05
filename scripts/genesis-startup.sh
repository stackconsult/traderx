#!/bin/bash
# Genesis Agent Startup Script
# Run this once per machine boot to bring up all Genesis dependencies.
# Usage: bash scripts/genesis-startup.sh
# Safe to re-run — idempotent checks throughout.

set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
ok()   { echo -e "${GREEN}✅ $1${NC}"; }
warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
fail() { echo -e "${RED}❌ FATAL: $1${NC}"; FATAL=1; }
FATAL=0

echo "════════════════════════════════════════"
echo "  Genesis Agent Startup — $(date '+%Y-%m-%d %H:%M')"
echo "════════════════════════════════════════"

# ── 1. .env file ─────────────────────────────
echo ""
echo "── [1/6] Environment config ──"
if [ ! -f "$REPO/.env" ]; then
  warn ".env missing — creating from .env.example"
  cp "$REPO/.env.example" "$REPO/.env"
  warn "EDIT $REPO/.env — set POSTGRES_PASSWORD, N8N_ENCRYPTION_KEY, GOOGLE_API_KEY"
else
  ok ".env exists"
fi

# Check critical keys
MISSING_KEYS=""
for key in POSTGRES_PASSWORD N8N_ENCRYPTION_KEY; do
  val=$(grep "^${key}=" "$REPO/.env" 2>/dev/null | cut -d= -f2-)
  if [ -z "$val" ] || echo "$val" | grep -q "change_me\|your_"; then
    MISSING_KEYS="$MISSING_KEYS $key"
  fi
done
if [ -n "$MISSING_KEYS" ]; then
  warn "Keys need real values in .env:$MISSING_KEYS"
else
  ok "Critical .env keys set"
fi

# ── 2. Ollama ─────────────────────────────────
echo ""
echo "── [2/6] Ollama ──"
if curl -s --max-time 2 http://127.0.0.1:11434/api/tags >/dev/null 2>&1; then
  ok "Ollama running"
  MODELS=$(curl -s http://127.0.0.1:11434/api/tags | python3 -c "import json,sys; print([m['name'] for m in json.load(sys.stdin)['models']])" 2>/dev/null)
  echo "  models: $MODELS"
  # Pull missing models in background
  for model in gemma3:1b qwen2.5-coder:1.5b; do
    if ! echo "$MODELS" | grep -q "$model"; then
      warn "Pulling $model (background)..."
      nohup ollama pull "$model" > /tmp/ollama-pull-${model//[:\/]/-}.log 2>&1 &
    fi
  done
else
  warn "Ollama not running — starting via launchctl"
  launchctl start com.genesis.ollama 2>/dev/null || ollama serve &>/tmp/ollama.log &
  sleep 3
  curl -s --max-time 5 http://127.0.0.1:11434/api/tags >/dev/null 2>&1 && ok "Ollama started" || fail "Ollama failed to start"
fi

# ── 3. PostgreSQL ─────────────────────────────
echo ""
echo "── [3/6] PostgreSQL ──"
if pg_isready -h localhost -p 5432 -U traderx -q 2>/dev/null; then
  ok "PostgreSQL ready"
else
  warn "PostgreSQL not ready — starting"
  brew services start postgresql@15 2>/dev/null \
    || pg_ctl start -D /usr/local/var/postgresql@15 2>/dev/null \
    || pg_ctl start -D /opt/homebrew/var/postgresql@15 2>/dev/null \
    || warn "Cannot auto-start Postgres — start manually: brew services start postgresql@15"
  sleep 2
  pg_isready -h localhost -p 5432 -q 2>/dev/null && ok "PostgreSQL started" || warn "PostgreSQL may need manual start"
fi

# ── 4. Docker + n8n ───────────────────────────
echo ""
echo "── [4/6] Docker + n8n ──"
if docker info >/dev/null 2>&1; then
  ok "Docker daemon running"
  if docker ps --format "{{.Names}}" 2>/dev/null | grep -q genesis-n8n; then
    ok "n8n already running → http://localhost:5678"
  else
    warn "n8n not running — starting"
    if [ -f "$REPO/docker-compose.n8n.yml" ]; then
      # Load env vars for compose
      set -a; source "$REPO/.env" 2>/dev/null || true; set +a
      # n8n needs an encryption key — generate one if missing
      if [ -z "${N8N_ENCRYPTION_KEY:-}" ] || echo "${N8N_ENCRYPTION_KEY:-}" | grep -q "change_me"; then
        N8N_KEY=$(python3 -c "import secrets; print(secrets.token_hex(32))")
        warn "N8N_ENCRYPTION_KEY not set — add to .env: N8N_ENCRYPTION_KEY=$N8N_KEY"
      fi
      docker compose -f "$REPO/docker-compose.n8n.yml" up -d 2>&1 | tail -5
      sleep 5
      curl -s --max-time 5 http://localhost:5678/healthz >/dev/null 2>&1 \
        && ok "n8n started → http://localhost:5678" \
        || warn "n8n starting (may take 30s) → check: docker compose -f docker-compose.n8n.yml logs"
    fi
  fi
else
  fail "Docker Desktop not running — open Docker.app first, then re-run this script"
fi

# ── 5. Supergateway (n8n MCP bridge) ─────────
echo ""
echo "── [5/6] Supergateway (n8n→MCP) ──"
if npx -y supergateway --version 2>/dev/null | grep -qE "[0-9]"; then
  ok "supergateway available"
else
  warn "Installing supergateway..."
  npm install -g supergateway 2>/dev/null && ok "supergateway installed" || warn "supergateway install failed — n8n MCP bridge may not work"
fi

# ── 6. LiteLLM config + venv ───────────────
echo ""
echo "── [6/6] LiteLLM config + venv ──"
if [ ! -d "$REPO/.venv" ]; then
  warn ".venv not found — creating"
  python3 -m venv "$REPO/.venv"
  source "$REPO/.venv/bin/activate"
  pip install litellm uvicorn fastapi tenacity
else
  source "$REPO/.venv/bin/activate"
fi

if python3 -c "import yaml; yaml.safe_load(open('litellm-config.yaml'))" 2>/dev/null; then
  ok "litellm-config.yaml valid"
else
  fail "litellm-config.yaml invalid YAML"
fi

if python3 -c "import litellm" 2>/dev/null; then
  ok "litellm installed in venv"
else
  fail "litellm not installed in venv"
fi

# ── Summary ───────────────────────────────────
echo ""
echo "════════════════════════════════════════"
if [ $FATAL -eq 0 ]; then
  echo -e "${GREEN}✅ Genesis ready (or warnings above need attention)${NC}"
  echo ""
  echo "  Windsurf → MCP servers available:"
  python3 -c "import json; d=json.load(open('.windsurf/mcp_config.json')); print('  ' + ', '.join(d['mcpServers'].keys()))"
  echo ""
  echo "  Ollama: http://127.0.0.1:11434"
  echo "  n8n:    http://localhost:5678"
else
  echo -e "${RED}❌ FATAL errors above — fix before starting Genesis${NC}"
fi
echo "════════════════════════════════════════"
