@echo off
REM Push Sentinel-Nexus Integration to GitHub
REM Run this from the traderx root directory

echo ============================================
echo Pushing Sentinel-Nexus to GitHub
echo ============================================
echo.

REM Check if we're in git repo
if not exist .git (
    echo ERROR: Not a git repository!
    exit /b 1
)

REM Show current branch
echo Current branch:
git branch --show-current
echo.

REM Add all changes
echo Adding all changes...
git add -A

REM Show status
echo.
echo Git status:
git status --short
echo.

REM Commit
echo Committing changes...
git commit -m "feat(sentinel-nexus): Complete live trading architecture integration

Integrate 8 production-ready repositories:
- TradingAgents: Multi-agent reasoning and sentiment analysis
- FinRL-Trading: RL-based portfolio weight generation
- AgentTrade: Node.js Fastify matching engine
- NautilusTrader: Rust-native deterministic execution
- QuestDB: High-frequency tick data storage
- JaxMARL: GPU-accelerated MARL agents
- FinnewsHunter: Live news sentiment analysis
- UltraLowLatencyFeedHandler: C++ ITCH/FIX parser

Features:
- Docker Compose orchestration
- Paper and live trading modes
- Real-time ML signal generation
- Risk circuit breakers
- Comprehensive monitoring"

REM Push
echo.
echo Pushing to GitHub...
git push origin sentinel-nexus-integration

echo.
echo ============================================
echo Push complete!
echo ============================================
echo.
echo Verify at: https://github.com/yourusername/traderx/tree/sentinel-nexus-integration
pause
