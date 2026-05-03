# Spec: Dependabot & K8s Deployment Integration

## Feature: Automated dependency scanning and K8s deployment readiness

### Input:
- `dependabot.yml` from git-workflow-skill (source template)
- `k8s/oms-engine/deployment.yaml` (existing K8s manifest)
- `orchestrate.rs` (ectoledger orchestration command)

### Output:
- `.github/dependabot.yml` in traderx-repo root
- Updated deployment.yaml with dependabot integration annotations
- Validation report showing alignment between orchestrate.rs and deployment.yaml

### Edge Cases:
- No `.github` directory exists: create it
- dependabot.yml already exists: merge/overwrite with latest template
- Deployment.yaml missing mem0 integration annotations: add them

### Success Criteria:
- [ ] `.github/dependabot.yml` created with cargo ecosystem support
- [ ] K8s deployment.yaml validated against orchestrate.rs requirements
- [ ] All files committed with conventional commit format
- [ ] CI/CD pipeline can detect dependency updates automatically
- [ ] Orchestrate.rs has proper resource limits matching K8s deployment

### Test Examples:

```yaml
# dependabot.yml validation
assert: file exists at .github/dependabot.yml
assert: contains cargo package-ecosystem
assert: contains weekly schedule interval
```

```yaml
# deployment.yaml validation
assert: contains mem0-related env vars or annotations
assert: resource limits match orchestrate.rs requirements
assert: liveness/readiness probes configured
```

---

## Validation: orchestrate.rs vs deployment.yaml

### Orchestrate.rs requirements:
- `max_steps: Option<u32>` - needs CPU/memory to support multi-step orchestration
- `reqwest::Client` with 60s timeout - needs network connectivity
- `tracing` logging - needs observability integration
- `sqlx::PgPool` - needs database connection

### Deployment.yaml alignment:
- CPU limits: 500m (sufficient for orchestration)
- Memory limits: 512Mi (sufficient for multi-agent)
- Ports: 9090 (observability), 8080 (signal-router)
- Liveness/readiness probes: configured
- Missing: mem0 integration annotations, cargo dependency scanning

### Required Changes:
1. Add `.github/dependabot.yml` with cargo + github-actions ecosystems
2. Verify deployment.yaml has proper resource limits
3. Add annotations for mem0 integration in deployment.yaml
