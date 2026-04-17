# Overstory - Self-Evolving Swarm Dashboard
FROM oven/bun:latest

WORKDIR /app

# Copy package files
COPY package.json bun.lockb ./

# Install dependencies
RUN bun install

# Copy application code
COPY . .

# Build if needed
RUN bun run build || true

# Expose dashboard port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Start the dashboard and orchestrator
CMD ["bun", "run", "start"]
