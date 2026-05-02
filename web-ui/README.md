# TraderX LLM Performance Dashboard

Real-time monitoring dashboard for Ollama and cloud LLM providers in the TraderX trading system.

## Features

- **Real-time Metrics**: Live latency, memory, CPU, and accuracy monitoring
- **Model Health**: Health status monitoring with visual indicators
- **Provider Analytics**: Performance comparison across Ollama, OpenAI, Anthropic, and Hybrid providers
- **Interactive Charts**: Time-series visualization of performance metrics
- **WebSocket Integration**: Real-time updates without page refresh

## Quick Start

### Prerequisites
- Node.js 16+ 
- npm or yarn

### Installation

```bash
cd web-ui
npm install
```

### Development

```bash
npm start
```

The dashboard will be available at `http://localhost:3000`

### Production Build

```bash
npm run build
```

## Architecture

### Components

- **ModelPerformanceDashboard**: Main dashboard component
- **Real-time WebSocket**: Connects to metrics server at `ws://localhost:8080`
- **Charts**: Uses Recharts for data visualization
- **Tailwind CSS**: For responsive design

### Data Flow

1. WebSocket connects to metrics server
2. Receives real-time updates for:
   - Model metrics (latency, memory, CPU, accuracy)
   - Model health status
   - Provider statistics
3. Updates React state and re-renders charts

### WebSocket Events

- `model_metrics`: Individual model performance data
- `model_health`: Model health status updates
- `provider_stats`: Aggregated provider statistics

## Configuration

### Environment Variables

Create `.env.local` in the web-ui directory:

```env
REACT_APP_WEBSOCKET_URL=ws://localhost:8080
REACT_APP_API_BASE_URL=http://localhost:8080
```

### Backend Integration

The dashboard expects a WebSocket server at `ws://localhost:8080` that emits:

```typescript
// Model metrics event
{
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

// Model health event
{
  model_name: string;
  status: 'healthy' | 'degraded' | 'critical' | 'offline';
  reason?: string;
  last_updated: string;
}

// Provider stats event
{
  provider: 'ollama' | 'openai' | 'anthropic' | 'hybrid';
  request_count: number;
  success_count: number;
  avg_latency_ms: number;
  error_rate: number;
}
```

## Performance Considerations

- **Data Limiting**: Keeps only last 1000 data points per model
- **Debounced Updates**: WebSocket events are debounced to prevent excessive re-renders
- **Memory Management**: Automatic cleanup of old data points
- **Responsive Design**: Optimized for desktop and mobile viewing

## Troubleshooting

### Common Issues

1. **WebSocket Connection Failed**
   - Check if backend server is running on port 8080
   - Verify WebSocket endpoint is accessible

2. **No Data Displaying**
   - Ensure metrics are being published to WebSocket
   - Check browser console for connection errors

3. **Charts Not Rendering**
   - Verify Recharts dependency is installed
   - Check data format matches expected schema

### Development Tips

- Use React DevTools to inspect component state
- Monitor WebSocket connection in browser Network tab
- Check console for real-time error messages

## Dependencies

- React 18.2.0
- Recharts 2.8.0
- Socket.io-client 4.7.0
- Tailwind CSS 3.3.0
- Lucide React 0.294.0

## License

Part of the TraderX trading system project.
