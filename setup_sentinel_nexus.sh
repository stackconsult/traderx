#!/bin/bash
# Sentinel-Nexus Architecture Setup Script
# This script sets up the complete live trading system

set -e  # Exit on any error

echo "=========================================="
echo "Sentinel-Nexus Setup Script"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored messages
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "docker-compose.sentinel.yml" ]; then
    print_error "Please run this script from the traderx root directory"
    exit 1
fi

# Check dependencies
print_status "Checking dependencies..."

# Check Docker
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed. Please install Docker first."
    print_error "Visit: https://docs.docker.com/get-docker/"
    exit 1
fi

# Check Docker Compose
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    print_error "Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Check Git
if ! command -v git &> /dev/null; then
    print_error "Git is not installed. Please install Git first."
    exit 1
fi

print_status "All dependencies found!"
echo ""

# Create necessary directories
print_status "Creating directory structure..."
mkdir -p config/{prometheus,grafana/dashboards,grafana/datasources,ssl}
mkdir -p models
mkdir -p logs
mkdir -p data/{questdb,redis,postgres}
print_status "Directory structure created"
echo ""

# Copy environment template if .env doesn't exist
if [ ! -f ".env" ]; then
    print_status "Creating .env file from template..."
    cat > .env << 'EOF'
# ============================================
# SENTINEL-NEXUS CONFIGURATION
# ============================================

# Trading Mode: paper, live, or backtest
TRADING_MODE=paper

# ============================================
# ALPACA MARKETS API (Required)
# Get your keys at: https://alpaca.markets
# ============================================
ALPACA_API_KEY=your_alpaca_api_key_here
ALPACA_API_SECRET=your_alpaca_secret_here
ALPACA_PAPER=true

# ============================================
# LLM API KEYS (Required for AI Agents)
# ============================================
OPENAI_API_KEY=your_openai_key_here
ANTHROPIC_API_KEY=your_anthropic_key_here

# ============================================
# NEWS DATA SOURCES (Optional)
# ============================================
FINNHUB_API_KEY=your_finnhub_key_here
NEWSAPI_KEY=your_newsapi_key_here
TWITTER_BEARER_TOKEN=your_twitter_token_here
REDDIT_CLIENT_ID=your_reddit_client_id
REDDIT_CLIENT_SECRET=your_reddit_secret

# ============================================
# DATABASE CONFIGURATION
# ============================================
QUESTDB_HOST=questdb
QUESTDB_PORT=8812
QUESTDB_USER=admin
QUESTDB_PASSWORD=quest

REDIS_URL=redis://redis:6379

POSTGRES_USER=traderx
POSTGRES_PASSWORD=traderx_secret
POSTGRES_DB=agenttrade

# ============================================
# TRADING CONFIGURATION
# ============================================
# Risk Management
MAX_POSITION_SIZE=0.10
MAX_DRAWDOWN=0.05
DAILY_LOSS_LIMIT=-10000

# Rebalancing
REBALANCE_INTERVAL=300
LOOKBACK_MINUTES=60

# Symbols to trade (comma-separated)
TRADING_SYMBOLS=AAPL,MSFT,GOOGL,AMZN,TSLA,NVDA,META,NFLX

# ============================================
# SYSTEM CONFIGURATION
# ============================================
LOG_LEVEL=INFO
RUST_LOG=info

# GPU Configuration (for JaxMARL)
CUDA_VISIBLE_DEVICES=0
JAX_PLATFORM_NAME=gpu
EOF

    print_warning ".env file created with default values"
    print_warning "Please edit .env and add your API keys before starting the system"
    echo ""
fi

# Check if .env has been configured
if grep -q "your_.*_here" .env; then
    print_warning "WARNING: .env file contains placeholder values"
    print_warning "Please edit .env and replace all 'your_*_here' with actual API keys"
    echo ""
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_status "Setup aborted. Please configure .env file first."
        exit 0
    fi
fi

# Create Prometheus configuration
print_status "Creating Prometheus configuration..."
cat > config/prometheus.yml << 'EOF'
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  # - "rules.yml"

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'ai-agents'
    static_configs:
      - targets: ['ai-agents:8081']
    metrics_path: /metrics

  - job_name: 'oms-engine'
    static_configs:
      - targets: ['oms-engine:8081']
    metrics_path: /metrics

  - job_name: 'questdb'
    static_configs:
      - targets: ['questdb:9003']
EOF
print_status "Prometheus configuration created"

# Create Grafana datasource configuration
print_status "Creating Grafana configuration..."
cat > config/grafana/datasources/datasources.yml << 'EOF'
apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true

  - name: QuestDB
    type: postgres
    url: questdb:8812
    database: qdb
    user: admin
    secureJsonData:
      password: quest
EOF
print_status "Grafana configuration created"

# Create Nginx configuration
print_status "Creating Nginx configuration..."
cat > config/nginx.conf << 'EOF'
events {
    worker_connections 1024;
}

http {
    upstream ai_agents {
        server ai-agents:8080;
    }

    upstream agenttrade {
        server agenttrade:3001;
    }

    upstream oms_engine {
        server oms-engine:8080;
    }

    server {
        listen 80;
        server_name localhost;

        location /api/agents {
            proxy_pass http://ai_agents;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        location /api/matching {
            proxy_pass http://agenttrade;
            proxy_set_header Host $host;
        }

        location /api/oms {
            proxy_pass http://oms_engine;
            proxy_set_header Host $host;
        }

        location / {
            return 200 "Sentinel-Nexus API Gateway\n";
            add_header Content-Type text/plain;
        }
    }
}
EOF
print_status "Nginx configuration created"

# Build and start services
print_status "Building and starting services..."
echo ""

# Pull latest images first
print_status "Pulling Docker images..."
docker-compose -f docker-compose.sentinel.yml pull questdb redis postgres

# Build custom images
print_status "Building custom Docker images..."
docker-compose -f docker-compose.sentinel.yml build --parallel

# Start infrastructure services first
print_status "Starting infrastructure services..."
docker-compose -f docker-compose.sentinel.yml up -d questdb redis postgres

# Wait for services to be healthy
print_status "Waiting for services to be healthy..."
sleep 10

# Check service health
print_status "Checking service health..."
docker-compose -f docker-compose.sentinel.yml ps

# Start application services
print_status "Starting application services..."
docker-compose -f docker-compose.sentinel.yml up -d ai-agents finrl-trading agenttrade oms-engine

echo ""
print_status "Setup complete!"
echo ""
echo "=========================================="
echo "Services Status:"
echo "=========================================="
docker-compose -f docker-compose.sentinel.yml ps
echo ""
echo "=========================================="
echo "Access Points:"
echo "=========================================="
echo "QuestDB Console:    http://localhost:9000"
echo "Grafana Dashboard:  http://localhost:3000 (admin/admin)"
echo "Prometheus:         http://localhost:9090"
echo "Redis Commander:    redis://localhost:6379"
echo ""
echo "=========================================="
echo "Next Steps:"
echo "=========================================="
echo "1. Verify QuestDB is running: http://localhost:9000"
echo "2. Check Grafana dashboards: http://localhost:3000"
echo "3. View logs: docker-compose -f docker-compose.sentinel.yml logs -f"
echo "4. Start live trading: docker-compose -f docker-compose.sentinel.yml up -d"
echo ""
echo "To stop all services:"
echo "  docker-compose -f docker-compose.sentinel.yml down"
echo ""
echo "To stop and remove all data:"
echo "  docker-compose -f docker-compose.sentinel.yml down -v"
echo ""
