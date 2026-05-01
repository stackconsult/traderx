use super::{AgentMessage, MiddlewareResult, MiddlewareError};
use tokio::sync::mpsc;

pub struct MessageBus {
    llm_tx: mpsc::Sender<AgentMessage>,
    llm_rx: mpsc::Receiver<AgentMessage>,
    ml_tx: mpsc::Sender<AgentMessage>,
    ml_rx: mpsc::Receiver<AgentMessage>,
    neural_tx: mpsc::Sender<AgentMessage>,
    neural_rx: mpsc::Receiver<AgentMessage>,
    capacity: usize,
}

impl MessageBus {
    pub fn new(capacity: usize) -> Self {
        let (llm_tx, llm_rx) = mpsc::channel(capacity);
        let (ml_tx, ml_rx) = mpsc::channel(capacity);
        let (neural_tx, neural_rx) = mpsc::channel(capacity);
        
        Self {
            llm_tx,
            llm_rx,
            ml_tx,
            ml_rx,
            neural_tx,
            neural_rx,
            capacity,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(1024)
    }

    pub fn get_llm_sender(&self) -> mpsc::Sender<AgentMessage> {
        self.llm_tx.clone()
    }

    pub fn get_ml_sender(&self) -> mpsc::Sender<AgentMessage> {
        self.ml_tx.clone()
    }

    pub fn get_neural_sender(&self) -> mpsc::Sender<AgentMessage> {
        self.neural_tx.clone()
    }

    pub async fn send_llm(&self, message: AgentMessage) -> MiddlewareResult<()> {
        self.llm_tx.send(message).await.map_err(|_| MiddlewareError::ChannelClosed)
    }

    pub async fn send_ml(&self, message: AgentMessage) -> MiddlewareResult<()> {
        self.ml_tx.send(message).await.map_err(|_| MiddlewareError::ChannelClosed)
    }

    pub async fn send_neural(&self, message: AgentMessage) -> MiddlewareResult<()> {
        self.neural_tx.send(message).await.map_err(|_| MiddlewareError::ChannelClosed)
    }

    pub async fn recv_llm(&mut self) -> Option<AgentMessage> {
        self.llm_rx.recv().await
    }

    pub async fn recv_ml(&mut self) -> Option<AgentMessage> {
        self.ml_rx.recv().await
    }

    pub async fn recv_neural(&mut self) -> Option<AgentMessage> {
        self.neural_rx.recv().await
    }

    pub fn try_recv_llm(&mut self) -> Result<AgentMessage, MiddlewareError> {
        self.llm_rx.try_recv().map_err(|_| MiddlewareError::Timeout)
    }

    pub fn try_recv_ml(&mut self) -> Result<AgentMessage, MiddlewareError> {
        self.ml_rx.try_recv().map_err(|_| MiddlewareError::Timeout)
    }

    pub fn try_recv_neural(&mut self) -> Result<AgentMessage, MiddlewareError> {
        self.neural_rx.try_recv().map_err(|_| MiddlewareError::Timeout)
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn clone_senders(&self) -> (mpsc::Sender<AgentMessage>, mpsc::Sender<AgentMessage>, mpsc::Sender<AgentMessage>) {
        (self.llm_tx.clone(), self.ml_tx.clone(), self.neural_tx.clone())
    }
}

pub struct MessageRouter {
    bus: MessageBus,
}

impl MessageRouter {
    pub fn new(bus: MessageBus) -> Self {
        Self { bus }
    }

    pub async fn route(&self, message: AgentMessage) -> MiddlewareResult<()> {
        match &message {
            AgentMessage::LlmRequest { .. } | AgentMessage::LlmResponse { .. } => {
                self.bus.send_llm(message).await
            }
            AgentMessage::MlRequest { .. } | AgentMessage::MlResponse { .. } => {
                self.bus.send_ml(message).await
            }
            AgentMessage::NeuralRequest { .. } | AgentMessage::NeuralResponse { .. } => {
                self.bus.send_neural(message).await
            }
            _ => {
                self.bus.send_llm(message.clone()).await.ok();
                self.bus.send_ml(message.clone()).await.ok();
                self.bus.send_neural(message).await.ok();
                Ok(())
            }
        }
    }

    pub fn get_bus(&self) -> &MessageBus {
        &self.bus
    }
}
