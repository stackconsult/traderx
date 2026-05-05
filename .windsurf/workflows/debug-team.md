---
description: Agent Teams competing-hypothesis debugging — for hard bugs where a single agent picks the wrong root cause. Requires CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1
---

# /debug-team — Adversarial Competing-Hypothesis Debugging

Use this workflow when a bug has **multiple plausible root causes** that need to be ruled out, not just investigated.
A single agent will pick the first plausible theory and stop. Agent Teams let three specialists challenge each other.

**Prerequisite**: Agent Teams is experimental. Enable once in your shell:
```bash
export CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1
# Or persist in ~/.claude/settings.json:
# { "env": { "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1" } }
```

---

## When to use this vs /ship

| Situation | Use |
|-----------|-----|
| Known diff, need a verdict | `/ship` (subagent fan-out) |
| Unknown root cause, multiple competing theories | `/debug-team` (Agent Teams) |
| Sequential lifecycle step | User-driven slash commands |

---

## Trigger prompt — paste into Windsurf lead session

Customize `[SYMPTOM]`, `[HYPOTHESIS_A/B/C]`, and `[SCOPE]`:

```
[SYMPTOM]: [describe observable bug — e.g. "SHM bridge drops ~1 in 500 signals under load. No panic, no log error. Started after adding the NVMe pool integration."]

Create an agent team to debug this with competing hypotheses. Spawn three teammates:

  - code-reviewer  — hypothesis: race condition or memory ordering issue
    Investigate: SHM bridge write/read synchronization, atomic ordering,
    any non-SeqCst operations in the hot path.
    Read .windsurf/agents/code-reviewer.md for your review framework.

  - security-auditor — hypothesis: buffer boundary or unsafe block misuse
    Investigate: any `unsafe` blocks in SHM bridge, buffer size calculations,
    pointer arithmetic, mmap region boundaries.
    Read .windsurf/agents/security-auditor.md for your review framework.

  - test-engineer — hypothesis: test gap masking the failure
    Investigate: existing SHM bridge tests, propose a minimal reproducer test
    that would catch a 1-in-500 failure reliably (property-based or stress test).
    Read .windsurf/agents/test-engineer.md for your review framework.

Have teammates message each other directly to challenge each other's theories.
Update findings as consensus emerges. Only converge when two teammates agree
they can disprove the others'. Then report the consensus root cause to me.
```

---

## BAM-specific hypothesis templates

Use these pre-filled versions for common BAM failure modes:

### Template A — Signal drop in hot path
```
[SYMPTOM]: Signal dropped between SignalRouter and OMS — no rejection log, no fill.

Spawn agent team:
- code-reviewer   → hypothesis: mpsc channel backpressure / dropped message
- security-auditor → hypothesis: RiskBus bypass or silent error swallow  
- test-engineer   → hypothesis: missing integration test for channel-full scenario
```

### Template B — Latency spike (p999 outlier)
```
[SYMPTOM]: p999 latency spikes to 50ms+ while p50 stays at 200µs. Intermittent.

Spawn agent team:
- code-reviewer   → hypothesis: lock contention or OS scheduler interference
- security-auditor → hypothesis: unexpected heap allocation triggering GC in hot path
- test-engineer   → hypothesis: no latency regression test catching p999 spikes
```

### Template C — FPGA timing misalignment
```
[SYMPTOM]: Model 3 outputs arrive 3 ticks late during high-volatility periods.

Spawn agent team:
- code-reviewer   → hypothesis: PCIe DMA buffer overflow under burst load
- security-auditor → hypothesis: VHDL timing constraint violation in synthesis
- test-engineer   → hypothesis: no burst-load stress test in Model 3 test suite
```

---

## Cleanup

When consensus is reached, tell the lead:
```
Clean up the team
```

Always clean up through the lead session, not a teammate.

---

## Cost note

Three Sonnet teammates × ~15 minutes investigation costs more than `/ship`.
Use this only for production-impacting bugs where the wrong fix is expensive.
For routine PR review → use `/ship`.
