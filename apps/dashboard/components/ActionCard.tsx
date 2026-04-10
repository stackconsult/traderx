"use client";

import React, { useState, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  Brain, 
  TrendingUp, 
  TrendingDown, 
  Eye, 
  Heart, 
  MessageCircle, 
  Share2,
  ChevronDown,
  ChevronUp,
  Activity,
  Clock,
  CheckCircle,
  XCircle,
  AlertCircle
} from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { Progress } from '@/components/ui/progress';
import { Separator } from '@/components/ui/separator';
import { cn } from '@/lib/utils';
import { ActionCard, ConfidenceScore, RationaleSummary } from '@/types/strategy';

interface ActionCardProps {
  card: ActionCard;
  onLike?: (cardId: string) => void;
  onComment?: (cardId: string) => void;
  onShare?: (cardId: string) => void;
  className?: string;
}

export function ActionCardComponent({ 
  card, 
  onLike, 
  onComment, 
  onShare, 
  className 
}: ActionCardProps) {
  const [isReasoningOpen, setIsReasoningOpen] = useState(false);
  const [isLiked, setIsLiked] = useState(false);

  // Calculate confidence score display
  const confidenceScore = useMemo<ConfidenceScore>(() => {
    const score = card.execution.confidence_score;
    if (score >= 0.8) {
      return { score, label: 'Very High', color: 'text-green-600' };
    } else if (score >= 0.6) {
      return { score, label: 'High', color: 'text-blue-600' };
    } else if (score >= 0.4) {
      return { score, label: 'Medium', color: 'text-yellow-600' };
    } else {
      return { score, label: 'Low', color: 'text-red-600' };
    }
  }, [card.execution.confidence_score]);

  // Generate rationale summary
  const rationaleSummary = useMemo<RationaleSummary>(() => {
    const decisionStep = card.reasoning.steps.find(s => s.type === 'DECISION');
    const signalStep = card.reasoning.steps.find(s => s.type === 'SIGNAL');
    
    return {
      title: decisionStep?.description || 'AI Trading Decision',
      description: signalStep?.description || 'Signal detected and analyzed',
      key_factors: card.reasoning.steps
        .filter(s => s.confidence > 0.7)
        .map(s => s.description)
        .slice(0, 3)
    };
  }, [card.reasoning]);

  // Status icon and color
  const statusInfo = useMemo(() => {
    switch (card.execution.status) {
      case 'COMPLETE':
        return { icon: CheckCircle, color: 'text-green-600', label: 'Completed' };
      case 'WORKING':
        return { icon: Activity, color: 'text-blue-600', label: 'Executing' };
      case 'FAILED':
        return { icon: XCircle, color: 'text-red-600', label: 'Failed' };
      case 'PENDING':
        return { icon: Clock, color: 'text-gray-600', label: 'Pending' };
      default:
        return { icon: AlertCircle, color: 'text-yellow-600', label: 'Unknown' };
    }
  }, [card.execution.status]);

  const StatusIcon = statusInfo.icon;

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      transition={{ duration: 0.3 }}
      className={cn("w-full max-w-md mx-auto", className)}
    >
      <Card className="overflow-hidden border-0 shadow-lg hover:shadow-xl transition-shadow duration-300">
        {/* Header with status and confidence */}
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <Badge 
              variant="outline" 
              className={cn("font-semibold", statusInfo.color)}
            >
              <StatusIcon className="w-3 h-3 mr-1" />
              {statusInfo.label}
            </Badge>
            
            <div className="flex items-center gap-2">
              <span className="text-sm text-muted-foreground">Confidence</span>
              <div className="flex items-center gap-1">
                <span className={cn("font-bold", confidenceScore.color)}>
                  {(confidenceScore.score * 100).toFixed(0)}%
                </span>
                <Badge variant="secondary" className="text-xs">
                  {confidenceScore.label}
                </Badge>
              </div>
            </div>
          </div>
          
          {/* Strategy name and description */}
          <div>
            <h3 className="font-semibold text-lg leading-tight">
              {card.strategy.name}
            </h3>
            <p className="text-sm text-muted-foreground mt-1 line-clamp-2">
              {card.strategy.description}
            </p>
          </div>
          
          {/* Market data if available */}
          {card.market_data && (
            <div className="flex items-center justify-between mt-3 p-2 bg-muted/50 rounded-lg">
              <div className="flex items-center gap-2">
                <span className="font-mono text-sm">{card.market_data.symbol}</span>
                <span className="font-semibold">${card.market_data.price.toFixed(2)}</span>
              </div>
              <div className={cn(
                "flex items-center gap-1 text-sm",
                card.market_data.change_24h >= 0 ? "text-green-600" : "text-red-600"
              )}>
                {card.market_data.change_24h >= 0 ? (
                  <TrendingUp className="w-3 h-3" />
                ) : (
                  <TrendingDown className="w-3 h-3" />
                )}
                {card.market_data.change_24h >= 0 ? '+' : ''}
                {card.market_data.change_24h.toFixed(2)}%
              </div>
            </div>
          )}
        </CardHeader>

        <CardContent className="pt-0">
          {/* Rationale Summary */}
          <div className="space-y-2">
            <h4 className="font-medium text-sm">{rationaleSummary.title}</h4>
            <p className="text-sm text-muted-foreground">
              {rationaleSummary.description}
            </p>
            
            {/* Key Factors */}
            <div className="space-y-1">
              <span className="text-xs text-muted-foreground">Key Factors:</span>
              <ul className="text-xs space-y-1">
                {rationaleSummary.key_factors.map((factor, index) => (
                  <li key={index} className="flex items-start gap-1">
                    <span className="w-1 h-1 bg-primary rounded-full mt-1.5 flex-shrink-0" />
                    <span className="line-clamp-2">{factor}</span>
                  </li>
                ))}
              </ul>
            </div>
          </div>

          <Separator className="my-4" />

          {/* Glass Box - Reasoning Trace */}
          <Collapsible 
            open={isReasoningOpen} 
            onOpenChange={setIsReasoningOpen}
          >
            <CollapsibleTrigger asChild>
              <Button 
                variant="ghost" 
                size="sm" 
                className="w-full justify-between h-auto p-3"
              >
                <div className="flex items-center gap-2">
                  <Brain className="w-4 h-4" />
                  <span className="text-sm font-medium">
                    Why did the agent do this?
                  </span>
                  <Badge variant="outline" className="text-xs">
                    {card.reasoning.agent}
                  </Badge>
                </div>
                {isReasoningOpen ? (
                  <ChevronUp className="w-4 h-4" />
                ) : (
                  <ChevronDown className="w-4 h-4" />
                )}
              </Button>
            </CollapsibleTrigger>
            
            <CollapsibleContent className="space-y-3">
              <div className="p-3 bg-muted/30 rounded-lg border">
                <div className="space-y-3">
                  {/* Reasoning Steps */}
                  {card.reasoning.steps.map((step, index) => (
                    <div key={index} className="flex gap-3">
                      <Badge 
                        variant="secondary" 
                        className="text-xs h-6 flex-shrink-0"
                      >
                        {step.type}
                      </Badge>
                      <div className="flex-1 space-y-1">
                        <p className="text-sm">{step.description}</p>
                        <div className="flex items-center gap-2">
                          <Progress 
                            value={step.confidence * 100} 
                            className="flex-1 h-2" 
                          />
                          <span className="text-xs text-muted-foreground w-12 text-right">
                            {(step.confidence * 100).toFixed(0)}%
                          </span>
                        </div>
                      </div>
                    </div>
                  ))}
                  
                  {/* Final Decision */}
                  <Separator />
                  <div className="pt-2">
                    <div className="flex items-center gap-2 mb-2">
                      <span className="text-sm font-semibold">Final Decision:</span>
                      <Badge variant="default" className="text-xs">
                        {(card.reasoning.total_confidence * 100).toFixed(0)}% confidence
                      </Badge>
                    </div>
                    <p className="text-sm">{card.reasoning.final_decision}</p>
                  </div>
                </div>
              </div>
            </CollapsibleContent>
          </Collapsible>

          <Separator className="my-4" />

          {/* Action Buttons */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-1">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  setIsLiked(!isLiked);
                  onLike?.(card.id);
                }}
                className={cn(
                  "gap-1",
                  isLiked && "text-red-600 hover:text-red-700"
                )}
              >
                <Heart className={cn("w-4 h-4", isLiked && "fill-current")} />
                <span className="text-xs">
                  {card.social_metrics?.likes || 0}
                </span>
              </Button>
              
              <Button
                variant="ghost"
                size="sm"
                onClick={() => onComment?.(card.id)}
                className="gap-1"
              >
                <MessageCircle className="w-4 h-4" />
                <span className="text-xs">
                  {card.social_metrics?.comments || 0}
                </span>
              </Button>
              
              <Button
                variant="ghost"
                size="sm"
                onClick={() => onShare?.(card.id)}
                className="gap-1"
              >
                <Share2 className="w-4 h-4" />
                <span className="text-xs">
                  {card.social_metrics?.shares || 0}
                </span>
              </Button>
            </div>
            
            <div className="flex items-center gap-1 text-xs text-muted-foreground">
              <Eye className="w-3 h-3" />
              <span>{card.social_metrics?.views || 0}</span>
            </div>
          </div>
        </CardContent>
      </Card>
    </motion.div>
  );
}
