# Full Scope Search Skill

## When to activate
Load when: the agent needs to find something in the codebase, docs, web, or memory
before acting. Always search before writing new code or making assumptions.

## Search Priority Ladder (use in order, stop when found)

```
1. mem0 / GENESIS_ROADMAP.md     — have I solved this before?
2. Local codebase grep           — does this already exist?
3. .windsurf/skills/             — is there a skill for this?
4. tinyfish/search (web)         — what does the internet say?
5. tinyfish/fetch (specific URL) — read the full doc/page
6. skills.chat                   — is there an agent skill for this?
7. github.com/addyosmani/agent-skills — upstream skill source
```

## Local Codebase Search Patterns

```bash
# Find all usages of a type/function
grep -rn "TypeName\|fn_name" packages/ src/ --include="*.rs" --include="*.py"

# Find error location
cargo check 2>&1 | grep "^error" | grep " --> " | awk '{print $NF}'

# Find all files modified recently
git diff --name-only HEAD~1

# Find where a module is used
grep -rn "use crate::" packages/oms-engine/src/ | grep "module_name"

# Find TODO/FIXME items
grep -rn "TODO\|FIXME\|HACK\|XXX" packages/ src/ | grep -v ".git"

# Find all public APIs
grep -rn "^pub fn\|^pub async fn" packages/ --include="*.rs" | head -50
```

## Web Search (TinyFish) Patterns

```
# For errors: search the exact error message
tinyfish/search: "rust E0596 cannot borrow Arc as mutable solution"

# For library docs: fetch the official page
tinyfish/fetch: "https://docs.rs/dashmap/latest/dashmap/"

# For skills: check skills.chat
tinyfish/fetch: "https://skills.chat/[skill-name]"

# For packages: check crates.io
tinyfish/fetch: "https://crates.io/crates/[crate-name]"
```

## Windsurf Cascade Search Tools

Use `code_search` for:
- "Find where X is handled"
- "Locate the implementation of Y"
- "What calls Z"

Use `grep_search` for:
- Exact string matching
- Pattern matching across files
- Finding all usages

Use `find_by_name` for:
- Locating files by name/glob
- Finding all files of a type

## Search-Before-Write Rule

**Before writing any new code:**
```bash
# Check if it exists
grep -rn "fn similar_function\|struct SimilarType" packages/ src/

# If found: extend it, don't duplicate
# If not found: write it, then add to GENESIS_ROADMAP.md
```

**Before installing any dependency:**
```bash
# Check if already in Cargo.toml / requirements.txt
grep "crate_name\|package_name" packages/*/Cargo.toml requirements*.txt

# Check for conflicts
cargo tree | grep "crate_name"
```

## Search Result Quality Rules

- **Never act on a single search result** — cross-reference with at least one other source
- **Prefer official docs** over Stack Overflow for API usage
- **Prefer recent results** — check the date; Rust ecosystem changes fast
- **Log findings** — anything non-obvious goes into `GENESIS_ROADMAP.md`
