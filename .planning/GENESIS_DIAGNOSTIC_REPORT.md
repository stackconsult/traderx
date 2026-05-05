# Genesis System Diagnostic Report
**Issued by**: workflow-optimizer (via api-tester + performance-benchmarker + tool-evaluator + test-results-analyzer)  
**Date**: 2026-05-05  
**For**: ai-engineer, backend-architect, devops-automator, rapid-prototyper, test-writer-fixer, trend-researcher, experiment-tracker, project-shipper, studio-producer, infrastructure-maintainer

---

## EXECUTIVE SUMMARY

The Genesis agent stack has a healthy Rust core (0 compile errors) and a well-defined skill + workflow system (69 skills, 32 workflows). However, **5 fatal infrastructure disconnects** were blocking autonomous agent execution. All 5 have been fixed or have clear remediation steps below.

---

## FATAL ERRORS (found by api-tester + tool-evaluator)

### F1 — Docker daemon not running ❌ → User action required
**Impact**: n8n automation layer completely dead. All workflow automation calls fail instantly.  
**Root cause**: Docker Desktop not open on machine boot.  
**Fix**: Open Docker.app → wait for whale icon in menu bar → then run startup script.  
**Proof command**: `docker info --format "{{.ServerVersion}}"`  
**Assigned to**: infrastructure-maintainer, devops-automator

### F2 — `.env` file missing ❌ → FIXED ✅
**Impact**: POSTGRES_PASSWORD undefined → docker-compose refuses to start → postgres, n8n, Redis all dead.  
**Root cause**: `.env` was not created from `.env.example`.  
**Fix applied**: `.env` created with generated secure `POSTGRES_PASSWORD` + `N8N_ENCRYPTION_KEY`.  
**Remaining action**: Fill in `GOOGLE_API_KEY` (Gemini) — without it, `think`/`balanced`/`cheap` model aliases all fail.  
**Proof command**: `grep GOOGLE_API_KEY .env | cut -c1-30`  
**Assigned to**: studio-producer (remind user to set keys)

### F3 — `gemma3:1b` + `qwen2.5-coder:1.5b` not pulled ❌ → Pull initiated ✅
**Impact**: `fast` and `coder` LiteLLM model aliases point to non-existent local models → inference fails for all agents using these tiers.  
**Root cause**: Models need to be pulled explicitly; only `nomic-embed-text` was present.  
**Fix applied**: `ollama pull` started in background for both models.  
**Proof command**: `ollama list | grep -E "gemma3|qwen2.5"`  
**Assigned to**: ai-engineer

### F4 — `litellm` Python package not installed ❌ → Installing ✅
**Impact**: LiteLLM proxy can't start → no unified model routing → agents can't switch between local/cloud models.  
**Root cause**: Python deps never installed after litellm-config.yaml was created.  
**Fix applied**: `pip3 install litellm mem0ai uvicorn fastapi tenacity` running in background.  
**Proof command**: `python3 -c "import litellm; print(litellm.__version__)"`  
**Assigned to**: ai-engineer, devops-automator

### F5 — n8n MCP bridge (`supergateway`) unverified ❌ → Needs Docker first
**Impact**: Windsurf can't connect to n8n as MCP server → Genesis can't call n8n workflows as tools.  
**Root cause**: Depends on Docker being up (F1) + n8n running + `N8N_MCP_TOKEN` set.  
**Fix sequence**:
1. Start Docker Desktop (F1)
2. `docker compose -f docker-compose.n8n.yml up -d`
3. Open http://localhost:5678 → create admin account
4. Settings → Instance-level MCP → Enable → copy Access Token
5. Add to `.env`: `N8N_MCP_TOKEN=<token>`
6. Restart Windsurf → n8n MCP tools appear  
**Proof command**: `curl -s -H "Authorization: Bearer $N8N_MCP_TOKEN" http://localhost:5678/mcp-server/http -d '{"jsonrpc":"2.0","method":"tools/list","id":1}' | python3 -m json.tool | head -10`  
**Assigned to**: devops-automator

---

## WARNINGS (found by performance-benchmarker + test-results-analyzer)

### W1 — PostgreSQL not running locally
**Impact**: pgvector feature store, mem0 long-term memory, and n8n database all unavailable.  
**Fix**: `brew services start postgresql@15` or `pg_ctl start`  
**Assigned to**: infrastructure-maintainer

### W2 — `GOOGLE_API_KEY` not set anywhere
**Impact**: All `think`, `balanced`, `cheap` model tiers → 100% failure rate. Genesis agents using Gemini = completely broken.  
**Fix**: Get key from https://aistudio.google.com/apikey → add to `.env`  
**Assigned to**: studio-producer, ai-engineer

### W3 — `N8N_MCP_TOKEN` placeholder in `.env`
**Impact**: n8n MCP server won't authenticate even when running.  
**Fix**: Set after first n8n login (see F5 sequence above)  
**Assigned to**: devops-automator

### W4 — LiteLLM proxy not running as a service
**Impact**: Model routing only works if LiteLLM is started manually each session.  
**Fix**: Add LiteLLM as a launchctl service (see optimization plan below)  
**Assigned to**: devops-automator, infrastructure-maintainer

---

## TOOL EVALUATION RESULTS (tool-evaluator findings)

| Tool | Status | Score | Recommendation |
|------|--------|-------|----------------|
| Ollama | ✅ Running | 9/10 | ADOPT — pull gemma3:1b + qwen2.5-coder:1.5b |
| LiteLLM | ⚠️ Not installed | 8/10 | ADOPT — install + run as service |
| n8n | ⚠️ Docker needed | 9/10 | ADOPT — start Docker, run compose |
| Supergateway | ⚠️ Unverified | 7/10 | TRIAL — verify after n8n is up |
| mem0ai | ⚠️ Not installed | 8/10 | ADOPT — install + connect to pgvector |
| TinyFish MCP | ✅ Configured | 8/10 | ADOPT — already wired, use for web search |
| PostgreSQL | ❌ Not running | 9/10 | ADOPT — start service |

---

## PERFORMANCE BENCHMARKS (performance-benchmarker baselines)

| Component | Current state | Target | Gap |
|-----------|--------------|--------|-----|
| Rust compile (oms-engine) | ✅ 0 errors | 0 errors | None |
| Ollama embed (nomic) | ✅ Running | <100ms | Verify |
| Ollama inference (fast) | ❌ Model not pulled | <500ms | Pull gemma3:1b |
| Gemini API (balanced) | ❌ No API key | <2s | Set GOOGLE_API_KEY |
| n8n workflow exec | ❌ Not running | <5s | Start Docker |
| Postgres query | ❌ Not running | <50ms p95 | Start postgres |
| MCP tool call (TinyFish) | ✅ Wired | <2s | Active |

---

## SYSTEMATIC ENGINEERING + DEPLOYMENT PLAN

### Phase 1 — Immediate (5 minutes, user action)
```
1. Open Docker Desktop → wait for ready
2. Set GOOGLE_API_KEY in .env (from aistudio.google.com/apikey)
3. brew services start postgresql@15
```

### Phase 2 — Infrastructure (10 minutes, autonomous after Docker up)
```bash
# Start n8n
cd /Users/kirtissiemens/CascadeProjects/traderx-repo
docker compose -f docker-compose.n8n.yml up -d

# Verify Python deps installed
python3 -c "import litellm, mem0; print('deps OK')"

# Verify Ollama models
ollama list | grep -E "gemma3|qwen2.5|nomic"

# Start LiteLLM proxy (model router)
litellm --config litellm-config.yaml --port 4000 &
echo "LiteLLM running at http://localhost:4000"
```

### Phase 3 — Validation (5 minutes)
```bash
# Test model routing
curl -s http://localhost:4000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"fast","messages":[{"role":"user","content":"ping"}]}' \
  | python3 -c "import json,sys; print('fast model:', json.load(sys.stdin)['choices'][0]['message']['content'])"

# Test n8n health
curl -s http://localhost:5678/healthz

# Test Postgres
pg_isready -h localhost -p 5432 -U traderx

# Test TinyFish MCP (already wired in Windsurf)
echo "TinyFish: use /orchestrate with web search task"
```

### Phase 4 — n8n MCP Token (after n8n first login)
```
1. Open http://localhost:5678
2. Create admin account
3. Settings (gear icon) → Instance-level MCP → Enable
4. Copy the Access Token
5. Edit .env: N8N_MCP_TOKEN=<paste_token>
6. Restart Windsurf → n8n tools appear in MCP
```

### Phase 5 — LiteLLM as persistent service
```bash
# Create launchctl plist for auto-start
cat > ~/Library/LaunchAgents/com.genesis.litellm.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>com.genesis.litellm</string>
  <key>ProgramArguments</key>
  <array>
    <string>/Library/Frameworks/Python.framework/Versions/3.13/bin/litellm</string>
    <string>--config</string>
    <string>/Users/kirtissiemens/CascadeProjects/traderx-repo/litellm-config.yaml</string>
    <string>--port</string><string>4000</string>
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>GOOGLE_API_KEY</key><string>REPLACE_WITH_YOUR_KEY</string>
  </dict>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>StandardOutPath</key><string>/tmp/litellm.log</string>
  <key>StandardErrorPath</key><string>/tmp/litellm-error.log</string>
</dict>
</plist>
EOF
launchctl load ~/Library/LaunchAgents/com.genesis.litellm.plist
echo "LiteLLM will now auto-start on login"
```

---

## AGENT ACTION ASSIGNMENTS

| Agent | Action | Priority |
|-------|--------|----------|
| **infrastructure-maintainer** | Start Postgres: `brew services start postgresql@15` | P0 NOW |
| **devops-automator** | Start n8n after Docker up: `docker compose -f docker-compose.n8n.yml up -d` | P0 AFTER DOCKER |
| **devops-automator** | Create LiteLLM launchctl plist (Phase 5 above) | P1 |
| **ai-engineer** | Verify model pulls: `ollama list` + test inference | P1 |
| **ai-engineer** | Wire mem0 to pgvector once Postgres up | P1 |
| **backend-architect** | Create `n8n` schema in Postgres: `CREATE SCHEMA IF NOT EXISTS n8n` | P1 |
| **test-writer-fixer** | Add integration test: verify LiteLLM `/v1/chat/completions` responds | P2 |
| **rapid-prototyper** | Build genesis health-check dashboard (localhost:3001) showing all service states | P2 |
| **trend-researcher** | No blocking issues — can operate via TinyFish + Gemini once API key set | P2 |
| **experiment-tracker** | Log these fixes as Experiment #001 in GENESIS_ROADMAP.md | P2 |
| **project-shipper** | Gate: no ship until F1-F5 all green | GATE |
| **studio-producer** | Coordinate: remind user to set GOOGLE_API_KEY + open Docker | P0 |

---

## SUCCESS CRITERIA (test-results-analyzer gates)

All green before calling Genesis "fully operational":

- [ ] `docker info` exits 0
- [ ] `curl -s http://localhost:5678/healthz` returns `{"status":"ok"}`
- [ ] `ollama list | grep gemma3:1b` shows model
- [ ] `python3 -c "import litellm"` exits 0
- [ ] `pg_isready -h localhost -p 5432` exits 0
- [ ] `curl -s http://localhost:4000/health` returns LiteLLM status
- [ ] `.env` has no `REPLACE_WITH_YOUR` values remaining for GOOGLE_API_KEY
- [ ] n8n MCP token set and Windsurf can list n8n tools

Run all checks: `bash scripts/genesis-startup.sh`
