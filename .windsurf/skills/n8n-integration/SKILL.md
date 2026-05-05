# n8n Integration Skill

## When to activate
Load when: automating any workflow, connecting to external APIs, building pipelines,
generating presentations/reports/slides, triggering actions from Genesis agent,
or turning a natural language command into an automated workflow.

## What n8n Gives Genesis (2026 evidence-based)

n8n self-hosted provides:
- **400+ native integrations**: Slack, Gmail, Notion, Airtable, GitHub, Google Sheets, Postgres, HTTP, etc.
- **AI Agent nodes**: LangChain-based, LLM-callable via n8n's toolsAgent
- **MCP Server**: Genesis calls n8n workflows as MCP tools — no code required
- **MCP Client**: n8n workflows call external MCP servers (TinyFish, Ollama, etc.)
- **Workflow builder**: visual + code; workflows export as JSON for version control
- **Built-in triggers**: webhook, cron, email, database, file watch, chat

## Genesis ↔ n8n Architecture

```
Genesis (Windsurf Cascade)
    │
    ├─ mcp.n8n.search_workflows("report generation")
    ├─ mcp.n8n.execute_workflow(id="generate-report", input={data})
    └─ mcp.n8n.create_workflow_from_code(typescript_code)
            │
            ▼
    n8n Self-Hosted (localhost:5678)
            │
    ┌───────┼──────────────────────────────────────┐
    │       │                                      │
    ▼       ▼                                      ▼
 Postgres  Ollama                           External APIs
 (data)   (local LLM)                  (Slack/Gmail/Notion/etc)
```

## MCP Connection Setup

**n8n as MCP server → Windsurf connects to it:**

In `.windsurf/mcp_config.json` (already configured after setup):
```json
"n8n": {
  "command": "npx",
  "args": ["-y", "supergateway", "--streamableHttp",
           "http://localhost:5678/mcp-server/http",
           "--header", "authorization:Bearer ${env:N8N_MCP_TOKEN}"]
}
```

**Available MCP tools from n8n (after instance-level MCP enabled):**
- `n8n.search_workflows` — find workflows by name/description
- `n8n.get_workflow_details` — inspect a workflow
- `n8n.execute_workflow` — run any enabled workflow
- `n8n.create_workflow_from_code` — create new workflow from TypeScript

## Capability → Workflow Mapping

| Genesis task | n8n workflow to call |
|-------------|---------------------|
| "send a slack message" | `slack-notify` |
| "create a report" | `generate-report` |
| "make slides / presentation" | `generate-slides` (Google Slides / Canva) |
| "schedule this task" | `create-cron-job` |
| "save this to Notion" | `notion-upsert` |
| "email this to someone" | `send-email` |
| "pull data from API X" | `http-request-workflow` |
| "retrain the model" | `ml-retrain-pipeline` |
| "post to social media" | `social-post` |
| "transform this codebase into a presentation" | `code-to-slides` |
| "generate marketing copy" | `marketing-copy-gen` |

## Startup + First-Time Setup

```bash
# 1. Start n8n
docker compose -f docker-compose.n8n.yml up -d

# 2. Wait for healthy
docker compose -f docker-compose.n8n.yml ps

# 3. Open browser
open http://localhost:5678

# 4. Create admin account (first run only)
# 5. Settings → Instance-level MCP → Enable
# 6. Copy Access Token → add to .env as N8N_MCP_TOKEN

# 7. Add n8n MCP server to Windsurf (run once):
npx -y install-mcp@latest http://localhost:5678/mcp-server/http \
  --client windsurf --header "authorization:Bearer $N8N_MCP_TOKEN"

# 8. Restart Windsurf → n8n appears in MCP tools
```

## Workflow as Code (version-controlled in ./n8n/workflows/)

All Genesis-managed n8n workflows are exported as JSON and committed:
```bash
# Export a workflow
curl -H "Authorization: Bearer $N8N_MCP_TOKEN" \
  http://localhost:5678/api/v1/workflows/[id] > n8n/workflows/workflow-name.json

# Import a workflow
curl -X POST -H "Authorization: Bearer $N8N_MCP_TOKEN" \
  -H "Content-Type: application/json" \
  -d @n8n/workflows/workflow-name.json \
  http://localhost:5678/api/v1/workflows
```

## Security Rules

- `N8N_ENCRYPTION_KEY` must be set in `.env` — never hardcoded
- `N8N_MCP_TOKEN` never committed to git — use `.env` only
- MCP access token rotated monthly
- All workflows that touch sensitive data use n8n credentials (encrypted at rest)
- Ollama endpoint in n8n always uses `host.docker.internal:11434` — never external

## Proof of Connection

```bash
# Test n8n is up
curl -s http://localhost:5678/healthz | python3 -c "import json,sys; print('n8n', json.load(sys.stdin)['status'])"

# Test MCP endpoint
curl -s -H "Authorization: Bearer $N8N_MCP_TOKEN" \
  http://localhost:5678/mcp-server/http \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}' | python3 -m json.tool | head -20
```
