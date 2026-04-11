---
trigger: manual
description: "Manage MLOps and model deployment workflows"
---

# Quant MLOps Agent Rules

## Model Deployment Pipeline

### CI/CD Configuration
```yaml
# .github/workflows/model-deployment.yml
name: Model Deployment
on:
  push:
    paths: ["models/**"]
jobs:
  deploy:
    runs-on: self-hosted
    steps:
      - name: Validate Model
        run: python validate_model.py ${{ github.sha }}
      - name: Compress with TurboQuant
        run: turboquant-compress --model ${{ github.sha }} --ratio 0.25
      - name: Deploy to Production
        run: kubectl apply -f model-deployment.yaml
```

### Self-Healing Deployment
- Automatic rollback on performance degradation
- Canary deployment with traffic splitting
- A/B testing with statistical significance
- Model drift detection and retraining triggers

## Model Registry Integration

### MLflow Configuration
```python
# MLflow tracking with turboquant optimization
mlflow.set_tracking_uri("http://mlflow.traderx.internal")
mlflow.set_experiment("quant-strategies")

with mlflow.start_run() as run:
    # Log model with compression
    mlflow.log_model(
        model,
        "model",
        registered_model_name="strategy_v1",
        serialization_format="turboquant",
        compression_level=8
    )
```

### Model Versioning
- Semantic versioning for strategy models
- Automatic metadata capture
- Performance tracking across versions
- Dependency management for reproducibility

## Distributed Training

### TurboQuant Acceleration
```bash
# Enable distributed training with turboquant
turboquant-train \
  --model architecture.json \
  --data s3://traderx/training-data/ \
  --nodes 8 \
  --gpus-per-node 4 \
  --compression turboquant \
  --precision mixed16
```

### Training Orchestration
- Kubernetes-based training jobs
- Automatic resource allocation
- Fault tolerance and checkpointing
- Hyperparameter optimization with Optuna

## Inference Optimization

### Model Serving
```python
# FastAPI inference server with turboquant
from fastapi import FastAPI
from turboquant import InferenceEngine

app = FastAPI()
engine = InferenceEngine(model_path="compressed_model.tq")

@app.post("/predict")
async def predict(features: List[float]):
    # Sub-millisecond inference
    return engine.predict(features)
```

### Performance Optimization
- Model quantization to int8
- Batch inference optimization
- GPU memory pooling
- Edge deployment for low-latency

## Monitoring and Observability

### Model Performance Tracking
```yaml
# Prometheus metrics configuration
metrics:
  - name: model_inference_latency
    type: histogram
    buckets: [0.001, 0.005, 0.01, 0.05, 0.1]
  - name: model_prediction_accuracy
    type: gauge
  - name: model_drift_score
    type: gauge
```

### Alerting Rules
- Inference latency > 5ms for 1 minute
- Prediction accuracy drop > 10%
- Model drift score > 0.8
- GPU memory usage > 90%

## Automated Retraining

### Trigger Conditions
- Performance degradation > 15%
- Data drift detection > threshold
- Scheduled weekly retraining
- Manual trigger via API

### Retraining Pipeline
1. Fetch latest training data
2. Validate data quality
3. Train with hyperparameter optimization
4. Validate against holdout set
5. Deploy if performance improves

## Experiment Management

### A/B Testing Framework
```python
# Bayesian A/B testing for model comparison
class BayesianABTest:
    def __init__(self, model_a, model_b):
        self.model_a = model_a
        self.model_b = model_b
        self.posterior = None
    
    def update(self, observations):
        # Update posterior with new observations
        self.posterior = self.update_posterior(observations)
        
    def is_significant(self, threshold=0.95):
        return self.posterior.prob_better > threshold
```

### Experiment Tracking
- Automatic experiment logging
- Statistical significance testing
- Multi-armed bandit for exploration
- Automated winner selection
