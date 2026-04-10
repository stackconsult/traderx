# Next.js 15 "Action Card" Frontend

## Description
Builds a TikTok-style strategy discovery UI with "Glass Box" transparency using Next.js 15 App Router and shadcn/ui. Renders agent reasoning traces and confidence signals for full auditability.

## Source
- Repository: Gentleman-Programming/gentleman-architecture-agents
- Reference: https://mintlify.com/Gentleman-Programming/gentleman-architecture-agents/agents/nextjs

## Implementation Pattern

### Core Architecture
1. **App Router**: Next.js 15 with nested layouts and route groups
2. **Server Components**: Strategy feed with Partial Prerendering (PPR)
3. **Client Components**: Interactive elements with state management
4. **Glass Box UI**: Transparent reasoning traces and confidence scores

### Key Components

#### Action Card Component
```typescript
// app/strategies/[id]/page.tsx
interface ActionCardProps {
  strategy: Strategy;
  execution: Execution;
  reasoning: ReasoningTrace;
}

export default function ActionCard({ strategy, execution, reasoning }: ActionCardProps) {
  return (
    <Card className="w-full max-w-md">
      <CardHeader>
        <div className="flex items-center justify-between">
          <Badge variant={execution.status === 'ACTIVE' ? 'default' : 'secondary'}>
            {execution.status}
          </Badge>
          <ConfidenceScore score={execution.confidence} />
        </div>
      </CardHeader>
      
      <CardContent>
        <StrategySummary strategy={strategy} />
        
        {/* Glass Box - Reasoning Trace */}
        <Collapsible>
          <CollapsibleTrigger className="flex items-center gap-2 text-sm">
            <Brain className="h-4 w-4" />
            Why did the agent do this?
          </CollapsibleTrigger>
          <CollapsibleContent>
            <ReasoningTrace trace={reasoning} />
          </CollapsibleContent>
        </Collapsible>
        
        <ActionButtons execution={execution} />
      </CardContent>
    </Card>
  );
}
```

#### Reasoning Trace Display
```typescript
// components/ReasoningTrace.tsx
export function ReasoningTrace({ trace }: { trace: ReasoningTrace }) {
  return (
    <div className="space-y-2 p-4 bg-muted rounded-lg">
      <h4 className="font-semibold">Gemma 4 Reasoning</h4>
      
      {trace.steps.map((step, index) => (
        <div key={index} className="flex gap-2">
          <Badge variant="outline">{step.type}</Badge>
          <span className="text-sm">{step.description}</span>
          <span className="text-xs text-muted-foreground">
            {step.confidence.toFixed(2)} confidence
          </span>
        </div>
      ))}
      
      <div className="mt-2 pt-2 border-t">
        <p className="text-xs text-muted-foreground">
          Final decision: {trace.decision}
        </p>
      </div>
    </div>
  );
}
```

#### Strategy Feed with PPR
```typescript
// app/strategies/page.tsx
export const revalidate = 60; // Revalidate every minute

export default async function StrategiesPage() {
  // Server component - pre-rendered with PPR
  const strategies = await getStrategies();
  
  return (
    <div className="container mx-auto py-6">
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {strategies.map((strategy) => (
          <Suspense 
            key={strategy.id} 
            fallback={<StrategyCardSkeleton />}
          >
            <StrategyCard strategy={strategy} />
          </Suspense>
        ))}
      </div>
    </div>
  );
}
```

### Integration Points
- **Phase 2 Components**: Display HSTR reconstructions, DeltaLag signals
- **ZK-Audit**: Show audit trail for each action
- **Real-time Updates**: WebSocket for live strategy updates
- **Multi-tenant**: Tenant-scoped strategy views

### UI/UX Features
1. **TikTok-style Feed**: Infinite scroll with swipe gestures
2. **Glass Box Transparency**: Full reasoning trace visibility
3. **Confidence Signals**: Visual indicators for decision confidence
4. **Interactive Elements**: Like, comment, share strategies
5. **Mobile Optimized**: PWA with offline support

### Performance Optimizations
- **Partial Prerendering**: Static shell with dynamic content
- **Image Optimization**: Next.js Image component with WebP
- **Code Splitting**: Route-based lazy loading
- **Edge Caching**: CDN for static assets

### Accessibility
- ARIA labels for all interactive elements
- Keyboard navigation support
- Screen reader compatibility
- High contrast mode support

## Success Criteria
- Sub-3 second load time on mobile throttle
- Action Card renders Gemma 4 reasoning traces
- Glass Box transparency for all executed signals
- PPR achieves <1s Time to Interactive
