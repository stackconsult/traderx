---
description: Spec-driven task refinement workflow for complex engineering tasks
---

# Task Refinement Workflow

## Purpose
Break down complex engineering tasks into demoable slices with clear verification gates. Ensures architectural integrity and prevents context drift.

## Source
- Repository: liatrio-labs/spec-driven-workflow
- File: prompts/SDD-3-manage-tasks.md

## Workflow Steps

### 1. Task Analysis
- Identify core requirements and constraints
- Map dependencies and prerequisites
- Define acceptance criteria
- Estimate complexity and risk

### 2. Wave Planning
- Create wave-ordered task list
- Assign "Relevant Files" to each task
- Define proof artifacts for verification
- Set binary success criteria

### 3. Intent Hypotheses
Generate 3 possible approaches before implementation:
1. **Hypothesis A**: [Description]
2. **Hypothesis B**: [Description]  
3. **Hypothesis C**: [Description]

### 4. Surgical Execution
- Use ast-grep or srgn for precise transforms
- Preview changes before applying
- Maintain single responsibility per change
- Link to relevant documentation

### 5. Verification Gates
- Unit tests: Component isolation
- Integration tests: Cross-component interaction
- Regression tests: No unintended side effects
- Performance tests: Meet specified targets

### 6. Documentation Update
- Update AGENTS.md with architectural decisions
- Log reasoning trees to DECISIONS.md
- Generate proof artifacts
- Update memory bank with patterns

## Example Application

### Task: Implement HSTR Intelligence Fabric

#### Analysis
- Requires TimescaleDB with hypertables
- Bitemporal data model (snapshots + deltas)
- pgvectorscale for vector search
- O(1) + O(k) performance targets

#### Wave Plan
1. Wave 1: Database schema (bitemporal-schema.sql)
2. Wave 2: Connection layer (Python async)
3. Wave 3: Snapshot manager
4. Wave 4: Delta applier
5. Wave 5: Query optimizer

#### Intent Hypotheses
1. Use native TimescaleDB hypertables
2. Simulate with SQLite time partitions
3. Hybrid approach with fallback

#### Verification
- Schema validation: CHECK constraints
- Performance: hstr-query-bench.log
- Integration: End-to-end reconstruction

## Integration with Reflex Loop
- Discuss: Propose hypotheses
- Plan: Create wave order
- Execute: Surgical changes
- Verify: Run test suite

## Notes
- Prevents "200K token cliff"
- Maintains traceability
- Essential for complex systems
