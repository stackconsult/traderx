# Multi-Model Agent Handoff Protocols

## Description
Implements "Thin-Context" handoff between Claude 4.6 (Thinking) and Gemma 4 (Execution) agents using deterministic state machine. Manages task delegation while maintaining context across 100k+ tokens without cloud costs.

## Source
- Repository: latestaiagents/agent-handoff-protocols
- Reference: https://lobehub.com/en/skills/latestaiagents-agent-skills-agent-handoff-protocols

## Implementation Pattern

### Core Architecture
1. **HandoffPackage**: State container for task delegation
2. **Finite State Machine**: PENDING → ACCEPTED → WORKING → COMPLETE
3. **Context Compression**: TurboQuant for memory efficiency
4. **Meta-Coordinator**: Orchestrates handoff and recovery

### Key Components

#### HandoffPackage Interface
```typescript
interface HandoffPackage {
  id: string;
  status: HandoffStatus;
  sourceAgent: 'claude-4.6' | 'gemma-4';
  targetAgent: 'claude-4.6' | 'gemma-4';
  task: TaskDefinition;
  context: CompressedContext;
  plan?: DeterministicPlan;
  execution?: ExecutionResult;
  error?: HandoffError;
  timestamp: Date;
}

enum HandoffStatus {
  PENDING = 'PENDING',
  ACCEPTED = 'ACCEPTED',
  WORKING = 'WORKING',
  COMPLETE = 'COMPLETE',
  FAILED = 'FAILED',
  TIMEOUT = 'TIMEOUT'
}
```

#### Meta-Coordinator
```typescript
class MetaCoordinator {
  private claudeClient: ClaudeClient;
  private gemmaClient: GemmaClient;
  private turboQuant: TurboQuant;
  
  async initiateHandoff(task: Task): Promise<HandoffPackage> {
    // Claude 4.6 generates deterministic plan
    const plan = await this.claudeClient.generatePlan(task);
    
    // Compress context using TurboQuant
    const compressedContext = await this.turboQuant.compress({
      task,
      plan,
      history: await this.getRelevantHistory(task)
    });
    
    const handoff: HandoffPackage = {
      id: generateId(),
      status: HandoffStatus.PENDING,
      sourceAgent: 'claude-4.6',
      targetAgent: 'gemma-4',
      task,
      context: compressedContext,
      plan,
      timestamp: new Date()
    };
    
    // Validate no "Reasoning Lock-In"
    this.validateHandoff(handoff);
    
    return handoff;
  }
  
  async executeHandoff(handoff: HandoffPackage): Promise<ExecutionResult> {
    handoff.status = HandoffStatus.ACCEPTED;
    
    try {
      handoff.status = HandoffStatus.WORKING;
      
      // Gemma 4 executes with compressed context
      const result = await this.gemmaClient.execute(
        handoff.plan,
        handoff.context
      );
      
      handoff.execution = result;
      handoff.status = HandoffStatus.COMPLETE;
      
      return result;
    } catch (error) {
      handoff.error = error;
      handoff.status = HandoffStatus.FAILED;
      
      // Trigger recovery flow
      await this.handleFailure(handoff);
      throw error;
    }
  }
}
```

#### Deterministic Plan Generation
```typescript
interface DeterministicPlan {
  id: string;
  steps: PlanStep[];
  dependencies: StepDependency[];
  expectedOutcome: Outcome;
  rollbackPlan?: RollbackPlan;
}

interface PlanStep {
  id: string;
  action: string;
  parameters: Record<string, any>;
  preconditions: string[];
  postconditions: string[];
  timeout: number;
}
```

#### Context Compression with TurboQuant
```typescript
class TurboQuant {
  async compress(context: {
    task: Task;
    plan: DeterministicPlan;
    history: Message[];
  }): Promise<CompressedContext> {
    // Extract semantic embeddings
    const embeddings = await this.generateEmbeddings(context);
    
    // Apply quantization for compression
    const quantized = this.quantize(embeddings);
    
    // Store compressed representation
    return {
      embeddings: quantized,
      metadata: {
        originalSize: JSON.stringify(context).length,
        compressedSize: quantized.length,
        compressionRatio: 0.1 // 10x compression
      }
    };
  }
  
  async decompress(compressed: CompressedContext): Promise<Context> {
    // Reverse compression process
    return this.dequantize(compressed.embeddings);
  }
}
```

### Integration Points
- **Phase 2 Components**: All decisions logged in ZK-Audit
- **HSTR**: Historical context for plan generation
- **DeltaLag**: Signals as input to planning
- **PTP**: Timestamped handoff events

### Error Handling & Recovery
1. **Timeout Detection**: Auto-fail on step timeout
2. **Rollback Mechanism**: Revert partial executions
3. **Retry Logic**: Exponential backoff with jitter
4. **Post-Mortem**: Automatic JOURNAL.md entries

### Performance Optimizations
- **Context Caching**: Reuse compressed contexts
- **Parallel Execution**: Independent steps run concurrently
- **Lazy Loading**: Load context on-demand
- **Memory Management**: LRU eviction for old contexts

### Monitoring & Observability
- Handoff latency metrics
- Success/failure rates per agent
- Context compression ratios
- "Reasoning Lock-In" detection

## Success Criteria
- handoff-trace.json shows successful 3-step plan execution
- Zero "Reasoning Lock-In" errors
- <100ms handoff latency between agents
- Context compression achieves 10x ratio
