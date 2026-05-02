import React, { useState, useEffect } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, BarChart, Bar, PieChart, Pie, Cell } from 'recharts';
import { Activity, Cpu, Memory, AlertTriangle, CheckCircle, Clock, Zap, TrendingUp } from 'lucide-react';
import io from 'socket.io-client';

interface ModelMetrics {
  timestamp: string;
  model_name: string;
  latency_ms: number;
  memory_usage_mb: number;
  cpu_usage_percent: number;
  accuracy_score?: number;
  request_count: number;
  error_count: number;
  error_rate: number;
}

interface ModelHealth {
  model_name: string;
  status: 'healthy' | 'degraded' | 'critical' | 'offline';
  reason?: string;
  last_updated: string;
}

interface ProviderStats {
  provider: 'ollama' | 'openai' | 'anthropic' | 'hybrid';
  request_count: number;
  success_count: number;
  avg_latency_ms: number;
  error_rate: number;
}

const COLORS = {
  ollama: '#10b981',
  openai: '#3b82f6',
  anthropic: '#f59e0b',
  hybrid: '#8b5cf6',
  healthy: '#10b981',
  degraded: '#f59e0b',
  critical: '#ef4444',
  offline: '#6b7280'
};

const ModelPerformanceDashboard: React.FC = () => {
  const [metrics, setMetrics] = useState<ModelMetrics[]>([]);
  const [health, setHealth] = useState<ModelHealth[]>([]);
  const [providerStats, setProviderStats] = useState<ProviderStats[]>([]);
  const [selectedModel, setSelectedModel] = useState<string>('');
  const [isConnected, setIsConnected] = useState<boolean>(false);
  const [lastUpdate, setLastUpdate] = useState<string>('');

  useEffect(() => {
    // Connect to WebSocket for real-time updates
    const socket = io('ws://localhost:8080', {
      transports: ['websocket'],
      reconnection: true,
      reconnectionDelay: 1000,
      reconnectionAttempts: 5
    });

    socket.on('connect', () => {
      console.log('Connected to metrics server');
      setIsConnected(true);
    });

    socket.on('disconnect', () => {
      console.log('Disconnected from metrics server');
      setIsConnected(false);
    });

    socket.on('model_metrics', (data: ModelMetrics) => {
      setMetrics(prev => {
        const newMetrics = [...prev, data];
        // Keep only last 1000 data points per model
        const filtered = newMetrics.filter(m => {
          const countForModel = newMetrics.filter(x => x.model_name === m.model_name).length;
          return countForModel <= 1000;
        });
        return filtered;
      });
      setLastUpdate(new Date().toLocaleTimeString());
    });

    socket.on('model_health', (data: ModelHealth) => {
      setHealth(prev => {
        const filtered = prev.filter(h => h.model_name !== data.model_name);
        return [...filtered, data];
      });
    });

    socket.on('provider_stats', (data: ProviderStats[]) => {
      setProviderStats(data);
    });

    return () => {
      socket.disconnect();
    };
  }, []);

  // Calculate aggregated stats
  const totalRequests = metrics.reduce((sum, m) => sum + m.request_count, 0);
  const totalErrors = metrics.reduce((sum, m) => sum + m.error_count, 0);
  const overallErrorRate = totalRequests > 0 ? (totalErrors / totalRequests) * 100 : 0;
  const avgLatency = metrics.length > 0 
    ? metrics.reduce((sum, m) => sum + m.latency_ms, 0) / metrics.length 
    : 0;

  // Get unique models
  const uniqueModels = Array.from(new Set(metrics.map(m => m.model_name)));

  // Filter metrics for selected model
  const selectedMetrics = selectedModel 
    ? metrics.filter(m => m.model_name === selectedModel)
    : metrics;

  return (
    <div className="min-h-screen bg-gray-50 p-6">
      {/* Header */}
      <div className="mb-8">
        <div className="flex justify-between items-center">
          <div>
            <h1 className="text-3xl font-bold text-gray-900">LLM Model Performance Dashboard</h1>
            <p className="text-gray-600 mt-1">Real-time monitoring of Ollama and cloud LLM providers</p>
          </div>
          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-2">
              <div className={`w-3 h-3 rounded-full ${isConnected ? 'bg-green-500' : 'bg-red-500'}`}></div>
              <span className="text-sm text-gray-600">
                {isConnected ? 'Connected' : 'Disconnected'}
              </span>
            </div>
            <div className="text-sm text-gray-500">
              Last update: {lastUpdate}
            </div>
          </div>
        </div>
      </div>

      {/* KPI Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <div className="bg-white rounded-lg shadow p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Total Requests</p>
              <p className="text-2xl font-bold text-gray-900">{totalRequests.toLocaleString()}</p>
            </div>
            <Activity className="h-8 w-8 text-blue-500" />
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Average Latency</p>
              <p className="text-2xl font-bold text-gray-900">{avgLatency.toFixed(2)}ms</p>
            </div>
            <Clock className="h-8 w-8 text-green-500" />
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Error Rate</p>
              <p className={`text-2xl font-bold ${overallErrorRate > 5 ? 'text-red-600' : 'text-green-600'}`}>
                {overallErrorRate.toFixed(2)}%
              </p>
            </div>
            <AlertTriangle className={`h-8 w-8 ${overallErrorRate > 5 ? 'text-red-500' : 'text-green-500'}`} />
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Active Models</p>
              <p className="text-2xl font-bold text-gray-900">{uniqueModels.length}</p>
            </div>
            <Cpu className="h-8 w-8 text-purple-500" />
          </div>
        </div>
      </div>

      {/* Model Selection and Health Status */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-8">
        {/* Model Selection */}
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Model Selection</h2>
          <select 
            value={selectedModel} 
            onChange={(e) => setSelectedModel(e.target.value)}
            className="w-full p-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="">All Models</option>
            {uniqueModels.map(model => (
              <option key={model} value={model}>{model}</option>
            ))}
          </select>
        </div>

        {/* Health Status */}
        <div className="bg-white rounded-lg shadow p-6 lg:col-span-2">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Model Health Status</h2>
          <div className="space-y-3">
            {health.map(h => (
              <div key={h.model_name} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                <div className="flex items-center space-x-3">
                  <div className={`w-3 h-3 rounded-full`} 
                    style={{ backgroundColor: COLORS[h.status] }}>
                  </div>
                  <span className="font-medium text-gray-900">{h.model_name}</span>
                  {h.reason && <span className="text-sm text-gray-600">({h.reason})</span>}
                </div>
                <div className="flex items-center space-x-2">
                  {h.status === 'healthy' && <CheckCircle className="h-4 w-4 text-green-500" />}
                  {h.status === 'degraded' && <AlertTriangle className="h-4 w-4 text-yellow-500" />}
                  {h.status === 'critical' && <AlertTriangle className="h-4 w-4 text-red-500" />}
                  <span className="text-sm text-gray-500">
                    {new Date(h.last_updated).toLocaleTimeString()}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Latency Chart */}
      <div className="bg-white rounded-lg shadow p-6 mb-8">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">
          Response Latency {selectedModel && `- ${selectedModel}`}
        </h2>
        <ResponsiveContainer width="100%" height={300}>
          <LineChart data={selectedMetrics.slice(-100)}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis 
              dataKey="timestamp" 
              tickFormatter={(value) => new Date(value).toLocaleTimeString()}
            />
            <YAxis />
            <Tooltip 
              labelFormatter={(value) => new Date(value).toLocaleString()}
              formatter={(value: any) => [`${value.toFixed(2)}ms`, 'Latency']}
            />
            <Legend />
            <Line 
              type="monotone" 
              dataKey="latency_ms" 
              stroke="#3b82f6" 
              strokeWidth={2}
              dot={false}
              name="Latency (ms)"
            />
          </LineChart>
        </ResponsiveContainer>
      </div>

      {/* Provider Distribution */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
        {/* Provider Stats Bar Chart */}
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Provider Performance</h2>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={providerStats}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="provider" />
              <YAxis />
              <Tooltip />
              <Legend />
              <Bar dataKey="request_count" fill="#3b82f6" name="Requests" />
              <Bar dataKey="success_count" fill="#10b981" name="Success" />
            </BarChart>
          </ResponsiveContainer>
        </div>

        {/* Provider Distribution Pie Chart */}
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Request Distribution</h2>
          <ResponsiveContainer width="100%" height={300}>
            <PieChart>
              <Pie
                data={providerStats}
                cx="50%"
                cy="50%"
                labelLine={false}
                label={({ provider, value, percent }) => `${provider}: ${(percent * 100).toFixed(0)}%`}
                outerRadius={80}
                fill="#8884d8"
                dataKey="request_count"
              >
                {providerStats.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={COLORS[entry.provider]} />
                ))}
              </Pie>
              <Tooltip />
            </PieChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Memory and CPU Usage */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Memory Usage */}
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Memory Usage</h2>
          <ResponsiveContainer width="100%" height={250}>
            <LineChart data={selectedMetrics.slice(-100)}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis 
                dataKey="timestamp" 
                tickFormatter={(value) => new Date(value).toLocaleTimeString()}
              />
              <YAxis />
              <Tooltip 
                labelFormatter={(value) => new Date(value).toLocaleString()}
                formatter={(value: any) => [`${value.toFixed(2)}MB`, 'Memory']}
              />
              <Legend />
              <Line 
                type="monotone" 
                dataKey="memory_usage_mb" 
                stroke="#8b5cf6" 
                strokeWidth={2}
                dot={false}
                name="Memory (MB)"
              />
            </LineChart>
          </ResponsiveContainer>
        </div>

        {/* CPU Usage */}
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">CPU Usage</h2>
          <ResponsiveContainer width="100%" height={250}>
            <LineChart data={selectedMetrics.slice(-100)}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis 
                dataKey="timestamp" 
                tickFormatter={(value) => new Date(value).toLocaleTimeString()}
              />
              <YAxis />
              <Tooltip 
                labelFormatter={(value) => new Date(value).toLocaleString()}
                formatter={(value: any) => [`${value.toFixed(2)}%`, 'CPU']}
              />
              <Legend />
              <Line 
                type="monotone" 
                dataKey="cpu_usage_percent" 
                stroke="#f59e0b" 
                strokeWidth={2}
                dot={false}
                name="CPU (%)"
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>
  );
};

export default ModelPerformanceDashboard;
