#[derive(Debug, thiserror::Error)]
pub enum OrchestraError {
    #[error("Question classification failed: {0}")]
    ClassificationFailed(String),
    
    #[error("Agent routing failed: {0}")]
    RoutingFailed(String),
    
    #[error("Agent processing failed: {0}")]
    ProcessingFailed(String),
    
    #[error("Response integration failed: {0}")]
    IntegrationFailed(String),
    
    #[error("Quality validation failed: {0}")]
    QualityValidationFailed(String),
}
