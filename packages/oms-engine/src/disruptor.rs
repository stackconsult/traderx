use crate::oms::OmsEvent;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tracing::{info, error, warn};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DisruptorError {
    #[error("Ring buffer is full")]
    BufferFull,
    #[error("Processor not registered")]
    ProcessorNotRegistered,
    #[error("Already started")]
    AlreadyStarted,
}

/// Event processor trait for disruptor events
#[async_trait::async_trait]
pub trait EventProcessor: Send + Sync {
    type Event;
    type Error;
    
    async fn process(&self, event: Self::Event) -> Result<(), Self::Error>;
}

/// LMAX-style Disruptor pattern implementation
/// Wraps rtrb::RingBuffer with additional sequencing and batch processing
pub struct Disruptor {
    /// Ring buffer for events
    ring_buffer: Arc<Mutex<RingBuffer<OmsEvent>>>,
    
    /// Event producers (one per thread for performance)
    producers: Vec<Arc<Mutex<Producer<OmsEvent>>>>,
    
    /// Event consumers (one per processor)
    consumers: Vec<Arc<Mutex<Consumer<OmsEvent>>>>,
    
    /// Registered processors
    processors: Arc<Mutex<Vec<Box<dyn EventProcessor<Event = OmsEvent, Error = Box<dyn std::error::Error + Send + Sync>> + Send>>>>,
    
    /// Batch size for processing
    batch_size: usize,
    
    /// Running state
    running: Arc<Mutex<bool>>,
}

impl Disruptor {
    /// Create new disruptor with specified buffer size
    pub fn new(buffer_size: usize) -> Result<Self, DisruptorError> {
        // Ensure buffer size is power of 2 for optimal performance
        let size = if buffer_size.is_power_of_two() {
            buffer_size
        } else {
            buffer_size.next_power_of_two()
        };
        
        let ring_buffer = RingBuffer::new(size);
        let (producer, consumer) = ring_buffer.split();
        
        info!("Created disruptor with ring buffer size: {}", size);
        
        Ok(Self {
            ring_buffer: Arc::new(Mutex::new(ring_buffer)),
            producers: vec![Arc::new(Mutex::new(producer))],
            consumers: vec![Arc::new(Mutex::new(consumer))],
            processors: Arc::new(Mutex::new(Vec::new())),
            batch_size: 32, // Optimal batch size for most workloads
            running: Arc::new(Mutex::new(false)),
        })
    }
    
    /// Publish event to the disruptor
    pub async fn publish(&self, event: OmsEvent) -> Result<(), DisruptorError> {
        // Try to publish to first producer
        let producer = self.producers[0].lock().await;
        
        match producer.try_push(event) {
            Ok(_) => Ok(()),
            Err(rtrb::PushError::Full(_)) => {
                // Buffer full - log warning but don't block
                warn!("Disruptor buffer full, dropping event");
                Err(DisruptorError::BufferFull)
            }
        }
    }
    
    /// Register event processor
    pub async fn register_processor<P>(&self, processor: P) -> Result<(), DisruptorError>
    where
        P: EventProcessor<Event = OmsEvent, Error = Box<dyn std::error::Error + Send + Sync>> + Send + 'static,
    {
        let mut processors = self.processors.lock().await;
        processors.push(Box::new(processor));
        
        info!("Registered event processor, total: {}", processors.len());
        Ok(())
    }
    
    /// Start processing events
    pub async fn start(&self) -> Result<(), DisruptorError> {
        let mut running = self.running.lock().await;
        if *running {
            return Err(DisruptorError::AlreadyStarted);
        }
        
        *running = true;
        
        // Spawn processing task for each consumer
        for (i, consumer) in self.consumers.iter().enumerate() {
            let consumer = consumer.clone();
            let processors = self.processors.clone();
            let batch_size = self.batch_size;
            
            tokio::spawn(async move {
                Self::processing_loop(i, consumer, processors, batch_size).await;
            });
        }
        
        info!("Started disruptor event processing");
        Ok(())
    }
    
    /// Stop processing events
    pub async fn stop(&self) {
        let mut running = self.running.lock().await;
        *running = false;
        info!("Stopped disruptor event processing");
    }
    
    /// Get current buffer capacity
    pub async fn capacity(&self) -> usize {
        self.ring_buffer.lock().await.capacity()
    }
    
    /// Get current buffer usage
    pub async fn usage(&self) -> usize {
        let ring_buffer = self.ring_buffer.lock().await;
        ring_buffer.capacity() - ring_buffer.slots().len()
    }
    
    /// Processing loop for consumer
    async fn processing_loop(
        id: usize,
        consumer: Arc<Mutex<Consumer<OmsEvent>>>,
        processors: Arc<Mutex<Vec<Box<dyn EventProcessor<Event = OmsEvent, Error = Box<dyn std::error::Error + Send + Sync>> + Send>>>>,
        batch_size: usize,
    ) {
        info!("Starting processing loop for consumer {}", id);
        
        let mut batch = Vec::with_capacity(batch_size);
        
        loop {
            // Collect batch of events
            {
                let mut cons = consumer.lock().await;
                
                // Try to fill batch
                for _ in 0..batch_size {
                    match cons.try_pop() {
                        Some(event) => batch.push(event),
                        None => break,
                    }
                }
            }
            
            // Process batch if not empty
            if !batch.is_empty() {
                let procs = processors.lock().await;
                
                for event in batch.drain(..) {
                    for processor in procs.iter() {
                        if let Err(e) = processor.process(event.clone()).await {
                            error!("Processor error: {}", e);
                        }
                    }
                }
            } else {
                // No events, brief sleep
                tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
            }
        }
    }
}

/// Sequence tracker for ordering events
#[derive(Debug)]
pub struct Sequence {
    value: u64,
}

impl Sequence {
    pub fn new() -> Self {
        Self { value: 0 }
    }
    
    pub fn next(&mut self) -> u64 {
        self.value += 1;
        self.value
    }
    
    pub fn current(&self) -> u64 {
        self.value
    }
}

/// Barrier for coordinating processors
pub struct Barrier {
    sequences: Arc<Mutex<Vec<u64>>>,
    processor_count: usize,
}

impl Barrier {
    pub fn new(processor_count: usize) -> Self {
        Self {
            sequences: Arc::new(Mutex::new(vec![0; processor_count])),
            processor_count,
        }
    }
    
    /// Get the minimum sequence across all processors
    pub async fn get_minimum_sequence(&self) -> u64 {
        let sequences = self.sequences.lock().await;
        *sequences.iter().min().unwrap_or(&0)
    }
    
    /// Update sequence for a processor
    pub async fn update_sequence(&self, processor_id: usize, sequence: u64) {
        let mut sequences = self.sequences.lock().await;
        if processor_id < sequences.len() {
            sequences[processor_id] = sequence;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oms::OmsEvent;
    
    struct TestProcessor;
    
    #[async_trait::async_trait]
    impl EventProcessor for TestProcessor {
        type Event = OmsEvent;
        type Error = Box<dyn std::error::Error + Send + Sync>;
        
        async fn process(&self, event: Self::Event) -> Result<(), Self::Error> {
            println!("Processed event: {:?}", event);
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_disruptor_publish_and_process() {
        let disruptor = Disruptor::new(1024).unwrap();
        
        // Register processor
        disruptor.register_processor(TestProcessor).await.unwrap();
        
        // Start processing
        disruptor.start().await.unwrap();
        
        // Publish test event
        let event = OmsEvent::OrderRejected {
            order_id: uuid::Uuid::new_v4(),
            reason: "Test".to_string(),
        };
        
        assert!(disruptor.publish(event).await.is_ok());
        
        // Give time for processing
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        disruptor.stop().await;
    }
}
