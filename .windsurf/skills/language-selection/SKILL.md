# Language Selection Skill

## When to activate
Load when: the agent must decide which programming language, framework, or
tooling to use for a task, or when switching languages mid-task.

## Language Decision Matrix — This Project

| Layer | Language | Why | Model to use |
|-------|----------|-----|-------------|
| Trading engine (HFT) | **Rust** | <100ns latency, memory safety, no GC | coder (qwen2.5-coder) |
| Strategy / signal logic | **Python** | NumPy/Pandas, rapid iteration | coder local or balanced |
| Infrastructure / CI | **bash/zsh** | macOS native, POSIX portable | fast (gemma3:1b) |
| Frontend dashboards | **TypeScript/React** | Type safety, component ecosystem | coder |
| Config / schemas | **YAML / JSON** | Human-readable, Serde compatible | fast |
| Notebooks / research | **Python (Jupyter)** | Exploratory, visualisation | balanced |
| Protocol buffers | **Proto3** | Cross-language contracts | balanced |

## Language-Specific Rules

### Rust (primary — packages/*)
- No `unwrap()` in production paths — use `?` or `match`
- No `Box<dyn Error>` in domain logic — use `thiserror`
- Async: `tokio` only — no blocking I/O in async tasks
- Fixed-point arithmetic for money: `rust_decimal` or `AtomicI64 × 1e-8`
- Always run `cargo check` after every edit before proceeding

### Python (src/*, scripts/*)
- Type hints required on all public functions
- No mutable global state
- Use `pathlib.Path` not `os.path`
- `requirements.txt` with pinned versions
- Never mix Python paths with Rust crate paths

### TypeScript/React
- `strict: true` in tsconfig
- No `any` types
- Components ≤ 200 lines; split if larger
- Tailwind for styling; no inline styles

### bash/zsh (scripts/*)
- `set -euo pipefail` at top of every script
- No hardcoded paths — use `$HOME` and `$(command -v tool)`
- Quote all variables: `"$var"` not `$var`
- No PowerShell — this machine is macOS

## Language Feature Usage by Task

| Task | Language | Pattern |
|------|----------|---------|
| Parse JSON | Rust: `serde_json` / Python: `json` | never eval() |
| HTTP calls | Rust: `reqwest` / Python: `httpx` | async only |
| File I/O | Rust: `tokio::fs` / Python: `aiofiles` | async only in hot path |
| Concurrency | Rust: `tokio::mpsc` / Python: `asyncio` | no threads in hot path |
| DB queries | Rust: `sqlx` / Python: `asyncpg` | prepared statements only |
| Embeddings | Python: `sentence-transformers` + `nomic-embed-text` via Ollama | local first |

## When Language Is Ambiguous

1. Check existing codebase: `grep -r "fn \|def \|function " packages/ src/ | head`
2. Match the dominant language in the affected module
3. If adding new functionality: prefer the language already used in that directory
4. Never introduce a new language dependency without logging the decision in `GENESIS_ROADMAP.md`
