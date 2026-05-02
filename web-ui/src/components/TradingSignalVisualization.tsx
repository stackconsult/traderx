import React, { useState, useEffect } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, AreaChart, Area, BarChart, Bar, ScatterChart, Scatter } from 'recharts';
import { TrendingUp, TrendingDown, Brain, Activity, AlertCircle, CheckCircle, Clock, Zap, Target, ArrowRight, Eye, EyeOff } from 'lucide-react';

interface LlmDecision {
  decision_id: string;
  timestamp: string;
  signal_id: string;
  symbol: string;
  direction: 'long' | 'short' | 'hold';
  conviction: number;
  model_used: string;
  provider_used: string;
  reasoning: string;
  confidence_score: number;
  risk_assessment: {
    level: 'low' | 'medium' | 'high' | 'critical';
    factors: string[];
  };
  context: {
    market_conditions: string;
    volatility: number;
    volume: number;
    price: number;
  };
  execution_details?: {
    order_id?: string;
    execution_time_ms?: number;
    fill_price?: number;
    slippage_bps?: number;
  };
}

interface ModelAttribution {
  model_name: string;
  contribution: number;
  confidence: number;
  key_factors: string[];
}

const TradingSignalVisualization: React.FC = () => {
  const [decisions, setDecisions] = useState<LlmDecision[]>([]);
  const [selectedDecision, setSelectedDecision] = useState<LlmDecision | null>(null);
  const [showReasoning, setShowReasoning] = useState<Record<string, boolean>>({});
  const [timeRange, setTimeRange] = useState<'1h' | '24h' | '7d' | '30d'>('24h');
  const [filterSymbol, setFilterSymbol] = useState<string>('');

  useEffect(() => {
    // Mock data - in production, this would come from WebSocket/API
    const mockDecisions: LlmDecision[] = [
      {
        decision_id: '1',
        timestamp: new Date(Date.now() - 3600000).toISOString(),
        signal_id: 'sig-001',
        symbol: 'AAPL',
        direction: 'long',
        conviction: 0.85,
        model_used: 'deepseek-coder:1.3b',
        provider_used: 'ollama',
        reasoning: 'Strong bullish momentum detected with increasing volume. Technical indicators show RSI oversold reversal pattern. EMA crossover confirmed with 85% confidence.',
        confidence_score: 0.92,
        risk_assessment: {
          level: 'medium',
          factors: ['High volatility', 'Earnings approaching', 'Market uncertainty']
        },
        context: {
          market_conditions: 'Bullish with moderate volatility',
          volatility: 0.023,
          volume: 45000000,
          price: 178.50
        },
        execution_details: {
          order_id: 'ord-001',
          execution_time_ms: 125,
          fill_price: 178.52,
          slippage_bps: 2
        }
      },
      {
        decision_id: '2',
        timestamp: new Date(Date.now() - 7200000).toISOString(),
        signal_id: 'sig-002',
        symbol: 'GOOGL',
        direction: 'short',
        conviction: 0.72,
        model_used: 'gemma-mini:2b',
        provider_used: 'ollama',
        reasoning: 'Bearish divergence on daily chart. Volume declining while price attempts to break resistance. Risk/reward ratio favorable for short position.',
        confidence_score: 0.78,
        risk_assessment: {
          level: 'medium',
          factors: ['Resistance level', 'Declining volume']
        },
        context: {
          market_conditions: 'Bearish consolidation',
          volatility: 0.018,
          volume: 12000000,
          price: 141.20
        }
      },
      {
        decision_id: '3',
        timestamp: new Date(Date.now() - 10800000).toISOString(),
        signal_id: 'sig-003',
        symbol: 'MSFT',
        direction: 'long',
        conviction: 0.91,
        model_used: 'deepseek-coder:1.3b',
        provider_used: 'ollama',
        reasoning: 'Strong fundamental catalyst with technical confirmation. Institutional accumulation detected. Breakout from consolidation pattern with high conviction.',
        confidence_score: 0.95,
        risk_assessment: {
          level: 'low',
          factors: ['Strong fundamentals', 'Technical confirmation']
        },
        context: {
          market_conditions: 'Strong bullish trend',
          volatility: 0.015,
          volume: 22000000,
          price: 378.90
        },
        execution_details: {
          order_id: 'ord-003',
          execution_time_ms: 98,
          fill_price: 378.95,
          slippage_bps: 1
        }
      }
    ];

    setDecisions(mockDecisions);
  }, []);

  const filteredDecisions = decisions.filter(d => 
    !filterSymbol || d.symbol.toLowerCase() === filterSymbol.toLowerCase()
  );

  const convictionData = filteredDecisions.map(d => ({
    time: new Date(d.timestamp).toLocaleTimeString(),
    conviction: d.conviction * 100,
    confidence: d.confidence_score * 100,
  }));

  const directionCounts = filteredDecisions.reduce((acc, d) => {
    acc[d.direction] = (acc[d.direction] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  const riskDistribution = filteredDecisions.reduce((acc, d) => {
    acc[d.risk_assessment.level] = (acc[d.risk_assessment.level] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  const modelUsage = filteredDecisions.reduce((acc, d) => {
    acc[d.model_used] = (acc[d.model_used] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  const toggleReasoning = (decisionId: string) => {
    setShowReasoning(prev => ({
      ...prev,
      [decisionId]: !prev[decisionId]
    }));
  };

  const getRiskColor = (level: string) => {
    switch (level) {
      case 'low': return '#10b981';
      case 'medium': return '#f59e0b';
      case 'high': return '#f97316';
      case 'critical': return '#ef4444';
      default: return '#6b7280';
    }
  };

  return (
    <div className="min-h-screen bg-gray-50 p-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900">LLM Trading Signal Visualization</h1>
        <p className="text-gray-600 mt-1">Real-time transparency of AI-driven trading decisions</p>
      </div>

      {/* Controls */}
      <div className="bg-white rounded-lg shadow p-4 mb-6 flex gap-4 items-center">
        <div className="flex-1">
          <input
            type="text"
            placeholder="Filter by symbol..."
            value={filterSymbol}
            onChange={(e) => setFilterSymbol(e.target.value)}
            className="w-full p-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          />
        </div>
        <div className="flex gap-2">
          {(['1h', '24h', '7d', '30d'] as const).map((range) => (
            <button
              key={range}
              onClick={() => setTimeRange(range)}
              className={`px-4 py-2 rounded-md ${
                timeRange === range
                  ? 'bg-blue-500 text-white'
                  : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
              }`}
            >
              {range}
            </button>
          ))}
        </div>
      </div>

      {/* KPI Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-6">
        <div className="bg-white rounded-lg shadow p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Total Decisions</p>
              <p className="text-2xl font-bold text-gray-900">{filteredDecisions.length}</p>
            </div>
            <Brain className="h-8 w-8 text-blue-500" />
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Avg Conviction</p>
              <p className="text-2xl font-bold text-gray-900">
                {(filteredDecisions.reduce((sum, d) => sum + d.conviction, 0) / filteredDecisions.length * 100).toFixed(1)}%
              </p>
            </div>
            <Target className="h-8 w-8 text-green-500" />
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Avg Confidence</p>
              <p className="text-2xl font-bold text-gray-900">
                {(filteredDecisions.reduce((sum, d) => sum + d.confidence_score, 0) / filteredDecisions.length * 100).toFixed(1)}%
              </p>
            </div>
            <CheckCircle className="h-8 w-8 text-purple-500" />
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Risk Level</p>
              <p className="text-2xl font-bold text-gray-900">
                {Object.keys(riskDistribution).length}
              </p>
            </div>
            <AlertCircle className="h-8 w-8 text-orange-500" />
          </div>
        </div>
      </div>

      {/* Charts Row */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        {/* Conviction Trend */}
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Conviction & Confidence Trend</h2>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={convictionData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="time" />
              <YAxis />
              <Tooltip />
              <Legend />
              <Line type="monotone" dataKey="conviction" stroke="#3b82f6" strokeWidth={2} name="Conviction %" />
              <Line type="monotone" dataKey="confidence" stroke="#10b981" strokeWidth={2} name="Confidence %" />
            </LineChart>
          </ResponsiveContainer>
        </div>

        {/* Direction Distribution */}
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Direction Distribution</h2>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={Object.entries(directionCounts).map(([name, value]) => ({ name, value }))}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="name" />
              <YAxis />
              <Tooltip />
              <Bar dataKey="value" fill="#8b5cf6" />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Risk Distribution */}
      <div className="bg-white rounded-lg shadow p-6 mb-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Risk Assessment Distribution</h2>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {Object.entries(riskDistribution).map(([level, count]) => (
            <div key={level} className="text-center">
              <div className="w-16 h-16 rounded-full mx-auto mb-2 flex items-center justify-center" style={{ backgroundColor: getRiskColor(level) }}>
                <span className="text-white text-2xl font-bold">{count}</span>
              </div>
              <p className="text-sm font-medium text-gray-700 capitalize">{level}</p>
            </div>
          ))}
        </div>
      </div>

      {/* Decision List */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Recent LLM Decisions</h2>
        <div className="space-y-4">
          {filteredDecisions.map((decision) => (
            <div key={decision.decision_id} className="border border-gray-200 rounded-lg p-4 hover:bg-gray-50 transition-colors">
              <div className="flex items-start justify-between mb-3">
                <div className="flex items-center gap-3">
                  {decision.direction === 'long' ? (
                    <TrendingUp className="h-5 w-5 text-green-500" />
                  ) : decision.direction === 'short' ? (
                    <TrendingDown className="h-5 w-5 text-red-500" />
                  ) : (
                    <Activity className="h-5 w-5 text-gray-500" />
                  )}
                  <div>
                    <p className="font-semibold text-gray-900">{decision.symbol}</p>
                    <p className="text-sm text-gray-600 capitalize">{decision.direction}</p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="text-sm text-gray-500">{new Date(decision.timestamp).toLocaleString()}</p>
                  <p className="text-sm font-medium">
                    Conviction: {(decision.conviction * 100).toFixed(0)}%
                  </p>
                </div>
              </div>

              <div className="grid grid-cols-3 gap-4 mb-3">
                <div>
                  <p className="text-xs text-gray-500">Model</p>
                  <p className="text-sm font-medium">{decision.model_used}</p>
                </div>
                <div>
                  <p className="text-xs text-gray-500">Provider</p>
                  <p className="text-sm font-medium capitalize">{decision.provider_used}</p>
                </div>
                <div>
                  <p className="text-xs text-gray-500">Risk Level</p>
                  <p 
                    className="text-sm font-medium capitalize"
                    style={{ color: getRiskColor(decision.risk_assessment.level) }}
                  >
                    {decision.risk_assessment.level}
                  </p>
                </div>
              </div>

              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <div className="w-24 bg-gray-200 rounded-full h-2">
                    <div 
                      className="bg-blue-500 h-2 rounded-full" 
                      style={{ width: `${decision.confidence_score * 100}%` }}
                    />
                  </div>
                  <span className="text-xs text-gray-600">Confidence: {(decision.confidence_score * 100).toFixed(0)}%</span>
                </div>
                <button
                  onClick={() => toggleReasoning(decision.decision_id)}
                  className="flex items-center gap-1 text-sm text-blue-500 hover:text-blue-700"
                >
                  {showReasoning[decision.decision_id] ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                  {showReasoning[decision.decision_id] ? 'Hide' : 'Show'} Reasoning
                </button>
              </div>

              {showReasoning[decision.decision_id] && (
                <div className="mt-4 p-4 bg-blue-50 rounded-md">
                  <p className="text-sm text-gray-700 mb-3">
                    <strong>Reasoning:</strong> {decision.reasoning}
                  </p>
                  <div className="mb-2">
                    <p className="text-xs text-gray-500 mb-1">Risk Factors:</p>
                    <div className="flex flex-wrap gap-2">
                      {decision.risk_assessment.factors.map((factor, idx) => (
                        <span key={idx} className="px-2 py-1 bg-orange-100 text-orange-700 rounded text-xs">
                          {factor}
                        </span>
                      ))}
                    </div>
                  </div>
                  <div>
                    <p className="text-xs text-gray-500 mb-1">Market Context:</p>
                    <p className="text-sm text-gray-700">{decision.context.market_conditions}</p>
                    <p className="text-xs text-gray-600 mt-1">
                      Vol: {(decision.context.volume / 1000000).toFixed(1)}M | 
                      Volatility: {(decision.context.volatility * 100).toFixed(2)}% | 
                      Price: ${decision.context.price.toFixed(2)}
                    </p>
                  </div>
                  {decision.execution_details && (
                    <div className="mt-3 pt-3 border-t border-blue-200">
                      <p className="text-xs text-gray-500 mb-1">Execution Details:</p>
                      <p className="text-sm text-gray-700">
                        Order ID: {decision.execution_details.order_id} | 
                        Time: {decision.execution_details.execution_time_ms}ms | 
                        Fill: ${decision.execution_details.fill_price?.toFixed(2)} | 
                        Slippage: {decision.execution_details.slippage_bps} bps
                      </p>
                    </div>
                  )}
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default TradingSignalVisualization;
