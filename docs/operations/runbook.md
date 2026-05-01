# Operations Runbook

**Version**: 1.0
**Last Updated**: 2026-05-01

---

## Deployment

### Pre-Deployment Checklist
- [ ] All tests passing (unit, integration, performance)
- [ ] Quality guardian Grade A or B
- [ ] Production guard passed
- [ ] Database migrations reviewed
- [ ] Rollback plan documented
- [ ] Monitoring dashboards configured
- [ ] Alert rules verified
- [ ] Stakeholders notified

### Deployment Process

#### 1. Blue-Green Deployment
```bash
# Deploy to green environment
kubectl apply -f k8s/green/ -n traderx-green

# Verify green environment is healthy
kubectl wait --for=condition=ready pod -l app=traderx -n traderx-green --timeout=300s

# Run smoke tests on green
./scripts/smoke_test.sh https://green.traderx.example.com

# Switch traffic to green
kubectl patch ingress traderx-ingress -p '{"spec":{"rules":[{"host":"traderx.example.com"}]}}'

# Monitor green environment
./scripts/monitor_deployment.sh green
```

#### 2. Canary Deployment
```bash
# Deploy canary version (5% traffic)
kubectl apply -f k8s/canary/ -n traderx-canary

# Monitor canary metrics
./scripts/monitor_canary.sh

# If canary healthy, increase traffic to 25%
kubectl patch service traderx-service -p '{"spec":{"traffic":[{"revisionName":"traderx-canary","percent":25}]}}'

# Continue monitoring and gradual traffic increase
```

#### 3. Rollback Procedure
```bash
# Immediate rollback to previous version
kubectl rollout undo deployment traderx -n traderx-prod

# Or switch traffic back to blue environment
kubectl patch ingress traderx-ingress -p '{"spec":{"rules":[{"host":"traderx.example.com"}]}}'

# Verify rollback successful
./scripts/smoke_test.sh https://traderx.example.com
```

### Database Migrations
```bash
# Review migration
./scripts/review_migration.sh

# Test migration on staging
./scripts/test_migration.sh staging

# Run migration on production
./scripts/run_migration.sh production

# Verify migration success
./scripts/verify_migration.sh production
```

---

## Monitoring

### Key Metrics to Monitor

#### Business Metrics
- Trade execution rate
- P&L by strategy
- Position sizes
- Risk limit breaches
- User activity

#### Technical Metrics
- Request latency (p50, p95, p99)
- Error rate
- Throughput (requests/sec)
- Database connection pool utilization
- Cache hit rate

#### Security Metrics
- Failed authentication attempts
- Rate limit violations
- Unauthorized access attempts
- Risk limit breaches

### Monitoring Dashboards

#### Main Dashboard
- System health overview
- Real-time trade volume
- Active users
- System alerts

#### Performance Dashboard
- Latency metrics by component
- Throughput metrics
- Error rates
- Resource utilization

#### Risk Dashboard
- Current exposure
- Risk limit utilization
- Margin requirements
- Concentration risk

### Alert Rules

#### Critical Alerts (PagerDuty)
- System down (> 5 min)
- Risk limit breach
- Database connection failure
- Authentication system failure

#### Warning Alerts (Email/Slack)
- High latency (> 1s p95)
- Error rate > 1%
- Low cache hit rate (< 80%)
- Disk space < 20%

#### Info Alerts (Dashboard only)
- Successful deployment
- Scheduled maintenance
- Performance degradation

---

## Troubleshooting

### Common Issues

#### High Latency
**Symptoms**: p95 latency > 1s
**Investigation**:
1. Check system metrics (CPU, memory, disk I/O)
2. Check database query performance
3. Check network latency
4. Check cache hit rate

**Resolution**:
- Scale horizontally if resource constrained
- Optimize slow database queries
- Add caching for frequently accessed data
- Investigate network issues

#### High Error Rate
**Symptoms**: Error rate > 1%
**Investigation**:
1. Check error logs
2. Identify error patterns
3. Check external dependencies
4. Check database connectivity

**Resolution**:
- Fix identified bugs
- Restart failing services
- Investigate external dependency issues
- Check database health

#### Database Connection Pool Exhaustion
**Symptoms**: Database connection errors
**Investigation**:
1. Check connection pool utilization
2. Check for connection leaks
3. Check query performance

**Resolution**:
- Increase connection pool size
- Fix connection leaks
- Optimize slow queries
- Add read replicas

### Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| TRX-001 | Authentication failed | Verify credentials |
| TRX-002 | Risk limit exceeded | Review risk settings |
| TRX-003 | Order rejected | Check order parameters |
| TRX-004 | Database error | Check database connectivity |
| TRX-005 | External API error | Check external service |

### Log Analysis

#### Log Levels
- **ERROR**: Critical errors requiring immediate attention
- **WARNING**: Warnings that may require investigation
- **INFO**: Normal operational messages
- **DEBUG**: Detailed debugging information

#### Log Aggregation
- Centralized log aggregation (ELK stack)
- Log retention: 30 days for INFO, 90 days for ERROR
- Log search and analysis tools

#### Common Log Patterns
```bash
# Search for errors
grep "ERROR" /var/log/traderx/*.log

# Search for specific error code
grep "TRX-002" /var/log/traderx/*.log

# Search for latency issues
grep "latency.*>1000" /var/log/traderx/*.log
```

---

## Emergency Procedures

### System Outage

#### Severity Levels
- **SEV-1**: Complete system outage (all users affected)
- **SEV-2**: Partial outage (some users affected)
- **SEV-3**: Degraded performance (slow but functional)

#### SEV-1 Procedure
1. Declare incident (Slack #incidents)
2. Page on-call engineer
3. Activate war room
4. Identify root cause
5. Implement fix or rollback
6. Verify fix
7. Close incident
8. Post-mortem analysis

#### SEV-2 Procedure
1. Declare incident (Slack #incidents)
2. Notify on-call engineer
3. Investigate and fix
4. Verify fix
5. Close incident
6. Post-mortem if needed

#### SEV-3 Procedure
1. Monitor situation
2. Investigate if degradation worsens
3. Plan fix during business hours
4. Implement fix
5. Verify fix

### Security Incident

#### Immediate Actions
1. Isolate affected systems
2. Preserve evidence (logs, memory dumps)
3. Notify security team
4. Activate incident response plan
5. Communicate with stakeholders

#### Investigation Steps
1. Determine scope of breach
2. Identify affected data
3. Determine root cause
4. Implement containment measures
5. Eradicate threat
6. Recover systems
7. Document lessons learned

### Data Breach

#### Notification Requirements
- Affected users: Within 72 hours
- Regulatory bodies: Within 72 hours
- Internal stakeholders: Immediately

#### Recovery Steps
1. Secure systems
2. Reset credentials
3. Monitor for suspicious activity
4. Implement additional security measures
5. Communicate with affected parties

---

## Maintenance Windows

### Scheduled Maintenance
- **Frequency**: Monthly
- **Duration**: 2 hours
- **Notification**: 7 days in advance
- **Time**: Sunday 2:00 AM - 4:00 AM UTC

### Maintenance Activities
- System updates
- Database maintenance
- Performance tuning
- Security patches
- Capacity planning

### Maintenance Checklist
- [ ] Notify stakeholders
- [ ] Schedule maintenance window
- [ ] Backup critical data
- [ ] Perform maintenance
- [ ] Verify system health
- [ ] Notify completion

---

## Capacity Planning

### Metrics to Track
- CPU utilization
- Memory utilization
- Disk I/O
- Network throughput
- Database size
- User growth

### Scaling Triggers
- CPU > 70% for 30 minutes
- Memory > 80% for 30 minutes
- Disk space < 20%
- Throughput > 80% of capacity

### Scaling Actions
- Horizontal scaling (add pods)
- Vertical scaling (increase resources)
- Database scaling (read replicas, sharding)
- Cache scaling (add nodes)
