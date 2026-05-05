---
description: Complete Genesis VS Code extension settings, keyboard shortcuts, permission modes, and runtime configuration — dialled in for the TraderX project
---

# Genesis Settings — Fully Dialled In

Reference for all Genesis configuration on this machine.
All settings are optimised for: Anthropic/Claude, TraderX HFT workspace, normal permission mode.

---

## VS Code Extension Settings (`settings.json`)

Add these to `~/Library/Application Support/Windsurf/User/settings.json`:

```json
{
  "genesis.executablePath": "/usr/local/bin/genesis",
  "genesis.projectPath": "/Users/kirtissiemens/CascadeProjects/traderx-repo",
  "genesis.defaultProvider": "anthropic",
  "genesis.defaultModel": "claude-sonnet-4-6",
  "genesis.permissionMode": "normal",
  "genesis.swarm.agentCount": 5,
  "genesis.swarm.maxCycles": 8,
  "genesis.applyWorkbenchTheme": true,
  "genesis.server.extraArgs": [
    "--mcp-config", "genesis.mcp.json",
    "--mode", "normal"
  ]
}
```

**Permission mode guidance**:
| Mode | Use when |
|------|---------|
| `safe` | Read-only exploration — no writes |
| `normal` | **Default** — asks before writes/shell/network |
| `auto` | Trusted long runs — auto-approves writes, asks for destructive only |
| `yolo` | Never on this project — all auto-approved including destructive |

---

## Genesis CLI Keyboard Shortcuts (TUI mode)

| Key | Action |
|-----|--------|
| `Enter` | Send message |
| `Shift+Enter` | New line in input |
| `Tab` | Autocomplete |
| `Ctrl+C` / `Esc Esc` | Abort current operation |
| `PageUp` / `PageDown` | Scroll history |
| `Ctrl+N` | Show notifications |
| `Ctrl+T` | Toggle sandbox mode |

---

## VS Code Extension Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+Enter` | Send message |
| `Shift+Enter` | New line in input |

---

## TUI Slash Commands (genesis --chat)

| Command | Purpose |
|---------|---------|
| `/help` | Show all commands |
| `/providers` | List configured providers |
| `/provider use anthropic` | Switch to Anthropic |
| `/model` | Open model picker |
| `/session` | Session management |
| `/theme` | Theme picker (6 themes × dark/light) |
| `/thinking` | Toggle extended reasoning |
| `/mode normal` | Set permission mode |
| `/mode auto` | Auto-approve non-destructive |
| `/sandbox` | Toggle sandbox |
| `/mcp list` | List MCP servers |
| `/secrets unlock` | Unlock vault |
| `/secrets status` | Check vault state |
| `/swarm` | Launch swarm from TUI |
| `/diff` | Review file changes |
| `/undo` | Undo last file change |
| `/compact` | Compact conversation history |
| `/context` | Show project context |
| `/cost` | Show token cost for session |
| `/clear` | Clear chat |

---

## Server Mode Launch (for VS Code extension)

```bash
# Standard launch — wired to TraderX project
genesis --server \
  --transport ws \
  --listen 127.0.0.1:7700 \
  --cwd /Users/kirtissiemens/CascadeProjects/traderx-repo \
  --mode normal \
  --mcp-config genesis.mcp.json

# With extended reasoning enabled
genesis --server \
  --transport ws \
  --listen 127.0.0.1:7700 \
  --cwd /Users/kirtissiemens/CascadeProjects/traderx-repo \
  --mode normal \
  --thinking \
  --thinking-budget 8000 \
  --mcp-config genesis.mcp.json
```

Add to `~/.zshrc` as alias:
```bash
alias genesis-traderx='genesis --server --transport ws --listen 127.0.0.1:7700 \
  --cwd /Users/kirtissiemens/CascadeProjects/traderx-repo \
  --mode normal --mcp-config /Users/kirtissiemens/CascadeProjects/traderx-repo/genesis.mcp.json'
```

---

## Swarm Mode Configuration

```bash
# Standard swarm — 5 agents, 8 cycles
genesis --task "YOUR TASK" --agents 5 --cycles 8

# Heavy swarm — full team for complex refactors
genesis --task "YOUR TASK" --agents 8 --cycles 12

# Quick swarm — fast iteration
genesis --task "YOUR TASK" --agents 3 --cycles 4
```

**Swarm role mapping for TraderX**:
| Role | Focus |
|------|-------|
| architect | Design decisions, contract changes, architecture review |
| scout | Research, source reading, gap analysis |
| worker (×3) | Implementation, file writes, test writing |
| reviewer | Code review against 5-axis framework |
| integrator | Merge outputs, resolve conflicts, final commit |

---

## Vault Setup (first time only)

```bash
# In genesis TUI or server mode:
/secrets unlock your-secure-passphrase

# Store Anthropic API key:
/secrets set ANTHROPIC_API_KEY sk-ant-...

# Store GitHub token:
/secrets set GITHUB_TOKEN ghp_...

# Verify:
/secrets status
```

Never store secrets in `.env` files or plaintext. Always use the vault.

---

## API Key Configuration

Genesis auto-discovers keys from the vault. If running without vault (development only):

```bash
export ANTHROPIC_API_KEY="sk-ant-..."  # Anthropic (Claude)
export OPENAI_API_KEY="sk-..."         # OpenAI (optional)
export GITHUB_TOKEN="ghp_..."          # GitHub MCP
```

---

## 6 Genesis Themes

| Theme | Mode | Best for |
|-------|------|---------|
| `default` | dark | General use |
| `ocean` | dark | Long coding sessions |
| `forest` | dark | Low-light environments |
| `default` | light | Daytime + bright monitor |
| `ocean` | light | Presentations |
| `forest` | light | Accessibility |

Switch in TUI: `/theme` → select
Sync with VS Code: `"genesis.applyWorkbenchTheme": true` in settings

---

## Provider Configuration

| Provider | Flag | Best models |
|----------|------|------------|
| Anthropic | `--provider anthropic` | `claude-sonnet-4-6` (default), `claude-opus-4` (complex) |
| OpenAI | `--provider openai` | `gpt-4o`, `o3` |
| Google | `--provider google` | `gemini-2.0-pro` |
| OpenRouter | `--provider openrouter` | Multi-model routing |

For this project: **always use Anthropic** for swarm tasks. OpenRouter acceptable for research scouts.

---

## Runtime Tuning Flags

| Flag | Value | Purpose |
|------|-------|---------|
| `--thinking` | — | Extended reasoning (Claude 3.5+) |
| `--thinking-budget` | `8000` | Token budget for thinking |
| `--reasoning-effort` | `high` | Maximum reasoning depth |
| `--budget` | `5.00` | Cost cap per session in USD |
| `--mode` | `normal` | Permission mode |
| `--agents` | `5` | Swarm agent count |
| `--cycles` | `8` | Max swarm cycles |

---

## Diagnostics

```bash
# Verify genesis is on PATH
which genesis && genesis --version

# Verify npm wrapper
node ~/.npm-global/bin/genesis --version

# Verify Python binary
/Library/Frameworks/Python.framework/Versions/3.13/bin/genesis --version

# Test server connectivity
curl -s --connect-timeout 2 http://127.0.0.1:7700 && echo "server UP" || echo "server DOWN"

# Check genesis config
cat ~/.genesis/config/global.json
```
