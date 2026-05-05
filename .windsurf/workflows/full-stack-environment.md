---
description: Genesis running its own full-stack environment — outside dev workflows, including self-learning, self-improving, scheduling, channels, and autonomous operation
---

# /full-stack-environment — Genesis Autonomous Runtime

Genesis is not only a coding agent — it is a **universal autonomous agent** capable of running
its own environment independently. This workflow defines how to configure and operate Genesis
in full-stack autonomous mode for tasks beyond the standard dev loop.

---

## What Full-Stack Autonomous Means

```
Standard dev mode:          User → Windsurf → Genesis → code
Full-stack autonomous:      Genesis → scheduler → tools → actions → channels → reports → back
```

Genesis can:
- Run scheduled jobs (monitoring, reports, backups) without user intervention
- Manage its own task queue via the `todo` tool
- Dispatch results to Telegram/Discord/Slack
- Execute research, analysis, and document generation autonomously
- Self-improve by running `/self-audit` on a schedule
- Manage its own memory via JOURNAL.md + vault

---

## Layer 1: Persistent Server Mode

Genesis must be running as a server for VS Code extension and channel access:

```bash
# Terminal 1 (keep running — add to login items or launchd)
genesis --server \
  --transport ws \
  --listen 127.0.0.1:7700 \
  --cwd /Users/kirtissiemens/CascadeProjects/traderx-repo \
  --mode auto \
  --mcp-config /Users/kirtissiemens/CascadeProjects/traderx-repo/genesis.mcp.json
```

**Mode `auto`** for server: auto-approves reads, writes, and shell — only asks for destructive operations.
Switch to `normal` if you want manual approval of file writes.

---

## Layer 2: Scheduled Autonomous Tasks

```bash
# Launch Genesis scheduler (runs background jobs)
genesis --scheduler

# One-shot scheduler run (for cron)
genesis --scheduler --scheduler-once
```

### Configuring scheduled tasks

Add to `~/.genesis/config/global.json` under `"scheduler"`:

```json
{
  "scheduler": {
    "jobs": [
      {
        "name": "daily-self-audit",
        "schedule": "0 9 * * *",
        "task": "Run /self-audit workflow. Check skill coverage, identify gaps, propose additions. Write report to JOURNAL.md.",
        "provider": "anthropic",
        "model": "claude-sonnet-4-6"
      },
      {
        "name": "cargo-health-check",
        "schedule": "*/30 * * * *",
        "task": "Run cargo check --package oms-engine. If error count increased since last run, alert via channel_send.",
        "provider": "anthropic",
        "model": "claude-haiku-3-5"
      },
      {
        "name": "upstream-skills-check",
        "schedule": "0 8 * * 1",
        "task": "Check addyosmani/agent-skills for new releases. If updated, run /sync-upstream-skills and commit.",
        "provider": "anthropic",
        "model": "claude-haiku-3-5"
      },
      {
        "name": "dependency-cve-scan",
        "schedule": "0 7 * * *",
        "task": "Run cargo audit. If new Critical or High CVEs found, write report to JOURNAL.md and alert.",
        "provider": "anthropic",
        "model": "claude-haiku-3-5"
      }
    ]
  }
}
```

---

## Layer 3: Self-Learning Loop

Genesis continuously improves itself through this cycle:

```
Execute task
    ↓
Observe: did it go smoothly or was there friction?
    ↓
Identify: what skill/knowledge was missing?
    ↓
Generate: draft SKILL.md for the gap
    ↓
Validate: does the new skill actually prevent the friction?
    ↓
Install: add to .windsurf/skills/ + .windsurfrules if always-active
    ↓
Commit: chore(skills): add [skill-name]
    ↓
Repeat on next execution
```

### Self-learning triggers (automatic)

| Trigger | Action |
|---------|--------|
| Same error occurs twice | `/self-audit` → add remediation skill |
| New domain added to codebase | `/self-audit` → check domain coverage |
| Regression introduced | Add regression-prevention step to relevant skill |
| New CVE category found | Add to `/security-gate` workflow |
| Swarm produces low-quality output | Improve swarm role definitions in `GENESIS_AGENT.md` |

### Self-improvement targets (queued)

From `GENESIS_AGENT.md` — skills to build for this project:

| Domain | Skill to build | Trigger |
|--------|---------------|---------|
| FPGA/VHDL | `fpga-hdl-development` | Stream 5 starts |
| Assembly hot-path | `low-level-optimization` | Stream 5 starts |
| Financial compliance | `regulatory-compliance` | Pre-launch (G6) |
| Aeron messaging | `aeron-messaging-patterns` | Stream 2 starts |
| QuestDB time-series | `time-series-storage` | Stream 2 starts |

---

## Layer 4: Subagent Delegation

For independent parallel tasks, Genesis uses subagents (lighter than full swarm):

```bash
# Example: analyze multiple files in parallel
genesis --task "
  Spawn 3 subagents:
  1. Analyze packages/oms-engine/src/risk_bus.rs for performance issues
  2. Analyze packages/oms-engine/src/signal_router.rs for correctness issues
  3. Analyze src/ Python strategies for security issues
  Merge findings into a single report.
" --agents 3 --cycles 4
```

Each subagent gets workspace isolation — changes don't bleed between agents.

---

## Layer 5: Channel Integration (Optional)

When Genesis is running as a server, connect messaging channels for remote operation:

```bash
# In genesis TUI or via /secrets:
# Set channel tokens via vault first
/secrets unlock your-passphrase
/secrets set TELEGRAM_BOT_TOKEN your-token

# Then add channel in config:
/channels add telegram \
  --command "node channels/telegram/index.js" \
  --env "TELEGRAM_BOT_TOKEN=$TELEGRAM_BOT_TOKEN"
```

Once connected, you can send messages to Genesis from Telegram/Discord/Slack:
- `/status` — current branch, errors, gate
- `/model claude-opus-4` — switch model
- `/swarm Build the RiskBus integration test` — trigger swarm from phone

---

## Layer 6: Web UI (Optional)

For browser access when not at the machine:

```bash
# Terminal 2 (after Genesis server is running)
cd ~/.npm-global/lib/node_modules/genesis-web 2>/dev/null \
  || pip3 show genesis-ai | grep Location  # find web bundle

# Or use Docker (all-in-one):
docker run -p 3000:3000 -p 7700:7700 \
  -e GENESIS_MODE=auto \
  -e ANTHROPIC_API_KEY="$(cat ~/.genesis/secrets/api_key 2>/dev/null || echo 'set-via-vault')" \
  deadraid/genesis
```

Access at `http://localhost:3000` — full feature parity with VS Code sidebar.

---

## Layer 7: macOS Autostart (launchd)

Keep Genesis server running automatically on login:

```bash
# Create launchd plist
cat > ~/Library/LaunchAgents/com.genesis.server.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.genesis.server</string>
  <key>ProgramArguments</key>
  <array>
    <string>/usr/local/bin/genesis</string>
    <string>--server</string>
    <string>--transport</string>
    <string>ws</string>
    <string>--listen</string>
    <string>127.0.0.1:7700</string>
    <string>--cwd</string>
    <string>/Users/kirtissiemens/CascadeProjects/traderx-repo</string>
    <string>--mode</string>
    <string>auto</string>
    <string>--mcp-config</string>
    <string>/Users/kirtissiemens/CascadeProjects/traderx-repo/genesis.mcp.json</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardOutPath</key>
  <string>/tmp/genesis-server.log</string>
  <key>StandardErrorPath</key>
  <string>/tmp/genesis-server.err</string>
</dict>
</plist>
EOF

# Load it
launchctl load ~/Library/LaunchAgents/com.genesis.server.plist
echo "Genesis server registered with launchd — starts on login"
```

---

## Full Environment Status Check

```bash
echo "=== GENESIS FULL-STACK STATUS ==="
echo "Binary:   $(genesis --version 2>/dev/null || echo 'NOT FOUND')"
echo "Server:   $(curl -sf --connect-timeout 1 http://127.0.0.1:7700 && echo 'RUNNING' || echo 'DOWN')"
echo "Vault:    $([ -f ~/.genesis/secrets/vault.v1 ] && echo 'CONFIGURED' || echo 'NOT SET UP')"
echo "Config:   $([ -f ~/.genesis/config/global.json ] && echo 'PRESENT' || echo 'MISSING')"
echo "Skills:   $(ls .windsurf/skills/*.md 2>/dev/null | wc -l | tr -d ' ') installed"
echo "Workflows:$(ls .windsurf/workflows/*.md 2>/dev/null | wc -l | tr -d ' ') installed"
echo "Extension:$(ls ~/.windsurf/extensions/deadraid.genesis* 2>/dev/null | head -1 || echo 'CHECK WINDSURF')"
echo "Node:     $(node --version 2>/dev/null || echo 'NOT ON PATH — source ~/.zshrc')"
```
