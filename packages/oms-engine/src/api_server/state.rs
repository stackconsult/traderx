use crate::api_server::types::WsMessage;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct AppState {
    pub jwt_secret: String,
    pub market_feed: broadcast::Sender<WsMessage>,
}

impl AppState {
    pub fn new(jwt_secret: String) -> Arc<Self> {
        let (market_feed, _) = broadcast::channel(100);
        Arc::new(Self {
            jwt_secret,
            market_feed,
        })
    }
}
