export interface Strategy {
  id: string;
  tenant_id: string;
  name: string;
  description: string;
  status: 'ACTIVE' | 'PAUSED' | 'STOPPED';
  created_at: string;
  updated_at: string;
  metadata: Record<string, any>;
}

export interface Execution {
  id: string;
  strategy_id: string;
  status: 'PENDING' | 'ACCEPTED' | 'WORKING' | 'COMPLETE' | 'FAILED';
  confidence_score: number;
  signal_strength: number;
  created_at: string;
  completed_at?: string;
  error_message?: string;
}

export interface ReasoningStep {
  type: 'ANALYSIS' | 'SIGNAL' | 'RISK_CHECK' | 'DECISION';
  description: string;
  confidence: number;
  data?: Record<string, any>;
  timestamp: string;
}

export interface ReasoningTrace {
  id: string;
  execution_id: string;
  agent: 'claude-4.6' | 'gemma-4';
  steps: ReasoningStep[];
  final_decision: string;
  total_confidence: number;
  created_at: string;
}

export interface ActionCard {
  id: string;
  strategy: Strategy;
  execution: Execution;
  reasoning: ReasoningTrace;
  market_data?: {
    symbol: string;
    price: number;
    change_24h: number;
    volume: number;
  };
  social_metrics?: {
    views: number;
    likes: number;
    comments: number;
    shares: number;
  };
}

export interface ConfidenceScore {
  score: number;
  label: string;
  color: string;
}

export interface RationaleSummary {
  title: string;
  description: string;
  key_factors: string[];
}

export interface StrategyFeedResponse {
  cards: ActionCard[];
  next_cursor?: string;
  has_more: boolean;
}
