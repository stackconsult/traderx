use axum::{
    extract::{Query, State, WebSocketUpgrade},
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::api_server::{
    auth::validate_token,
    state::AppState,
    types::{WsMessage, WsQuery},
};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(params): Query<WsQuery>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, params.token))
}

async fn handle_socket(
    mut socket: axum::extract::ws::WebSocket,
    state: Arc<AppState>,
    token: String,
) {
    if validate_token(&token, &state.jwt_secret).is_err() {
        let _ = socket
            .send(axum::extract::ws::Message::Text(
                serde_json::to_string(&WsMessage::Error {
                    message: "Invalid token".to_string(),
                })
                .unwrap(),
            ))
            .await;
        return;
    }

    let session_id = Uuid::new_v4().to_string();
    let connected_msg = serde_json::to_string(&WsMessage::Connected {
        session_id: session_id.clone(),
    })
    .unwrap();

    if socket
        .send(axum::extract::ws::Message::Text(connected_msg))
        .await
        .is_err()
    {
        return;
    }

    let mut rx = state.market_feed.subscribe();

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(msg) => {
                        let text = serde_json::to_string(&msg).unwrap();
                        if socket.send(axum::extract::ws::Message::Text(text)).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(30)) => {
                if socket.send(axum::extract::ws::Message::Ping(vec![])).await.is_err() {
                    break;
                }
            }
        }
    }
}
