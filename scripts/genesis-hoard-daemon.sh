#!/bin/bash
# Genesis Hoard Daemon — Persistent tmux session for agent orchestration
# Run this once to start the Genesis agent swarm in a tmux session that survives IDE disconnects
# Usage: bash scripts/genesis-hoard-daemon.sh {start|stop|status|attach}

set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SESSION_NAME="genesis-hoard"
LOG_DIR="$REPO/logs"
LOG_FILE="$LOG_DIR/hoard.log"
PID_FILE="$LOG_DIR/hoard.pid"
GUARDRAIL_CONFIG_DIR="$REPO/config"

mkdir -p "$LOG_DIR"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
ok()   { echo -e "${GREEN}✅ $1${NC}"; }
warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
fail() { echo -e "${RED}❌ $1${NC}"; }

case "${1:-start}" in
  start)
    if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
      warn "Session $SESSION_NAME already exists"
      tmux list-sessions | grep "$SESSION_NAME"
      exit 0
    fi

    echo "Starting Genesis hoard in tmux session: $SESSION_NAME"
    
    # Start the session with the genesis-startup script
    tmux new-session -d -s "$SESSION_NAME" -c "$REPO" "bash scripts/genesis-startup.sh | tee -a $LOG_FILE"
    
    # Save the tmux session PID
    tmux list-sessions -F "#{session_name}:#{session_pid}" | grep "^$SESSION_NAME:" | cut -d: -f2 > "$PID_FILE"
    
    ok "Genesis hoard started in tmux session"
    echo "  Session: $SESSION_NAME"
    echo "  Logs: $LOG_FILE"
    echo "  PID: $(cat $PID_FILE)"
    echo ""
    echo "Attach anytime with: tmux attach -t $SESSION_NAME"
    echo "View logs with: tail -f $LOG_FILE"
    ;;
    
  stop)
    if ! tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
      warn "Session $SESSION_NAME not found"
      exit 0
    fi
    
    echo "Stopping Genesis hoard session..."
    tmux send-keys -t "$SESSION_NAME" C-c
    sleep 2
    tmux kill-session -t "$SESSION_NAME"
    rm -f "$PID_FILE"
    ok "Genesis hoard stopped"
    ;;
    
  status)
    if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
      ok "Genesis hoard is running"
      tmux list-sessions | grep "$SESSION_NAME"
      echo ""
      echo "Last 10 log lines:"
      tail -10 "$LOG_FILE" 2>/dev/null || echo "No logs yet"
    else
      warn "Genesis hoard is not running"
      exit 1
    fi
    ;;
    
  attach)
    if ! tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
      fail "Session $SESSION_NAME not found"
      echo "Start it first: bash scripts/genesis-hoard-daemon.sh start"
      exit 1
    fi
    echo "Attaching to Genesis hoard session (Ctrl+B D to detach)..."
    tmux attach -t "$SESSION_NAME"
    ;;
    
  *)
    echo "Usage: $0 {start|stop|status|attach}"
    exit 1
    ;;
esac
