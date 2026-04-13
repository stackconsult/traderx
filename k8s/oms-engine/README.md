# OMS Engine Kubernetes Deployment

## Overview

This directory contains Kubernetes manifests for deploying the TraderX OMS Engine with production-grade security, observability, and high availability.

## Architecture

- **3 replicas** for high availability
- **Non-root containers** with security hardening
- **Read-only filesystem** with minimal capabilities
- **gVisor runtime** for additional isolation
- **PodDisruptionBudget** to ensure minimum availability
- **HorizontalPodAutoscaler** for automatic scaling
- **NetworkPolicy** for network segmentation

## Prerequisites

1. Kubernetes cluster v1.24+
2. gVisor runtime class installed
3. StorageClass `fast-ssd` for journal storage
4. PriorityClass `high-priority` for critical workloads
5. Prometheus Operator for ServiceMonitor
6. Redis deployment in `traderx` namespace
7. QuestDB deployment for market data

## Security Features

### Container Security
- `runAsNonRoot: true` with user ID 1000
- `readOnlyRootFilesystem: true`
- All capabilities dropped
- `allowPrivilegeEscalation: false`
- Seccomp profile set to RuntimeDefault

### Network Security
- Default-deny NetworkPolicy
- Explicit allowlists for required traffic
- Namespace isolation
- TLS required for Redis connections

### RBAC
- Least privilege ServiceAccount
- Role-based access to required resources only
- No cluster-wide permissions

## Deployment Steps

1. **Create namespace and prerequisites:**
```bash
kubectl apply -f namespace.yaml
kubectl apply -f priorityclass.yaml
```

2. **Deploy dependencies:**
```bash
# Deploy Redis with TLS
kubectl apply -f ../redis/
# Deploy QuestDB
kubectl apply -f ../questdb/
```

3. **Create secrets:**
```bash
kubectl create secret generic traderx-secrets \
  --from-literal=redis-password=$(openssl rand -base64 32) \
  --namespace=traderx
```

4. **Deploy OMS Engine:**
```bash
# Using kustomize
kubectl apply -k .

# Or individual manifests
kubectl apply -f serviceaccount.yaml
kubectl apply -f rbac.yaml
kubectl apply -f configmap.yaml
kubectl apply -f pvc.yaml
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f pdb.yaml
kubectl apply -f hpa.yaml
kubectl apply -f servicemonitor.yaml
kubectl apply -f networkpolicy.yaml
```

## Configuration

### Environment Variables
- `RUST_LOG`: Logging level (default: info)
- `OMS_CONFIG_PATH`: Path to configuration file
- `REDIS_PASSWORD`: Redis authentication password
- `RISK_BUS_CONFIG`: Risk bus configuration JSON

### Resource Limits
- CPU: 100m-500m per replica
- Memory: 256Mi-512Mi per replica
- Storage: 10Gi for journal, 5Gi for cache

### Scaling
- Minimum replicas: 3
- Maximum replicas: 10
- Target CPU utilization: 70%
- Target memory utilization: 80%
- Custom metric: orders_per_second

## Monitoring

### Metrics
- Exposed on port 9090 at `/metrics`
- Prometheus scraping every 15 seconds
- Custom metrics for orders, risk checks, latency

### Health Checks
- Liveness: `/health/live` (30s initial delay)
- Readiness: `/health/ready` (5s initial delay)
- Startup: `/health/live` (5min grace period)

### Alerts
Recommended alerts:
- High order rejection rate
- Risk bus halted
- Memory/CPU usage > 80%
- Pod restarts
- Health check failures

## Troubleshooting

### Common Issues

1. **Pod stuck in CrashLoopBackOff**
   - Check logs: `kubectl logs -n traderx deployment/oms-engine`
   - Verify configuration: `kubectl get configmap oms-config -o yaml`
   - Check resource limits

2. **Health checks failing**
   - Verify observability server is running
   - Check Risk Bus status: `curl http://service:9090/health/detailed`
   - Review component dependencies

3. **High memory usage**
   - Check journal size: `kubectl exec -it pod -- du -sh /data/journal`
   - Review WAL rotation settings
   - Monitor with: `kubectl top pods -n traderx`

### Debug Commands
```bash
# Check pod status
kubectl get pods -n traderx -l app=oms-engine

# View logs
kubectl logs -n traderx -l app=oms-engine --tail=100

# Exec into pod
kubectl exec -it -n traderx deployment/oms-engine -- /bin/bash

# Check metrics
kubectl port-forward -n traderx svc/oms-engine 9090:9090
curl http://localhost:9090/metrics

# Check HPA status
kubectl get hpa -n traderx oms-engine-hpa

# Describe deployment
kubectl describe deployment -n traderx oms-engine
```

## Maintenance

### Rolling Updates
- Zero-downtime updates with maxUnavailable: 0
- Progressive deployment with health checks
- Automatic rollback on failures

### Backup
- Journal data persisted in PVC
- Regular PVC snapshots recommended
- Configuration versioned in Git

### Scaling
- Manual scaling: `kubectl scale deployment oms-engine --replicas=5`
- HPA handles automatic scaling
- Consider resource quotas for multi-tenant clusters

## Performance Tuning

### Kernel Parameters
Applied via securityContext:
- `net.core.somaxconn=65535`
- `net.ipv4.tcp_tw_reuse=1`

### JVM/Rust Tuning
- Adjust worker threads based on CPU cores
- Optimize disruptor buffer size
- Tune journal sync interval

### Storage
- Use fast SSD for journal storage
- Consider local SSD nodes for low latency
- Monitor I/O metrics

## Security Considerations

1. **Secrets Management**
   - Use Kubernetes secrets
   - Rotate Redis passwords regularly
   - Consider external secret store

2. **Network Isolation**
   - NetworkPolicy enforced
   - TLS for all inter-service communication
   - No direct internet access

3. **Runtime Security**
   - gVisor runtime for attack surface reduction
   - Regular image scanning
   - Minimal base images

## Compliance

- SOC2 Type II ready
- PCI DSS considerations for trading
- Audit logging enabled
- Immutable infrastructure
