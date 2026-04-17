# ClawTeam - AI Hedge Fund Swarm with Kanban UI
FROM python:3.11-slim

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements
COPY requirements.txt ./

# Install Python dependencies
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY . .

# Install ClawTeam in editable mode
RUN pip install -e .

# Expose dashboard port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command - serve the Kanban board
CMD ["clawteam", "board", "serve", "--port", "8080", "--host", "0.0.0.0"]
