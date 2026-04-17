#!/bin/bash
# Push Sentinel-Nexus Integration to GitHub
# Run this from the traderx root directory

set -e

echo "============================================"
echo "Pushing Sentinel-Nexus to GitHub"
echo "============================================"
echo ""

# Check if we're in git repo
if [ ! -d .git ]; then
    echo "ERROR: Not a git repository!"
    exit 1
fi

# Show current branch
echo "Current branch:"
git branch --show-current
echo ""

# Add all changes
echo "Adding all changes..."
git add -A

# Show status
echo ""
echo "Git status:"
git status --short
echo ""

# Commit
echo "Committing changes..."
git commit -m "feat(sentinel-nexus): Complete live trading architecture integration

Integrate 8 production-ready repositories:
- TradingAgents: Multi-agent reasoning and sentiment analysis
- FinRL-Trading: RL-based portfolio weight generation (S→A→T→R pipeline)
- AgentTrade: Node.js Fastify matching engine for internal order routing
- NautilusTrader: Rust-native deterministic execution core
- QuestDB: High-frequency tick data storage with nanosecond recall
- JaxMARL: GPU-accelerated multi-agent RL for HFT strategies
- FinnewsHunter: Live news sentiment and alternative data
- UltraLowLatencyFeedHandler: C++ ITCH/FIX protocol parser

Features:
- Docker Compose orchestration for all services
- Paper and live trading via Alpaca API
- Real-time ML signal generation
- Risk circuit breakers and drawdown protection
- QuestDB time-series storage for tick data
- Redis pub/sub for inter-service communication
- Prometheus/Grafana monitoring stack
- Comprehensive setup scripts and documentation

Infrastructure:
- docker-compose.sentinel.yml with 10+ services
- setup_sentinel_nexus.sh automation script
- Production Dockerfiles for Python services
- Integration with existing TraderX execution layer

Refs: sentinel-nexus-integration"

# Push
echo ""
echo "Pushing to GitHub..."
git push origin sentinel-nexus-integration

echo ""
echo "============================================"
echo "Push complete!"
echo "============================================"
echo ""
echo "Verify at: https://github.com/yourusername/traderx/tree/sentinel-nexus-integration"
