---
description: Parallel fan-out quality gate — code review + security audit + test coverage → go/no-go decision before any merge or deploy
---

# /ship — Parallel Fan-Out Quality Gate

Run this workflow before merging any branch or deploying any stream of the BAM build.
It fans out three specialist personas in parallel and synthesizes a single go/no-go verdict.

## Pattern

```
/ship
  ├── (parallel) code-reviewer    → review report    (.windsurf/agents/code-reviewer.md)
  ├── (parallel) security-auditor → audit report     (.windsurf/agents/security-auditor.md)
  └── (parallel) test-engineer    → coverage report  (.windsurf/agents/test-engineer.md)
                  ↓
        merge phase (main agent)
                  ↓
        go/no-go decision + rollback plan
```

Each sub-agent operates on the **same diff** but produces a **different perspective**.
They have no dependencies on each other → genuine parallelism.

---

## Step 1 — Stage the diff for review

```bash
git diff HEAD~1 --stat
git diff HEAD~1 -- . ':(exclude)*.lock' ':(exclude)*.sum'
```

Confirm the diff is scoped to one logical change (<300 lines). If larger, split first.

---

## Step 2 — Fan out in parallel (issue all three in one turn)

Invoke all three personas simultaneously by sending this prompt block to Windsurf:

```
I need a parallel quality gate on the staged diff above.

Run these three reviews simultaneously:

[code-reviewer]: Five-axis review (correctness, readability, architecture, security, performance).
Read .windsurf/agents/code-reviewer.md for your persona and output format.
Focus on: Rust hot-path correctness, no heap allocation in signal path, RiskBus bypass risk.

[security-auditor]: OWASP + dependency CVE scan.
Read .windsurf/agents/security-auditor.md for your persona and output format.
Focus on: API key exposure, Polygon.io WebSocket auth, QuestDB credentials, Cargo.toml CVEs.

[test-engineer]: Coverage gap analysis + Prove-It pattern.
Read .windsurf/agents/test-engineer.md for your persona and output format.
Focus on: RiskBus unit tests, SHM bridge integration tests, p50/p99/p999 latency regression tests.

Return all three reports, then synthesize a go/no-go verdict.
```

---

## Step 3 — Merge phase (main agent synthesizes)

After all three reports are returned, apply this decision matrix:

| Condition | Decision |
|-----------|----------|
| Any Critical finding (any persona) | ❌ NO-GO — fix before merge |
| Any High security finding | ❌ NO-GO — fix before merge |
| Missing regression test for changed hot path | ❌ NO-GO — add test first |
| Only Important/Suggestion findings | ✅ GO — address in follow-up task |
| All three personas: no Critical/High | ✅ GO — merge approved |

---

## Step 4 — On GO: commit and push

```bash
# Atomic commit with descriptive message
git add -p                          # review each hunk before staging
git commit -m "type(scope): description"
git push origin feature/github-mcp-setup
```

Commit message format: `type(scope): description`
Types: `feat`, `fix`, `refactor`, `test`, `chore`, `perf`

---

## Step 5 — On NO-GO: fix and re-run

1. Address every Critical finding
2. Address every High security finding
3. Add any required missing tests
4. Run `cargo check --package oms-engine` — error count must not increase
5. Re-run `/ship` from Step 1

---

## Rollback plan (post-deploy)

If a regression is detected after merge:

```bash
git log --oneline -5                        # identify last known-good commit
git revert HEAD                             # create revert commit (preferred)
# or
git reset --hard <last-good-hash>           # only on unshared branches
git push origin feature/github-mcp-setup
```

---

## BAM-specific guard lines (enforced by code-reviewer)

- ❌ No `unwrap()` in hot paths (`src/signal_router`, `src/risk_bus`, `src/oms`)
- ❌ No heap allocation inside `route_signal()` or `check_symbol()`
- ❌ No floating point in order quantity calculations (use `Decimal`)
- ❌ No `RiskBus` bypass — all orders MUST pass through `risk_bus.check_symbol()`
- ❌ No blocking I/O inside tokio tasks
- ❌ No secrets committed — scan with `git diff --staged | grep -iE "key|secret|token|password"`
