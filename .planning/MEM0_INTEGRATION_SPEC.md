# Mem0 Integration Specification

## Objective
Establish mem0 as the memory layer for all learnship skills and workflows, enabling guided knowledge building across agent runs.

## Requirements

### 1. Mem0 CLI Installation
- Install mem0 CLI locally via pip (completed)
- Configure mem0 for local storage (SQLite/Vector DB)
- Set up user_id and agent_id for memory scoping

### 2. MCP Server Configuration
- Create mem0 MCP server configuration
- Configure memory persistence to local storage
- Set up memory retrieval endpoints for agent access

### 3. Learnship Skills Integration
For each skill in `packages/learnship/skills/*/SKILL.md`:
- Add mem0 memory hook at skill invocation
- Store skill execution context (inputs, outputs, decisions)
- Retrieve relevant past experiences before skill execution
- Update memory after skill completion

### 4. Learnship Workflows Integration
For each workflow in `.windsurf/workflows/*.md`:
- Add mem0 memory checkpoints at workflow phases
- Store workflow state transitions
- Retrieve workflow context from memory on resume
- Embed learning from previous workflow runs

### 5. Engineering Orchestra Integration
Update `packages/oms-engine/src/engineering_orchestra.rs`:
- Add mem0 memory layer to agent responses
- Store agent Q&A interactions with context
- Retrieve relevant past Q&A for similar questions
- Build knowledge base from agent interactions

## Memory Schema

### Skill Execution Memory
```json
{
  "type": "skill_execution",
  "skill_name": "debug-team",
  "action": "triage",
  "inputs": {...},
  "outputs": {...},
  "context": {
    "timestamp": "2026-05-02T19:00:00Z",
    "session_id": "uuid",
    "user_id": "traderx"
  },
  "lessons_learned": ["..."],
  "tags": ["debug", "triage", "bug"]
}
```

### Workflow Memory
```json
{
  "type": "workflow_phase",
  "workflow_name": "spec-driven-workflow",
  "phase": "planning",
  "state": {...},
  "decisions": [...],
  "context": {
    "timestamp": "2026-05-02T19:00:00Z",
    "session_id": "uuid"
  },
  "lessons_learned": ["..."]
}
```

### Agent Q&A Memory
```json
{
  "type": "agent_qa",
  "agent_role": "SystemsArchitect",
  "question": "...",
  "response": "...",
  "metadata": {
    "category": "Architecture",
    "confidence": 0.95,
    "timestamp": "2026-05-02T19:00:00Z"
  },
  "lessons_learned": ["..."]
}
```

## Integration Points

### Before Skill Execution
```python
# Retrieve relevant memories
memories = mem0.search(
    query=skill_context,
    user_id="traderx",
    agent_id=skill_name,
    limit=5
)
# Apply lessons from memory to current execution
```

### After Skill Execution
```python
# Store execution context
mem0.add(
    content=skill_execution_record,
    user_id="traderx",
    agent_id=skill_name,
    metadata=execution_metadata
)
```

### Workflow Checkpoints
```python
# Store workflow state at phase transitions
mem0.add(
    content=workflow_state,
    user_id="traderx",
    agent_id=workflow_name,
    metadata={"phase": current_phase}
)
```

## Success Criteria

- [ ] Mem0 CLI installed and configured locally
- [ ] MCP server configured for mem0
- [ ] All skills have mem0 hooks integrated
- [ ] All workflows have mem0 checkpoints
- [ ] Engineering orchestra uses mem0 for knowledge building
- [ ] Memory retrieval improves agent performance (measured)
- [ ] Memory persistence verified across sessions

## Implementation Order

1. Configure mem0 for local storage
2. Create mem0 MCP server config
3. Add mem0 hooks to one skill as proof of concept
4. Extend to all skills
5. Add mem0 checkpoints to one workflow
6. Extend to all workflows
7. Integrate mem0 into engineering_orchestra.rs
8. Test memory retrieval and storage
9. Commit integration to GitHub
