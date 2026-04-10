FROM python:3.11-slim

# Set working directory
WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    gcc \
    g++ \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements first for better caching
COPY packages/api-gateway/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY packages/api-gateway/src ./src
COPY packages/database/src ./packages/database/src
COPY packages/execution-adapters/src ./packages/execution-adapters/src
COPY packages/handoff/src ./packages/handoff/src

# Create non-root user
RUN useradd --create-home --shell /bin/bash traderx
RUN chown -R traderx:traderx /app
USER traderx

# Expose port
EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Run the application
CMD ["python", "-m", "src.main"]
