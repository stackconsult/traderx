"use client";

import React, { useCallback, useRef, useEffect, useState } from 'react';
import { useInfiniteQuery, useQueryClient } from '@tanstack/react-query';
import { motion, AnimatePresence } from 'framer-motion';
import { ActionCardComponent } from './ActionCard';
import { StrategyFeedSkeleton } from './StrategyFeedSkeleton';
import { ActionCard } from '@/types/strategy';
import { cn } from '@/lib/utils';

interface StrategyFeedProps {
  className?: string;
}

// Mock API function - replace with real API
async function fetchStrategies({ pageParam = 0 }): Promise<{
  cards: ActionCard[];
  nextCursor?: string;
  hasMore: boolean;
}> {
  // Simulate API delay
  await new Promise(resolve => setTimeout(resolve, 500));
  
  // Mock data - replace with real API call
  const mockCards: ActionCard[] = Array.from({ length: 10 }, (_, i) => ({
    id: `strategy-${pageParam}-${i}`,
    strategy: {
      id: `strategy-${pageParam}-${i}`,
      tenant_id: "tenant-1",
      name: `AI Momentum Strategy ${pageParam + i + 1}`,
      description: "Detects momentum shifts using multi-timeframe analysis and executes trades with optimal risk-reward ratios",
      status: "ACTIVE" as const,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      metadata: {}
    },
    execution: {
      id: `exec-${pageParam}-${i}`,
      strategy_id: `strategy-${pageParam}-${i}`,
      status: "COMPLETE" as const,
      confidence_score: 0.75 + Math.random() * 0.2,
      signal_strength: 0.8 + Math.random() * 0.2,
      created_at: new Date().toISOString()
    },
    reasoning: {
      id: `reasoning-${pageParam}-${i}`,
      execution_id: `exec-${pageParam}-${i}`,
      agent: "gemma-4" as const,
      steps: [
        {
          type: "ANALYSIS" as const,
          description: "Analyzed BTC/USDT 4h timeframe showing bullish divergence on RSI",
          confidence: 0.85,
          timestamp: new Date().toISOString()
        },
        {
          type: "SIGNAL" as const,
          description: "Strong buy signal detected with volume confirmation",
          confidence: 0.9,
          timestamp: new Date().toISOString()
        },
        {
          type: "RISK_CHECK" as const,
          description: "Risk/reward ratio favorable at 1:2.5",
          confidence: 0.8,
          timestamp: new Date().toISOString()
        },
        {
          type: "DECISION" as const,
          description: "Execute long position with 2% risk",
          confidence: 0.88,
          timestamp: new Date().toISOString()
        }
      ],
      final_decision: "Execute long position on BTC/USDT with 2% risk allocation",
      total_confidence: 0.88,
      created_at: new Date().toISOString()
    },
    market_data: {
      symbol: "BTCUSDT",
      price: 45000 + Math.random() * 5000,
      change_24h: (Math.random() - 0.5) * 10,
      volume: 1000000000 + Math.random() * 500000000
    },
    social_metrics: {
      views: Math.floor(Math.random() * 1000),
      likes: Math.floor(Math.random() * 100),
      comments: Math.floor(Math.random() * 50),
      shares: Math.floor(Math.random() * 20)
    }
  }));
  
  return {
    cards: mockCards,
    hasMore: pageParam < 10, // Limit to 10 pages for demo
    nextCursor: pageParam < 10 ? String(pageParam + 1) : undefined
  };
}

export function StrategyFeed({ className }: StrategyFeedProps) {
  const queryClient = useQueryClient();
  
  const {
    data,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
    status,
    refetch
  } = useInfiniteQuery({
    queryKey: ['strategies'],
    queryFn: fetchStrategies,
    initialPageParam: 0,
    getNextPageParam: (lastPage) => lastPage.nextCursor,
    refetchOnWindowFocus: false,
    staleTime: 30000 // 30 seconds
  });
  
  // Intersection Observer for infinite scroll
  const observer = useRef<IntersectionObserver>();
  const lastCardRef = useCallback((node: HTMLDivElement | null) => {
    if (isFetchingNextPage) return;
    if (observer.current) observer.current.disconnect();
    
    observer.current = new IntersectionObserver(entries => {
      if (entries[0].isIntersecting && hasNextPage) {
        fetchNextPage();
      }
    }, {
      threshold: 0.1,
      rootMargin: '100px' // Start loading 100px before visible
    });
    
    if (node) observer.current.observe(node);
  }, [isFetchingNextPage, fetchNextPage, hasNextPage]);
  
  // Pull-to-refresh functionality
  const pullToRefreshRef = useRef<HTMLDivElement>(null);
  const [isPulling, setIsPulling] = useState(false);
  const [pullDistance, setPullDistance] = useState(0);
  const startY = useRef(0);
  
  useEffect(() => {
    const handleTouchStart = (e: TouchEvent) => {
      if (window.scrollY === 0) {
        startY.current = e.touches[0].clientY;
        setIsPulling(true);
      }
    };
    
    const handleTouchMove = (e: TouchEvent) => {
      if (isPulling) {
        const currentY = e.touches[0].clientY;
        const distance = currentY - startY.current;
        setPullDistance(Math.min(distance, 120));
      }
    };
    
    const handleTouchEnd = () => {
      if (isPulling && pullDistance > 80) {
        refetch();
      }
      setIsPulling(false);
      setPullDistance(0);
    };
    
    window.addEventListener('touchstart', handleTouchStart);
    window.addEventListener('touchmove', handleTouchMove);
    window.addEventListener('touchend', handleTouchEnd);
    
    return () => {
      window.removeEventListener('touchstart', handleTouchStart);
      window.removeEventListener('touchmove', handleTouchMove);
      window.removeEventListener('touchend', handleTouchEnd);
    };
  }, [isPulling, pullDistance, refetch]);
  
  const handleLike = useCallback((cardId: string) => {
    // Optimistic update
    queryClient.setQueryData(['strategies'], (old: any) => {
      if (!old) return old;
      
      const newPages = old.pages.map((page: any) => ({
        ...page,
        cards: page.cards.map((card: ActionCard) =>
          card.id === cardId
            ? {
                ...card,
                social_metrics: {
                  ...card.social_metrics,
                  likes: (card.social_metrics?.likes || 0) + 1
                }
              }
            : card
        )
      }));
      
      return { ...old, pages: newPages };
    });
    
    // API call to like
    fetch('/api/strategies/like', {
      method: 'POST',
      body: JSON.stringify({ cardId })
    }).catch(console.error);
  }, [queryClient]);
  
  const handleComment = useCallback((cardId: string) => {
    // Open comment modal or navigate
    console.log('Comment on:', cardId);
  }, []);
  
  const handleShare = useCallback((cardId: string) => {
    // Share functionality
    if (navigator.share) {
      navigator.share({
        title: 'AI Trading Strategy',
        text: 'Check out this AI-powered trading strategy',
        url: `${window.location.origin}/strategies/${cardId}`
      });
    } else {
      navigator.clipboard.writeText(`${window.location.origin}/strategies/${cardId}`);
    }
  }, []);
  
  if (status === 'pending') {
    return <StrategyFeedSkeleton />;
  }
  
  if (status === 'error') {
    return (
      <div className="flex flex-col items-center justify-center py-20">
        <p className="text-muted-foreground mb-4">Failed to load strategies</p>
        <button
          onClick={() => refetch()}
          className="px-4 py-2 bg-primary text-primary-foreground rounded-md"
        >
          Retry
        </button>
      </div>
    );
  }
  
  const allCards = data?.pages.flatMap(page => page.cards) || [];
  
  return (
    <div className={cn("space-y-6", className)}>
      {/* Pull-to-refresh indicator */}
      <div
        ref={pullToRefreshRef}
        className="fixed top-0 left-0 right-0 z-50 flex justify-center pointer-events-none"
        style={{ transform: `translateY(${pullDistance - 120}px)` }}
      >
        <div className={cn(
          "bg-background border rounded-full p-2 shadow-lg transition-opacity",
          pullDistance > 80 ? "opacity-100" : "opacity-0"
        )}>
          <div className="w-6 h-6 border-2 border-primary border-t-transparent rounded-full animate-spin" />
        </div>
      </div>
      
      {/* Strategy Cards */}
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        <AnimatePresence>
          {allCards.map((card, index) => (
            <motion.div
              key={card.id}
              ref={index === allCards.length - 3 ? lastCardRef : undefined}
              layout
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -20 }}
              transition={{ duration: 0.3, delay: index * 0.05 }}
            >
              <ActionCardComponent
                card={card}
                onLike={handleLike}
                onComment={handleComment}
                onShare={handleShare}
              />
            </motion.div>
          ))}
        </AnimatePresence>
      </div>
      
      {/* Loading indicator */}
      {isFetchingNextPage && (
        <div className="flex justify-center py-8">
          <div className="w-8 h-8 border-2 border-primary border-t-transparent rounded-full animate-spin" />
        </div>
      )}
      
      {/* End of feed */}
      {!hasNextPage && allCards.length > 0 && (
        <div className="text-center py-8 text-muted-foreground">
          <p>You've reached the end of the feed</p>
        </div>
      )}
    </div>
  );
}
