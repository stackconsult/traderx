# Sentrux Integration Plan for TraderX

## Assessment: HIGHLY VALUABLE — Proceed with Integration

### Why Sentrux Fits TraderX

| TraderX Need | Sentrux Solution | Match Score |
|--------------|-----------------|-------------|
| Enforce file size limits (< 200-300 lines) | `max_file_lines` + `no_god_files` rules | 10/10 |
| Detect dependency cycles | `max_cycles = 0` constraint | 10/10 |
| Layer enforcement (state_machine → risk → oms → agents) | Layer rules + boundaries | 10/10 |
| CI quality gates before merge | `sentrux check .` exits 0/1 | 10/10 |
| Real-time agent feedback | MCP server (scan, health, session_start/end) | 10/10 |
| Rust-native, no runtime deps | Pure Rust binary | 10/10 |
| Architecture decay prevention | Quality score tracking (0-10000) | 9/10 |
| Performance gate for risk_bus | Custom performance rules | 9/10 |

**Overall Score: 9.7/10 — Critical infrastructure**

### Integration Architecture

```
┌─────────────────────────────────────────┐
│  Windsurf IDE / Cascade Agent           │
│  ┌─────────────────────────────────┐   │
│  │ MCP Server: sentrux --mcp        │   │
│  │  ├─ scan() → quality score     │   │
│  │  ├─ session_start() → baseline │   │
│  │  ├─ session_end() → regression │   │
│  │  └─ check_rules() → violations │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│  Sentrux Binary (Rust, local)          │
│  ├─ Tree-sitter parser for Rust        │
│  ├─ 5 metrics: modularity, acyclicity  │
│  │              depth, equality,        │
│  │              redundancy              │
│  └─ Rules engine (.sentrux/rules.toml) │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│  GitHub Actions / CI Pipeline           │
│  ├─ sentrux check . (fails on regress) │
│  ├─ cargo check (existing)              │
│  └─ cargo test (existing)              │
└─────────────────────────────────────────┘
```

### Phase 1: Baseline & Rules (This Session)

**Status: IN PROGRESS**

- [x] Create `.sentrux/rules.toml` with TraderX-specific constraints
- [ ] Install sentrux binary (`brew install sentrux/tap/sentrux`)
- [ ] Run initial scan to establish baseline quality score
- [ ] Configure MCP server in Windsurf settings
- [ ] Test session_start/session_end workflow with this agent session

### Phase 2: CI Integration

- [ ] Add `sentrux check .` to GitHub Actions workflow
- [ ] Configure quality score threshold (e.g., must not drop below baseline - 200)
- [ ] Block PRs that violate architectural boundaries
- [ ] Add sentrux report as PR comment artifact

### Phase 3: Agent Workflow Integration

- [ ] Modify agent session start to call `sentrux session_start`
- [ ] Modify agent session end to call `sentrux session_end`
- [ ] Agent reads quality degradation warnings and self-corrects
- [ ] Quality score displayed in agent status updates

## Rules Engine Deep Dive

### Layer Architecture (10 layers defined)

1. **state_machine** (order=0) — Core domain types. Foundation.
2. **risk** (order=1) — RiskBus. All orders must pass through.
3. **oms** (order=2) — Order management.
4. **middleware** (order=3) — Message bus, routing.
5. **ml** (order=4) — Self-healing, model monitoring.
6. **agents** (order=5) — Multi-agent orchestration.
7. **cross_market** (order=6) — Regime detection, correlation.
8. **stability** (order=7) — Reliability assessment.
9. **observability** (order=8) — Metrics, logging, health.
10. **adapters** (order=9) — Exchange adapters. Outermost.

### Key Boundaries

- **adapters → state_machine**: Blocked. Adapters must not bypass layers.
- **agents → oms**: Allowed but monitored. Risk enforcement is runtime.
- **ml → oms**: Monitoring only, no direct order manipulation.

### Performance Gates

- **risk_bus.rs**: <100ns per check (trading system critical path)
- **state_machine/**: <150 lines per file (core types must be lean)

## Specialist Agent Instructions

### backend-architect Agent
**Focus**: Validate that sentrux layer rules match actual dependency graph.
**Task**: Review `packages/oms-engine/src/` directory structure and confirm:
- No adapter imports core types directly without middleware layer
- RiskBus is imported by oms, not bypassed
- ML layer only observes, does not control order flow

### ai-engineer Agent
**Focus**: Optimize MCP integration and agent feedback loop.
**Task**: Design the `sentrux session_start/session_end` integration:
- Cascade agent calls `session_start()` at beginning of work session
- Cascade agent calls `session_end()` before commit
- If score degraded > 200 points, agent must explain why in commit message
- If score degraded > 500 points, block commit and request human review

### frontend-developer Agent
**Focus**: Not applicable for this integration (no frontend component).
**Task**: If future dashboard needed, design quality score visualization.

### test-writer-fixer Agent
**Focus**: Ensure sentrux does not break existing test suite.
**Task**: 
- Run `cargo test` after sentrux integration
- Verify no test files are flagged as "god files" (>500 lines)
- Ensure test data factories stay under 150 lines
- Add integration test for sentrux rules validation

## Next Steps

1. **Install sentrux**: `brew install sentrux/tap/sentrux`
2. **Run baseline**: `sentrux check .` in `packages/oms-engine/`
3. **Fix initial violations**: Address any pre-existing architectural debt
4. **Save baseline**: `sentrux gate --save .`
5. **Configure MCP**: Add to Windsurf MCP server config
6. **Test workflow**: Start new agent session, verify scan/save/gate cycle
7. **Add CI step**: Update `.github/workflows/*.yml`
8. **Monitor**: Track quality score trends across commits

## Risk Assessment

| Risk | Mitigation |
|------|-----------|
| Sentrux false positives on complex Rust generics | Configure `ignore_patterns` for macro-generated code |
| Performance impact on CI | Binary runs in <1s, negligible overhead |
| Agent paralysis from strict rules | Use Warning for non-critical, Error for critical |
| Existing architectural debt blocks progress | Phase 1: baseline + warnings only |

## Success Criteria

- [ ] Quality score baseline established (target: >7000)
- [ ] Zero dependency cycles in oms-engine
- [ ] All files < 300 lines (existing violations documented)
- [ ] CI fails on quality regression > 200 points
- [ ] Agent session workflow includes sentrux gate
- [ ] No trading functionality regression (cargo test passes)
