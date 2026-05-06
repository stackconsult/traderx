// Model Comparison Dashboard Component
// Phase C: Comparative Testing - Task C5: Dashboard comparative visualization

import React from 'react';

interface ModelMetrics {
  name: string;
  totalReturn: number;
  sharpeRatio: number;
  maxDrawdown: number;
  winRate: number;
  tradesExecuted: number;
}

interface ModelComparisonProps {
  models: ModelMetrics[];
  winner?: string;
}

export const ModelComparison: React.FC<ModelComparisonProps> = ({ models, winner }) => {
  const metrics = ['totalReturn', 'sharpeRatio', 'maxDrawdown', 'winRate', 'tradesExecuted'];
  
  const formatMetric = (key: string, value: number): string => {
    switch (key) {
      case 'totalReturn':
      case 'maxDrawdown':
      case 'winRate':
        return `${(value * 100).toFixed(2)}%`;
      case 'sharpeRatio':
        return value.toFixed(2);
      case 'tradesExecuted':
        return value.toString();
      default:
        return value.toString();
    }
  };

  const getMetricLabel = (key: string): string => {
    const labels: Record<string, string> = {
      totalReturn: 'Total Return',
      sharpeRatio: 'Sharpe Ratio',
      maxDrawdown: 'Max Drawdown',
      winRate: 'Win Rate',
      tradesExecuted: 'Trades Executed',
    };
    return labels[key] || key;
  };

  const isWinner = (modelName: string): boolean => modelName === winner;

  return (
    <div className="model-comparison">
      <h2>Model Comparison Dashboard</h2>
      
      <div className="model-cards">
        {models.map((model) => (
          <div
            key={model.name}
            className={`model-card ${isWinner(model.name) ? 'winner' : ''}`}
          >
            <h3>{model.name} {isWinner(model.name) && '🏆'}</h3>
            
            <div className="metrics">
              {metrics.map((metric) => (
                <div key={metric} className="metric">
                  <span className="metric-label">{getMetricLabel(metric)}:</span>
                  <span className="metric-value">{formatMetric(metric, model[metric as keyof ModelMetrics] as number)}</span>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default ModelComparison;
