use crate::api_server::types::{OrderResponse, PositionResponse, WsMessage};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

pub struct AppState {
    pub jwt_secret: String,
    pub paper_trading: bool,
    pub market_feed: broadcast::Sender<WsMessage>,
    pub orders: Mutex<Vec<OrderResponse>>,
    pub positions: Mutex<Vec<PositionResponse>>,
}

impl AppState {
    pub fn new(jwt_secret: String) -> Arc<Self> {
        let (market_feed, _) = broadcast::channel(1024);
        Arc::new(Self {
            jwt_secret,
            paper_trading: true,
            market_feed,
            orders: Mutex::new(Vec::new()),
            positions: Mutex::new(Vec::new()),
        })
    }
}
