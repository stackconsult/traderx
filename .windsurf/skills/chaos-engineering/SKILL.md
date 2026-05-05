# Chaos Engineering Skill

## When to activate
Load when: resilience testing, circuit breaker validation, fault injection,
disaster recovery drills, or before any production deployment.

## Chaos Principles for HFT Systems

1. **Blast radius first** — know the worst case before injecting fault
2. **Reversible always** — every chaos experiment must have a kill switch
3. **Prod-like env** — run chaos in staging that mirrors production
4. **Steady state first** — define what "normal" looks like before breaking it

## Experiment Library — TraderX

### Experiment 1: Ollama Service Failure
```bash
# Kill Ollama — does Genesis degrade gracefully to Gemini?
launchctl stop com.genesis.ollama
sleep 10
curl -s http://127.0.0.1:11434/api/tags | python3 -c "import sys,json; json.load(sys.stdin)" \
  && echo "FAIL: still responding" || echo "PASS: Ollama down"
# Expected: Genesis routes to google/gemini-2.0-flash-lite automatically
# Kill switch: launchctl start com.genesis.ollama
```

### Experiment 2: Channel Backpressure Saturation
```rust
#[tokio::test]
async fn chaos_channel_full() {
    let (tx, rx) = mpsc::channel::<PriceUpdate>(1);  // tiny buffer
    // Flood sender — should NOT block OMS
    for i in 0..1000 {
        let _ = tx.try_send(PriceUpdate { symbol: "AAPL".into(), price_usd: 150.0, timestamp_ns: i });
    }
    // OMS must still process orders during flood
    // Expected: drops logged, OMS unaffected
}
```

### Experiment 3: WAL Disk Full
```bash
# Simulate disk-full condition
dd if=/dev/urandom of=/tmp/diskfill bs=1M count=4096 &
FILL_PID=$!
# Run WAL writes — should fail gracefully, not panic
cargo test --package portfolio-aggregation wal_write_failure -- --nocapture
# Kill switch:
kill $FILL_PID && rm /tmp/diskfill
```

### Experiment 4: n8n Workflow Timeout
```bash
# Simulate n8n unreachable
iptables -A OUTPUT -p tcp --dport 5678 -j DROP  # or: pfctl equivalent on macOS
sleep 5
# Genesis should timeout and continue without n8n
# Kill switch: iptables -D OUTPUT -p tcp --dport 5678 -j DROP
```

### Experiment 5: Database Connection Loss
```bash
# Kill pgvector connection pool
pg_ctl stop -D /usr/local/var/postgresql@15
sleep 5
# Expected: mem0 falls back to in-memory cache
# Genesis logs error but continues operating
# Kill switch: pg_ctl start -D /usr/local/var/postgresql@15
```

## Circuit Breaker Validation Checklist

Before production deployment, validate each circuit breaker trips correctly:

- [ ] Ollama down → fallback to Gemini Flash Lite ✓/✗
- [ ] Gemini rate-limited → exponential backoff + retry ✓/✗
- [ ] pgvector unavailable → in-memory mem0 cache ✓/✗
- [ ] n8n unreachable → Genesis continues without automation ✓/✗
- [ ] TinyFish search fails → graceful degradation, no crash ✓/✗
- [ ] OMS channel full → orders dropped with log, no panic ✓/✗

## Recovery Time Objectives

| Component | Max acceptable downtime | Recovery mechanism |
|-----------|------------------------|-------------------|
| Ollama | 0s (fallback instant) | Route to Gemini |
| Gemini | 30s (backoff) | Route to Ollama |
| pgvector | 60s | In-memory cache |
| n8n | 5min | Deferred execution queue |
| WAL | 0s (must never fail) | Pre-allocated disk space |

## Game Day Template

```markdown
## Chaos Game Day — [date]
Experiment: [name]
Hypothesis: [what we expect to happen]
Steady state: [metric that defines normal]
Blast radius: [what could break]
Kill switch: [exact command to restore]

Results:
- Steady state before: [value]
- Fault injected at: [time]
- System behaviour: [what happened]
- Recovery time: [seconds]
- Steady state after: [value]
- Pass/Fail: [result]

Action items: [anything to fix]
```

Log every game day result in `GENESIS_ROADMAP.md` under `## Chaos Experiments`.
