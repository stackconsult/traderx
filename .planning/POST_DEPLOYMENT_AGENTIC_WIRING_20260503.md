# Post-Deployment Agentic Wiring Analysis
**Generated:** 2026-05-03
**Purpose:** Engineer full scope AI agent LLM workflows, handoffs, modelling, and controls for post-deployment agentic capabilities

## Executive Summary

Post-deployment agentic capabilities require a separate control plane from build-time skills. While skills provide workflow logic during development and deployment, **production agents require authentication, governance, state management, and runtime controls** that operate independently of the skill system.

**Key Finding:** Agentic capabilities in production require **four distinct layers**:
1. **Authentication Layer** - Identity verification, token management, tenant isolation
2. **Governance Layer** - Policy enforcement, access control, approval gates
3. **Execution Layer** - State management, checkpoint-resume, cost controls
4. **Observability Layer** - Audit trails, monitoring, incident response

## Research Findings Summary

### Agent Handoff Patterns

**Two Primary Patterns:**

1. **Handoffs** - Specialist takes ownership of the conversation
   - Use when: Specialist should own the next response
   - Control moves to the specialist agent
   - Full conversation context preserved

2. **Agents as Tools** - Manager stays in control, calls specialists as bounded capabilities
   - Use when: Manager should synthesize the final answer
   - Manager keeps ownership of the reply
   - Specialist performs bounded task

**Four Handoff Mechanisms (Priority Order):**
1. **Context-based** - Deterministic routing based on shared state (no LLM needed)
2. **LLM-based** - Agent's LLM evaluates when transition should occur
3. **Tool-based** - Tools explicitly return the next agent as part of result
4. **After-work** - Fallback when nothing else triggers

### Authentication Architecture

**Three Token Types:**

1. **User Token** - Human user authentication (OAuth 2.0)
   - Scopes: User-level permissions
   - TTL: Longer (hours to days)
   - Audience: Application

2. **Agent Token** - Agent service account authentication
   - Scopes: Agent-specific capabilities
   - TTL: Short (60-300 seconds)
   - Audience: Tool servers
   - Tenant-scoped: Each agent identity scoped to single tenant

3. **Capability Token** - Per-tool capability token
   - Scopes: Narrowest scope for specific tool
   - TTL: Shortest (60 seconds)
   - Audience: Single tool server
   - Rule: Only narrow, never widen

**Token Flow Rules:**
- Scopes flow strictly downward (never widen)
- Tenant ID encoded in every token, log line, policy evaluation
- Cross-tenant isolation is foundational invariant

### Agent Gateway Pattern

**Gateway Responsibilities:**

1. **Policy Authorization** - Open Policy Agent (OPA) evaluation
   - Agent declares intent (resource, action, context)
   - Policy permits or denies
   - Policy lives outside agent code

2. **Full Observability** - OpenTelemetry integration
   - Trace span for every tool call
   - Full structured data (request, decision, result, timing)
   - Parent span linking (agent session, task ID)

3. **Ephemeral Execution** - Isolated container execution
   - Tool call executes in short-lived container
   - Container gets only credentials for specific call
   - Blast radius bounded to execution window

**Gateway Decision Flow:**
```
Agent Request → Gateway → OPA Evaluation → Policy Decision
→ Spawn Ephemeral Container → Execute Tool → Return Result
→ Emit Trace Span → Terminate Container
```

### State Management

**Five-Layer Operations Stack:**

1. **Execution Bus** - NATS or Kafka with explicit routing
   - Backpressure visibility
   - Retry management
   - Job queuing

2. **State Store** - Redis or Postgres for job state
   - Job state persistence
   - Context pointers
   - Result pointers
   - Reproducible run history

3. **Scheduler/Orchestrator** - Deterministic behavior
   - Retries with backoff
   - Timeouts
   - Dead-letter handling
   - No orphaned runs

4. **Policy Decision Point** - Pre-dispatch checks
   - Policy checks before submit
   - Policy checks before dispatch
   - Approval queue for risky actions
   - Immutable policy snapshot binding

5. **Output Safety** - Result pipeline
   - Allow, redact, or quarantine decisions
   - PII/secrets leak prevention
   - Output validation

**Checkpoint-Resume Pattern:**
- Serialize agent state after each successful tool call
- Store durably
- On failure, resume from last checkpoint
- Reduces token spend by 40% during degraded conditions

### Governance Controls

**Six Governance Layers:**

1. **Access Control** - RBAC allowlist/denylist
   - Tool access control
   - Default deny policy
   - Checked before every tool call

2. **Execution Control** - Run execution flow limits
   - Max steps (stops loops)
   - Rate limiting
   - Tool call limits

3. **Cost Control** - Spending limits
   - Max tokens per task
   - Max tool calls per task
   - Max USD per task/hour/day
   - 80% budget threshold alert

4. **Human Control** - Intervention mechanisms
   - Human approval gates
   - Kill switch (emergency stop)
   - Escalation triggers

5. **Observability** - Event visibility
   - Audit logs
   - Distributed traces
   - Metrics
   - Alerts (Slack/PagerDuty)

6. **Lifecycle Control** - Agent updates
   - Versioning
   - Rollback capability
   - Safe releases

**Policy Enforcement Point (PEP):**
- Intercepts every tool call before execution
- Returns: allow, deny, or approval_required
- Separate system level, not part of prompt/model logic
- Implemented as middleware/tool gateway

### Deployment Strategies

**Four Deployment Patterns:**

1. **Blue-Green Deployment**
   - Two identical production environments
   - Deploy to inactive environment
   - Run validation tests
   - Switch traffic
   - Keep old as instant rollback

2. **Canary Deployment**
   - Gradual rollout: 5% → 25% → 50% → 100%
   - Monitor error rates, latency, satisfaction
   - Automatic rollback on metrics degradation

3. **Feature Flag Deployment**
   - Deploy code with features disabled
   - Enable gradually per user segment
   - A/B test different behaviors
   - Instant disable if issues arise

4. **Phased Rollout**
   - Phase 0: Synthetic replay (3-7 days)
   - Phase 1: 5% low-risk traffic (3-5 days)
   - Phase 2: 25% mixed workload (5-7 days)
   - Phase 3: 50-100% (7-14 days)

## Post-Deployment Agentic Architecture

### Complete System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     User Application                        │
│                  (Frontend / API Gateway)                   │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ User Request (with User JWT)
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  Agent Gateway / Control Plane              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ JWT Auth     │  │ Policy Engine│  │ Rate Limiter │      │
│  │ (Validation) │  │ (OPA/Rego)   │  │ (Per Tenant) │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ Agent Request (with Agent Token)
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                    Agent Runtime / Orchestrator              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ State Manager│  │ Scheduler    │  │ Handoff Logic│      │
│  │ (Redis/PG)   │  │ (Deterministic│  │ (Context/LLM/ │      │
│  │              │  │  Retries)    │  │  Tool/After) │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ Tool Call (with Capability Token)
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                    Tool Gateway / Policy Layer                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ PEP (Policy  │  │ Approval     │  │ Output Safety│      │
│  │  Enforcement)│  │  Queue       │  │  Pipeline    │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ Allowed Tool Call
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Ephemeral Execution Environment                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Isolated     │  │ Scoped       │  │ Checkpoint-  │      │
│  │ Container    │  │ Credentials  │  │  Resume      │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ Tool Result
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   Backend Systems / APIs                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Database     │  │ External API │  │ Internal     │      │
│  │              │  │              │  │ Services     │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
                         │
                         │ Audit Trail
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Observability Platform (OpenTelemetry)           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Distributed  │  │ Audit Logs   │  │ Metrics &    │      │
│  │ Tracing      │  │ (Immutable)  │  │  Alerts      │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

## Wiring Strategy for TraderX

### Phase 1: Authentication Layer

**Implementation:**

1. **JWT Authentication at Agent Gateway**
   ```yaml
   agent_gateway:
     jwt_auth:
       mode: Strict
       providers:
         - issuer: traderx-auth
           jwks: ${JWKS_ENDPOINT}
       authorization:
         action: Allow
         policy:
           matchExpressions:
             - 'jwt.sub in data.authorized_users'
             - 'jwt.tenant == input.tenant_id'
   ```

2. **Token Exchange Pattern (OBO)**
   - User JWT + Agent Identity → On-Behalf-Of Token
   - Agent runtime validates OBO token
   - Includes both user and agent claims

3. **Tenant Isolation**
   - Tenant ID in every token claim
   - Policy enforces tenant-aware authorization
   - Tenant-specific rate limits and quotas

### Phase 2: Governance Layer

**OPA Policy Example:**

```rego
# Allow read-only operations in staging namespace
allow if {
    input.action in {"get", "list", "watch"}
    input.namespace == "staging"
    input.agent_id in data.authorized_agents
}

# Allow scale operations, but cap max replicas
allow if {
    input.action == "scale"
    input.resource == "deployment"
    input.desired_replicas <= 10
    input.namespace != "production"
    input.agent_id in data.authorized_agents
}

# Require approval for write operations in production
approval_required if {
    input.action in {"create", "update", "delete"}
    input.namespace == "production"
    input.agent_id in data.authorized_agents
}
```

**Governance Controls:**

```yaml
governance:
  access_control:
    mode: default_deny
    allowlist:
      - risk_bus_check
      - portfolio_read
      - order_submit
    blocklist:
      - database_delete
      - config_modify
  
  execution_control:
    max_steps: 100
    rate_limit:
      per_agent: 10/minute
      per_tenant: 100/minute
    tool_call_limit: 50
  
  cost_control:
    max_tokens_per_task: 100000
    max_usd_per_hour: 10.0
    max_usd_per_day: 100.0
    budget_alert_threshold: 0.8
  
  human_control:
    approval_required:
      - write_operations
      - financial_changes
      - data_deletion
    kill_switch:
      enabled: true
      check_interval: 5s
```

### Phase 3: Execution Layer

**State Management:**

```yaml
state_management:
  store: redis
  checkpoint_interval: after_each_tool_call
  resume_strategy: from_last_checkpoint
  
  session_store:
    backend: redis
    ttl: 3600
    key_pattern: "session:{agent_id}:{session_id}"
  
  job_state:
    backend: postgres
    fields:
      - job_id
      - agent_id
      - tenant_id
      - status
      - current_step
      - checkpoint_data
      - created_at
      - updated_at
```

**Scheduler Configuration:**

```yaml
scheduler:
  execution_bus: nats
  subject_routing: explicit
  
  retry_policy:
    max_retries: 3
    backoff: exponential
    initial_delay: 1s
    max_delay: 30s
  
  timeout_policy:
    step_timeout: 30s
    task_timeout: 300s
    dead_letter_ttl: 86400
```

### Phase 4: Observability Layer

**OpenTelemetry Integration:**

```yaml
observability:
  tracing:
    enabled: true
    exporter: otlp
    sampling: 1.0
    propagation: w3c
  
  metrics:
    enabled: true
    exporter: prometheus
    interval: 10s
  
  audit_logging:
    backend: postgres
    immutable: true
    retention: 7years
    fields:
      - trace_id
      - span_id
      - agent_id
      - tenant_id
      - user_id
      - action
      - resource
      - decision
      - approver
      - timestamp
```

**Alerting:**

```yaml
alerts:
  error_rate:
    threshold: 0.05
    window: 5m
    action: pagerduty
  
  latency:
    threshold: 10s
    percentile: 95
    window: 5m
    action: slack
  
  cost_spike:
    threshold: 2.0
    baseline: hourly
    window: 1h
    action: email
  
  success_rate_drop:
    threshold: 0.1
    baseline: daily
    window: 1h
    action: slack
```

## Agent Handoff Implementation for TraderX

### Multi-Agent Architecture

**Agent Hierarchy:**

```
┌─────────────────────────────────────────────────────────────┐
│                   Orchestrator Agent                          │
│              (Routes to specialists based on task)            │
└──────┬──────────────┬──────────────┬──────────────┬─────────┘
       │              │              │              │
       ▼              ▼              ▼              ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ Risk Agent   │ │ Portfolio    │ │ Execution    │ │ Analysis    │
│              │ │ Agent        │ │ Agent        │ │ Agent        │
└──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘
```

**Handoff Patterns:**

1. **Context-Based Handoffs** (Fast, Deterministic)
   ```yaml
   handoffs:
     context_based:
       - condition: request.risk_level == "high"
         target: risk_agent
       - condition: request.type == "portfolio_query"
         target: portfolio_agent
       - condition: request.type == "order"
         target: execution_agent
   ```

2. **LLM-Based Handoffs** (Semantic Understanding)
   ```yaml
   handoffs:
     llm_based:
       - condition: customer_needs_technical_support
         target: technical_support_agent
       - condition: customer_needs_billing_help
         target: billing_agent
   ```

3. **Tool-Based Handoffs** (Logic-Dependent Routing)
   ```yaml
   handoffs:
     tool_based:
       - tool: check_account_tier
         result:
           vip: true
           target: vip_specialist
           standard: true
           target: standard_specialist
   ```

4. **After-Work Fallback** (Safety Net)
   ```yaml
   handoffs:
     after_work:
       - fallback: orchestrator_agent
         reason: "No handoff condition matched"
   ```

## Deployment Checklist

### Pre-Deployment

- [ ] Test agent in staging with production-like data
- [ ] Verify API rate limits and cost controls
- [ ] Confirm tool permissions are production-appropriate
- [ ] Review agent decision logs for unexpected behavior
- [ ] Set up monitoring alerts (error rates, latency, cost)
- [ ] Document rollback procedure
- [ ] Test rollback procedure
- [ ] Verify tenant isolation
- [ ] Test kill switch
- [ ] Validate audit logging

### During Deployment

- [ ] Deploy during low-traffic window
- [ ] Monitor agent decisions in real-time
- [ ] Track error rates vs. baseline
- [ ] Verify integrations (databases, APIs, tools)
- [ ] Check cost metrics (LLM API usage)
- [ ] Monitor approval queue depth
- [ ] Verify checkpoint-resume functionality

### Post-Deployment

- [ ] Monitor for 24-48 hours
- [ ] Review agent decision quality
- [ ] Analyze user feedback
- [ ] Document any unexpected behaviors
- [ ] Update runbooks with learnings
- [ ] Conduct governance review
- [ ] Update baselines

## Security Considerations

### Threat Model

| Threat | Control |
|--------|---------|
| Prompt injection via tool output | Sanitize tool outputs; treat as untrusted data |
| Tool abuse / scope escalation | Allow-list tools per agent role; enforce at tool server |
| Supply chain compromise | Sign dependencies; maintain SBOMs; use runtime isolation |
| SSRF via agent-initiated HTTP calls | Egress allow-list on agent runtime network |
| Data exfiltration | Content filters on outbound tool calls; canary data |
| Token replay | DPoP or MTLS on all agent-to-tool calls |
| Cross-tenant data access | Tenant ID in every token claim; policy enforcement |

### Security Controls

1. **Sender-Constrained Tokens**
   - DPoP (Demonstrating Proof-of-Possession)
   - mTLS (Mutual TLS)
   - Prevent token replay

2. **Egress Network Controls**
   - Allow-list external endpoints
   - Block RFC 1918 ranges (private IPs)
   - Rate limit per tenant

3. **Credential Management**
   - Workload identity or External Secrets
   - No static credentials in container specs
   - Per-tool service accounts with minimal permissions

4. **Supply Chain Security**
   - Sign all dependencies
   - Maintain SBOMs (Software Bill of Materials)
   - Runtime isolation (containers/VMs)

## Cost Engineering

### Cost Controls

```yaml
cost_controls:
  per_task:
    max_tokens: 100000
    max_tool_calls: 50
    max_usd: 10.0
  
  per_hour:
    max_tokens: 1000000
    max_usd: 100.0
  
  per_day:
    max_tokens: 10000000
    max_usd: 1000.0
  
  alerts:
    budget_threshold: 0.8
    anomaly_detection: true
```

### Semantic Caching

- Store and reuse responses to identical queries
- Reduces redundant API calls
- Implement with Redis or similar
- Cache key based on prompt hash + tool context

## Incident Response

### Escalation Triggers

1. **Confidence Below Threshold**
   - Agent stops and escalates to human
   - Provides context summary
   - Binds blast radius

2. **Consecutive Tool Call Failures**
   - After N failures, escalate
   - Include error context
   - Request human intervention

3. **Max Step Count Reached**
   - Agent halts execution
   - Summarizes progress
   - Escalates for decision

### Rollback Procedures

1. **Kill Switch Activation**
   - Global flag in Redis
   - Policy layer checks before every step
   - Stops execution immediately

2. **Version Rollback**
   - Revert to previous agent version
   - Use blue-green deployment pattern
   - Instant traffic switch

3. **Policy Rollback**
   - Revert to previous policy version
   - Git-based policy management
   - Instant deployment

## Next Steps

### Implementation Priority

**Phase 1 (Before First Production Task):**
1. Implement JWT authentication at Agent Gateway
2. Set up tenant isolation
3. Configure default-deny allowlist
4. Implement max_steps and budgets
5. Set up append-only audit logging
6. Implement kill switch

**Phase 2 (Week 2-4):**
1. Implement policy enforcement layer
2. Add deny rules for high-impact actions
3. Implement approval queue for top 3 triggers
4. Set up OpenTelemetry tracing
5. Configure checkpoint-resume
6. Test rollback procedures

**Phase 3 (Month 2):**
1. Implement behavioral evaluation pipeline
2. Add shadow mode infrastructure
3. Configure cost anomaly alerting
4. Establish quarterly governance review
5. Implement semantic caching
6. Add egress network controls

### Skills Integration

**Skills Required for Post-Deployment:**

1. **agent-handoff** - Already exists, needs configuration
2. **adaptive-self-healing** - For automatic recovery
3. **production-guard** - For validation layers
4. **autonomous-audit-loop** - For continuous monitoring
5. **debug-team** - For incident investigation
6. **qa-team** - For quality assurance

**Skills to Create:**

1. **agent-gateway-config** - Gateway configuration management
2. **agent-policy-management** - OPA policy creation and updates
3. **agent-incident-response** - Automated incident handling
4. **agent-cost-optimization** - Cost monitoring and optimization
5. **agent-security-audit** - Security compliance checking

## Conclusion

Post-deployment agentic capabilities require a **separate control plane** from build-time skills. The architecture consists of **four layers**: authentication, governance, execution, and observability. Each layer must be implemented with production-grade controls including JWT authentication, OPA policy enforcement, state management with checkpoint-resume, and comprehensive observability with OpenTelemetry.

The implementation should follow a **phased approach**, starting with critical controls (authentication, governance, audit logging) before adding advanced features (behavioral evaluation, shadow mode, cost optimization). This ensures a solid foundation before building more sophisticated capabilities.
