# TraderX Production Deployment Guide

## Overview
This guide covers production deployment of TraderX Phase 4: Institutional Sovereign Extension with real infrastructure requirements.

## Infrastructure Prerequisites

### 1. Kernel Requirements (for eBPF/XDP)
- **Minimum**: Linux kernel 5.15+ with XDP support
- **Recommended**: Ubuntu 22.04 LTS or RHEL 9+
- **Verification**: `uname -r` should show kernel >= 5.15
- **Required packages**:
  ```bash
  sudo apt-get update
  sudo apt-get install -y linux-headers-$(uname -r) clang llvm libelf-dev libpcap-dev
  ```

### 2. Network Interface Requirements
- **NIC**: Must support XDP_REDIRECT (most modern 10G+ NICs)
- **Verification**: `ethtool -k eth0 | grep xdp`
- **Required**: `xdp-offload` or `xdp-generic` support

### 3. Database Requirements
- **PostgreSQL**: 15+ with row-level security
- **Configuration**:
  ```ini
  shared_preload_libraries = 'pg_stat_statements,pgcrypto'
  row_security = on
  max_connections = 200
  ```
- **PgBouncer**: Transaction pooling mode required for RLS

### 4. External API Credentials
Required environment variables:

```bash
# Sumsub KYA Service
export SUMSUB_API_TOKEN="your_production_token"
export SUMSUB_API_SECRET="your_production_secret"
export SUMSUB_BASE_URL="https://api.sumsub.com"

# Exchange APIs (Production, NOT testnet)
export BYBIT_API_KEY="your_production_bybit_key"
export BYBIT_API_SECRET="your_production_bybit_secret"
export BINANCE_API_KEY="your_production_binance_key"
export BINANCE_API_SECRET="your_production_binance_secret"

# AI Model APIs
export ANTHROPIC_API_KEY="your_production_anthropic_key"
export GOOGLE_API_KEY="your_production_google_key"
export OPENAI_API_KEY="your_production_openai_key"
```

## Production Deployment Steps

### Step 1: Prepare Infrastructure

```bash
# 1.1 Create production user
sudo useradd -m -s /bin/bash traderx
sudo usermod -aG docker traderx

# 1.2 Create directories
sudo mkdir -p /opt/traderx/{config,data,logs}
sudo chown -R traderx:traderx /opt/traderx

# 1.2 Install Rust for eBPF
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 1.3 Install aya-rs for eBPF
cargo install aya-cli
```

### Step 2: Deploy eBPF/XDP Router

```bash
# 2.1 Build eBPF program
cd packages/dealing-desk/ebpf-router
cargo build --release --target bpfel-unknown-none

# 2.2 Load XDP program
sudo ip link set dev eth0 xdp obj ebpf-router.o sec xdp
sudo ip link set dev eth0 xdp pinned /sys/fs/bpf/traderx-router

# 2.3 Verify XDP is loaded
sudo bpftool prog list
```

### Step 3: Deploy Application Stack

```bash
# 3.1 Clone and configure
git clone https://github.com/your-org/traderx.git
cd traderx

# 3.2 Configure environment
cp .env.example .env
# Edit .env with production credentials

# 3.3 Deploy with Docker Compose
docker-compose -f docker-compose.prod.yml up -d

# 3.4 Verify all services are healthy
docker-compose ps
curl http://localhost:8000/health
```

### Step 4: Initialize Database

```bash
# 4.1 Run migrations
docker-compose exec api-gateway python -m alembic upgrade head

# 4.2 Setup RLS policies
docker-compose exec postgres psql -U traderx -d traderx -f /docker-entrypoint-initdb.d/02-policies/01-rls-policies.sql

# 4.3 Create tenants
docker-compose exec api-gateway python scripts/create_tenants.py
```

### Step 5: Configure Monitoring

```bash
# 5.1 Access Grafana
# URL: http://localhost:3001
# Username: admin
# Password: (from .env)

# 5.2 Access Jaeger
# URL: http://localhost:16686

# 5.3 Access Prometheus
# URL: http://localhost:9090
```

## Feature Flags and Graceful Degradation

The system uses feature flags to handle missing infrastructure:

```python
# Example: eBPF Router
USE_XDP_ROUTING = os.getenv("USE_XDP_ROUTING", "true").lower() == "true"

if USE_XDP_ROUTING and check_xdp_available():
    router = XDPRouter()
else:
    logger.warning("DEGRADED MODE: XDP not available, using userspace routing")
    router = UserspaceRouter()
```

## Security Configuration

### 1. TLS/SSL Setup
```bash
# Generate certificates
certbot --nginx -d traderx.com
```

### 2. Firewall Rules
```bash
# Allow only necessary ports
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow 80/tcp    # HTTP (redirect to HTTPS)
sudo ufw enable
```

### 3. Rate Limiting
```nginx
# In nginx.conf
limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
limit_req zone=api burst=20 nodelay;
```

## Performance Tuning

### 1. System Optimization
```bash
# Increase network buffer sizes
echo 'net.core.rmem_max = 134217728' >> /etc/sysctl.conf
echo 'net.core.wmem_max = 134217728' >> /etc/sysctl.conf

# Optimize for low latency
echo 'net.ipv4.tcp_low_latency = 1' >> /etc/sysctl.conf
echo 'vm.swappiness = 1' >> /etc/sysctl.conf

sysctl -p
```

### 2. PostgreSQL Tuning
```ini
# In postgresql.conf
shared_buffers = 4GB
effective_cache_size = 12GB
work_mem = 256MB
maintenance_work_mem = 1GB
max_worker_processes = 16
max_parallel_workers_per_gather = 4
```

## Monitoring and Alerting

### 1. Critical Metrics to Monitor
- Order latency (p95 < 10ms)
- VPIN threshold breaches
- KYA verification failures
- eBPF packet drops
- Database connection pool usage

### 2. Alert Configuration
```yaml
# In prometheus.yml
alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

rule_files:
  - "traderx_alerts.yml"
```

## Backup and Recovery

### 1. Database Backup
```bash
# Automated daily backups
0 2 * * * pg_dump -U traderx traderx | gzip > /opt/traderx/backups/traderx_$(date +\%Y\%m\%d).sql.gz
```

### 2. Configuration Backup
```bash
# Backup all configurations
tar -czf /opt/traderx/backups/config_$(date +\%Y\%m\%d).tar.gz /opt/traderx/config/
```

## Troubleshooting

### 1. eBPF Issues
```bash
# Check if XDP is loaded
sudo bpftool prog show

# Check logs
sudo dmesg | grep -i xdp

# Reload if needed
sudo ip link set dev eth0 xdp off
sudo ip link set dev eth0 xdp obj ebpf-router.o sec xdp
```

### 2. KYA Verification Issues
```bash
# Check Sumsub connectivity
curl -H "X-App-Token: $SUMSUB_API_TOKEN" https://api.sumsub.com/resources/checkToken

# Verify certificates
docker-compose exec redis redis-cli get "kya:certificate:*"
```

### 3. Performance Issues
```bash
# Check system resources
htop
iotop
nethogs

# Check eBPF performance
sudo bpftool prog profile id <prog_id>
```

## Compliance Checklist

- [ ] GDPR compliance (data retention policies)
- [ ] EU AI Act Article 12 compliance (audit trails)
- [ ] KYC/AML procedures documented
- [ ] Data encryption at rest and in transit
- [ ] Access control and audit logging
- [ ] Disaster recovery plan tested
- [ ] Penetration testing completed

## Support Contacts

- Infrastructure: infra@traderx.com
- Security: security@traderx.com
- Compliance: compliance@traderx.com
- 24/7 Emergency: +1-555-TRADERX
