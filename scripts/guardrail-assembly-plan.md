# Guardrail Assembly Plan — Genesis Hoard Task

**Objective:** Systematically implement AGENT_MASTER_SYSTEM.md guardrails into Genesis agent hoard

**Scope:** 4 build steps with defined boundaries and success criteria

---

## BUILD STEP 1: Role Guardrails (§6)

### Scope
Implement 7 agent role definitions with task_scope boundaries

### Tasks
1. Create `agent_roles.yaml` with role definitions
2. Implement task_scope boundaries for each role
3. Wire interaction rules (no override, MCA authority)
4. Implement drift prevention (3-escalation, 5-halt)
5. Test role boundary enforcement

### Success Criteria
- [ ] All 7 roles defined in YAML
- [ ] task_scope boundaries enforceable
- [ ] Interaction rules wired
- [ ] Drift prevention protocol operational

### Time Estimate: 2 hours

---

## BUILD STEP 2: Anti-Drift Guardrails (§8)

### Scope
Implement 10 drift detection signals as runtime checks

### Tasks
1. Create `drift_signals.yaml` with 10 signals
2. Implement signal detection logic
3. Wire self-correction protocol (STOP → STATE → RESET → JOURNAL)
4. Implement hallucination prevention
5. Test drift detection and correction

### Success Criteria
- [ ] All 10 signals detectable
- [ ] Self-correction protocol operational
- [ ] Hallucination prevention active
- [ ] Drift events logged to JOURNAL

### Time Estimate: 3 hours

---

## BUILD STEP 3: Rule Registry (§5)

### Scope
Wire 10 engineering rules into Genesis hoard validation

### Tasks
1. Create `rule_registry.yaml` with 10 rules
2. Implement rule-specific validation logic
3. Wire rules into pre-action checklist
4. Test rule enforcement per domain
5. Document rule violations

### Success Criteria
- [ ] All 10 rules in registry
- [ ] Rule validation operational
- [ ] Pre-action checklist enforces rules
- [ ] Domain-specific rules active

### Time Estimate: 2 hours

---

## BUILD STEP 4: Skill Registry (§4)

### Scope
Wire 9 skills into Genesis hoard routing

### Tasks
1. Create `skill_registry.yaml` with 9 skills
2. Implement skill-specific guardrails
3. Wire skills into Genesis hoard routing
4. Test skill activation per domain
5. Document skill usage

### Success Criteria
- [ ] All 9 skills in registry
- [ ] Skill-specific guardrails active
- [ ] Routing operational
- [ ] Domain-specific skills active

### Time Estimate: 2 hours

---

## EXECUTION PROTOCOL

### Phase Loop (DISCUSS → PLAN → EXECUTE → VERIFY → JOURNAL)

**DISCUSS:**
- State the build step being executed
- Identify affected files
- Identify risks

**PLAN:**
- Break into ≤50-line change steps
- Define success criteria
- Estimate time

**EXECUTE:**
- One step at a time
- Commit after each successful step
- Never batch risky changes

**VERIFY:**
- Test each build step
- Confirm success criteria met
- Cross-reference spec

**JOURNAL:**
- Write entry to JOURNAL.md
- Update MASTER_OPERATIONAL_CHECKLIST.md
- Trigger auto-journal-sync

---

## BLOCKER HANDLING

If any build step is blocked:
1. Document blocker in JOURNAL.md
2. Update TODO list
3. Escalate to next build step
4. Return after blocker resolved

---

## SUCCESS METRICS

- All 4 build steps completed
- All guardrails operational
- Genesis hoard enforces guardrails
- Zero guardrail violations in testing
- Full documentation in JOURNAL.md
