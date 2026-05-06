# Kubernetes Orchestration Skill

---

name: kubernetes-orchestration
description: Implements K8s deployment manifests and Helm charts for TraderX components. Use when deploying services to Kubernetes or when containerizing applications.

---

## When to Activate

Use when:
- Creating K8s deployment manifests
- Writing Helm charts for services
- Configuring K8s resources (Deployments, Services, ConfigMaps, Secrets)
- Setting up rolling updates and blue-green deployments
- Configuring resource limits and requests

## Core Principles

### Declarative Configuration
K8s uses declarative configuration - describe desired state, K8s makes it happen.

### Resource Management
- **Requests**: Guaranteed resources
- **Limits**: Maximum resources (CPU throttling, OOM killing)
- **QoS**: Quality of Service class (Guaranteed, Burstable, BestEffort)

### Rolling Updates
Deploy new versions without downtime:
1. Deploy new pods
2. Wait for new pods to be healthy
3. Terminate old pods one by one
4. Verify system remains operational

### TraderX-Specific Requirements

### High Availability
- Multiple replicas for critical services
- Pod anti-affinity (spread across nodes)
- Readiness and liveness probes
- Graceful shutdown (drain connections)

### Performance
- Resource limits tuned for latency targets
- CPU pinning for latency-critical pods
- Hugepages for zero-copy networking
- Network policies for isolation

## Implementation Checklist

- [ ] Define K8s Deployment resource
- [ ] Define K8s Service resource
- [ ] Configure resource limits and requests
- [ ] Add liveness and readiness probes
- [ ] Configure ConfigMaps and Secrets
- [ ] Set up rolling update strategy
- [ ] Add pod anti-affinity rules
- [ ] Test deployment in dev environment

## Common Pitfalls

- ❌ No resource limits → pod can consume all node resources
- ❌ No probes → K8s can't detect unhealthy pods
- ❌ No graceful shutdown → connections dropped abruptly
- ❌ No anti-affinity → all pods on same node (single point of failure)

## TraderX-Specific Adaptations

### OMS Engine Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: oms-engine
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: oms-engine
  template:
    metadata:
      labels:
        app: oms-engine
    spec:
      affinity:
        podAntiAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
          - labelSelector:
              matchLabels:
                app: oms-engine
            topologyKey: kubernetes.io/hostname
      containers:
      - name: oms-engine
        image: traderx/oms-engine:latest
        resources:
          requests:
            cpu: "2"
            memory: "4Gi"
          limits:
            cpu: "4"
            memory: "8Gi"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
```

### Risk Bus Deployment (Latency-Critical)
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: risk-bus
spec:
  replicas: 2
  template:
    spec:
      # CPU pinning for latency
      containers:
      - name: risk-bus
        image: traderx/risk-bus:latest
        resources:
          limits:
            cpu: "4"  # Use all 4 cores
            memory: "8Gi"
        env:
        - name: RUST_LOG
          value: "error"  # Minimal logging for latency
```

## Verification

After creating K8s manifests:
- [ ] Manifests are syntactically valid (kubectl apply --dry-run)
- [ ] Resource limits are appropriate (not too high/low)
- [ ] Probes correctly detect pod health
- [ ] Rolling update strategy configured
- [ ] Deployment tested in dev environment
- [ ] No single point of failure
