---
trigger: manual
description: "Integrate and process alternative data sources"
---

# Quant Alternative Data Agent Rules

## Data Ingestion Architecture

### Multi-Source Pipeline
```python
# Self-learning data ingestion with turboquant compression
class AlternativeDataIngester:
    def __init__(self):
        self.sources = {
            'news': NewsSource(),
            'social': SocialSource(),
            'satellite': SatelliteSource(),
            'supply_chain': SupplyChainSource()
        }
        self.compressor = TurboQuantCompressor()
        self.quality_monitor = DataQualityMonitor()
    
    async def ingest_all(self):
        tasks = []
        for source_name, source in self.sources.items():
            task = self.ingest_source(source_name, source)
            tasks.append(task)
        
        # Parallel ingestion with turboquant compression
        results = await asyncio.gather(*tasks)
        compressed_data = self.compressor.compress_batch(results)
        return compressed_data
```

### Self-Healing Data Pipelines
- Automatic retry with exponential backoff
- Source failover configuration
- Data quality validation and cleansing
- Schema evolution handling

## News Sentiment Processing

### Real-Time News Analysis
```rust
// Rust-based news processing with NLP acceleration
pub struct NewsProcessor {
    pub sentiment_model: SentimentModel,
    pub entity_extractor: EntityExtractor,
    pub turboquant_encoder: TurboQuantEncoder,
}

impl NewsProcessor {
    pub async fn process_news_stream(&self) -> Stream<SentimentData> {
        self.news_stream
            .map(|article| {
                let sentiment = self.sentiment_model.predict(&article.content);
                let entities = self.entity_extractor.extract(&article.content);
                let encoded = self.turboquant_encoder.encode(&sentiment, &entities);
                
                SentimentData::new(article.symbol, sentiment, entities, encoded)
            })
            .filter(|data| data.confidence > 0.8)
    }
}
```

### Feature Engineering
- Sentiment momentum indicators
- News volume anomaly detection
- Entity co-occurrence networks
- Topic modeling with dynamic topics

## Social Media Integration

### Multi-Platform Monitoring
```python
# Unified social media processing
class SocialMediaProcessor:
    def __init__(self):
        self.platforms = {
            'twitter': TwitterConnector(),
            'reddit': RedditConnector(),
            'stocktwits': StockTwitsConnector()
        }
        self.turboquant_processor = TurboQuantProcessor()
        self.influence_calculator = InfluenceCalculator()
    
    def process_social_signals(self, symbol):
        all_signals = []
        for platform, connector in self.platforms.items():
            signals = connector.get_mentions(symbol)
            weighted_signals = self.influence_calculator.weight_signals(signals)
            all_signals.extend(weighted_signals)
        
        # Turboquant-accelerated signal aggregation
        aggregated = self.turboquant_processor.aggregate_signals(all_signals)
        return aggregated
```

### Influence Weighting
- Follower count-based weighting
- Engagement rate normalization
- Bot detection and filtering
- Historical influence tracking

## Satellite Data Processing

### Economic Activity Monitoring
```yaml
satellite_config:
  providers:
    - name: "planet_labs"
      api_key: "${PLANET_API_KEY}"
      resolution: "3m"
      coverage: "global"
    - name: "orbital_insight"
      api_key: "${ORBITAL_API_KEY}"
      features: ["vehicle_count", "construction", "night_lights"]
  
  processing:
    - cloud_detection
    - image_registration
    - change_detection
    - activity_index_calculation
  
  turboquant_features:
    - neural_compression_for_imagery
    - gpu_accelerated_processing
    - parallel_tile_processing
```

### Feature Extraction
- Vehicle count analysis
- Construction activity monitoring
- Night light intensity tracking
- Supply chain movement detection

## Self-Learning Data Quality

### Automated Quality Assessment
```python
class DataQualityAssessor:
    def __init__(self):
        self.quality_models = {
            'completeness': CompletenessModel(),
            'accuracy': AccuracyModel(),
            'timeliness': TimelinessModel(),
            'consistency': ConsistencyModel()
        }
        self.turboquant_validator = TurboQuantValidator()
    
    def assess_data_quality(self, data):
        quality_scores = {}
        for metric, model in self.quality_models.items():
            score = model.score(data)
            quality_scores[metric] = score
        
        # Turboquant-accelerated validation
        overall_quality = self.turboquant_validator.aggregate_scores(quality_scores)
        
        if overall_quality < 0.7:
            self.trigger_data_healing(data)
        
        return overall_quality
```

### Data Healing Procedures
- Missing value imputation using generative models
- Outlier detection and correction
- Temporal consistency checks
- Cross-source validation

## TurboQuant Data Compression

### Neural Compression
```bash
# Configure turboquant for alternative data
turboquant-compress \
  --input raw_alternative_data.parquet \
  --output compressed_data.tq \
  --model neural_compressor \
  --compression_ratio 0.05 \
  --preserve_information 0.95 \
  --acceleration gpu
```

### Adaptive Compression
- Dynamic compression based on data importance
- Lossy compression for noisy data
- Lossless compression for key signals
- Real-time decompression optimization

## Real-Time Feature Generation

### Streaming Feature Pipeline
```python
# Real-time feature generation with state management
class RealTimeFeatureGenerator:
    def __init__(self):
        self.state_store = StateStore()
        self.turboquant_engine = TurboQuantEngine()
        self.feature_cache = FeatureCache()
    
    async def generate_features(self, data_stream):
        async for data in data_stream:
            # Update rolling state
            self.state_store.update(data)
            
            # Generate features with turboquant acceleration
            features = self.turboquant_engine.compute_features(
                data, 
                self.state_store.get_state()
            )
            
            # Cache with TTL
            self.feature_cache.set(data.symbol, features, ttl=300)
            
            yield features
```

### Feature Management
- Version-controlled feature definitions
- Automatic feature dependency resolution
- Feature lineage tracking
- Performance monitoring

## Alternative Data Fusion

### Multi-Source Fusion
```rust
// Turboquant-accelerated data fusion
pub struct DataFusionEngine {
    pub fusion_models: HashMap<String, FusionModel>,
    pub attention_weights: AttentionWeights,
    pub turboquant_fusion: TurboQuantFusion,
}

impl DataFusionEngine {
    pub fn fuse_signals(&mut self, signals: Vec<DataSignal>) -> FusedSignal {
        // Learn attention weights automatically
        self.attention_weights.update(&signals);
        
        // Fuse with turboquant acceleration
        let fused = self.turboquant_fusion.fuse(
            signals,
            &self.attention_weights
        );
        
        fused
    }
}
```

### Fusion Strategies
- Attention-based fusion
- Graph neural network fusion
- Ensemble methods
- Dynamic weighting based on predictive power

## Performance Optimization

### Processing Acceleration
```yaml
turboquant_config:
  gpu_acceleration: true
  memory_mapping: true
  parallel_processing: auto
  batch_size: adaptive
  compression_format: neural
```

### Caching Strategy
- Multi-level caching (memory, SSD, disk)
- Intelligent pre-fetching
- Cache warming based on usage patterns
- Automatic cache eviction

## Monitoring and Alerting

### Data Quality Monitoring
- Real-time quality score tracking
- Source availability monitoring
- Latency measurement
- Cost optimization alerts

### Performance Metrics
- Processing throughput (records/second)
- Feature generation latency
- Compression ratios
- Query response times
