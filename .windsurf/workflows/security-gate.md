---
description: Mandatory pre-commit security scan — dependency CVE audit, secrets detection, and OWASP surface check before any code touches external systems
---

# /security-gate — Pre-Commit Security Scan

Run this workflow BEFORE any commit that touches:
- External API integrations (Polygon.io, QuestDB, FPGA PCIe)
- Authentication / API key handling
- Dependency additions or upgrades
- Any file in `src/`, `packages/oms-engine/`, `dashboard/`

This is the first line of defense against the **32 known CVEs** (1 critical, 13 high, 16 moderate)
already flagged by GitHub Dependabot on this repository.

---

## Step 1 — Secrets scan (run before every commit)

```bash
# Scan staged diff for secrets
git diff --staged | grep -iE "(api_key|apikey|secret|password|token|bearer|private_key|polygon|questdb)" 

# Scan entire working tree for accidentally committed secrets
grep -rn --include="*.rs" --include="*.py" --include="*.ts" --include="*.env" \
  -E "(api_key|POLYGON_API|QUESTDB_PASS|JWT_SECRET)" \
  packages/ src/ dashboard/src/ \
  --exclude-dir=node_modules --exclude-dir=target

# Check .env files are gitignored
cat .gitignore | grep -E "\.env"
```

**STOP** if any match is found — do not commit until secrets are moved to env vars.

---

## Step 2 — Rust dependency CVE audit

```bash
# Install if not present
cargo install cargo-audit 2>/dev/null || true

# Run audit against RustSec advisory database
cargo audit

# Focus on high/critical only
cargo audit --deny warnings
```

Expected current state: review output against Dependabot report at
`https://github.com/stackconsult/traderx/security/dependabot`

Triage each finding:
| Severity | Action |
|----------|--------|
| Critical | Block — fix before any commit |
| High | Block — fix or add `cargo audit --ignore RUSTSEC-XXXX` with written justification |
| Moderate | Track — schedule fix within current sprint |
| Low | Log — address in next sprint |

---

## Step 3 — Node.js dependency audit (dashboard)

```bash
# From dashboard package root
npm audit --audit-level=high

# Auto-fix safe updates
npm audit fix

# Review what cannot be auto-fixed
npm audit fix --dry-run
```

---

## Step 4 — OWASP surface check (invoke security-auditor persona)

For any change touching auth, external I/O, or user-facing API, run:

```
[security-auditor persona]: Read .windsurf/agents/security-auditor.md

Review the staged diff for:
1. Input Handling — any unsanitized data from Polygon.io WebSocket entering order logic?
2. Authentication — API keys loaded from env vars, not hardcoded?
3. Data Protection — QuestDB credentials, FPGA PCIe access tokens secured?
4. Infrastructure — CORS, CSP headers on dashboard endpoints?
5. Third-Party — Polygon.io webhook signature validation in place?

Output a severity-classified report using the security-auditor template.
```

---

## Step 5 — Hot-path safety checks (HFT-specific)

```bash
# Scan for floating point in order calculation paths
grep -rn --include="*.rs" \
  -E "(f32|f64|as f32|as f64)" \
  packages/oms-engine/src/ | grep -v "#\[cfg(test)\]" | grep -v "//.*f64"

# Scan for heap allocation patterns in hot path
grep -rn --include="*.rs" \
  -E "(Box::new|Vec::new|String::new|format!|to_string\(\))" \
  packages/oms-engine/src/signal_router.rs \
  packages/oms-engine/src/risk_bus.rs \
  packages/oms-engine/src/oms/ 2>/dev/null || true

# Scan for unwrap() in production paths
grep -rn --include="*.rs" \
  "\.unwrap()" \
  packages/oms-engine/src/ | grep -v "#\[cfg(test)\]" | grep -v "tests/"
```

Each match = a guarded-line violation. Fix before committing.

---

## Step 6 — Gate result

All of the following must pass before proceeding to commit:

- [ ] Zero secrets in staged diff
- [ ] Zero Critical CVEs in `cargo audit`
- [ ] Zero High CVEs in `npm audit` (dashboard)
- [ ] security-auditor persona: zero Critical/High findings
- [ ] Zero floating-point in order quantity hot paths
- [ ] Zero `unwrap()` outside test code in production paths
- [ ] `.env` files confirmed in `.gitignore`

If all pass → run `/ship` → commit.
If any fail → fix and re-run this gate from Step 1.
