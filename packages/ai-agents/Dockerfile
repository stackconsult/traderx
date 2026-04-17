# AI Agents Service - Sentinel-Nexus Architecture
FROM python:3.11-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    gcc \
    g++ \
    libffi-dev \
    libssl-dev \
    libpq-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy requirements first for better caching
COPY pyproject.toml ./
COPY src/ ./src/

# Install Python dependencies
RUN pip install --no-cache-dir -e .

# Install additional dependencies for production
RUN pip install --no-cache-dir \
    asyncpg \
    aiohttp \
    websockets \
    redis \
    yfinance \
    textblob \
    TA-Lib

# Download NLTK data for TextBlob
RUN python -m textblob.download_corpora

# Create directories
RUN mkdir -p /app/config /app/logs /app/data

# Copy source code
COPY src/ ./src/

# Set environment variables
ENV PYTHONPATH=/app/src
ENV PYTHONUNBUFFERED=1
ENV LOG_LEVEL=INFO

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD python -c "import sys; sys.path.insert(0, 'src'); from traderx_ai_agents.analysis.base import CircuitBreaker; sys.exit(0)" || exit 1

# Expose port for metrics
EXPOSE 8081

# Run the service
CMD ["python", "-u", "-m", "traderx_ai_agents.enhanced_agent_hierarchy"]
