import { Suspense } from 'react';
import { StrategyFeed } from '@/components/StrategyFeed';
import { StrategyFeedSkeleton } from '@/components/StrategyFeedSkeleton';

export const revalidate = 60; // ISR every minute

export default function StrategiesPage() {
  return (
    <div className="min-h-screen bg-background">
      <main className="container mx-auto px-4 py-6">
        <div className="mb-8">
          <h1 className="text-3xl font-bold tracking-tight">Strategy Feed</h1>
          <p className="text-muted-foreground">
            Discover AI-powered trading strategies with full transparency
          </p>
        </div>
        
        <Suspense fallback={<StrategyFeedSkeleton />}>
          <StrategyFeed />
        </Suspense>
      </main>
    </div>
  );
}
