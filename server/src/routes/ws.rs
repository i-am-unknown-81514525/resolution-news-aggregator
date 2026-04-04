use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use axum::body::Bytes;
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::extract::ws::{Message, Utf8Bytes};
use axum::response::IntoResponse;
use axum_extra::TypedHeader;
use tokio::sync::Mutex;
use crate::ServerState;

static KEEPALIVE_BYTE: once_cell::sync::Lazy<Bytes> =
    once_cell::sync::Lazy::new(|| Bytes::from("keepalive"));

use crate::routes::Router;
use axum::routing::any;

pub fn routes() -> Router {
    Router::new()
        .route("/", any(news_ws_handler))
}

async fn news_ws_handler(
    ws: WebSocketUpgrade,
    _user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<Mutex<ServerState>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |mut socket| async move {
        let state_clone = state.clone();
        let _ = tokio::spawn(async move {
            let mut rx_backend = state_clone.lock().await.receiver.resubscribe();
            loop {
                tokio::select! {
                    backend = rx_backend.recv() => {
                        match backend {
                            Ok(v) => {
                                match socket.send(Message::Text(Utf8Bytes::from(v.data))).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        tracing::warn!("Unhandled websocket disconnection: {}", e);
                                        break;
                                    }
                                }
                            },
                            Err(e) => {
                                tracing::warn!("Websocket disconnection as backend worker disconnected: {}", e);
                                break;
                            }
                        }
                    }
                    sock_ret = socket.recv() => {
                        match sock_ret {
                            Some(Ok(Message::Close(c)))=> {
                                tracing::debug!("Websocket disconnection - received closing frame: {:?}", c);
                                break;
                            },
                            Some(Ok(_)) => {},
                            Some(Err(e)) => {
                                tracing::warn!("Unexpected websocket disconnection: {}", e);
                                break;
                            },
                            None => {
                                tracing::debug!("Websocket disconnection gracefully");
                            }
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_secs(5)) => {
                        match socket.send(Message::Ping(KEEPALIVE_BYTE.clone())).await {
                            Ok(_) => {},
                            Err(e) => {
                                tracing::warn!("Unhandled websocket disconnection: {}", e);
                                break;
                            }
                        }
                    }
                    else => {
                        tracing::warn!("Unhandled websocket disconnection");
                        break;
                    }
                }
            }
            let _ = socket.send(Message::Close(None)).await; // Attempt graceful close and expect failed
        }).await;

    })
}
