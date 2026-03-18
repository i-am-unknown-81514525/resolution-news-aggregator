mod plugins;
mod unify;
mod value_enum;

use axum::{
    Router,
    body::Bytes,
    extract::ws::{Message, WebSocketUpgrade},
    response::IntoResponse,
    routing::any,
};
use axum_extra::TypedHeader;
use tokio;

use crate::plugins::source::{RSSSource, RSSSourceType, remap};
use crate::unify::{ToVecUnify, UnifyOutputRaw};
use crate::value_enum::EnumFromStr;
use axum::extract::{ConnectInfo, State};
use plugins::net::rss_fetch::get_raw;
use std::sync::Arc;
use std::time::Duration;
use std::net::SocketAddr;
use tokio::sync::Mutex;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct ServerState {
    // conns: Arc<Mutex<Vec<Arc<Mutex<WebSocket>>>>>,
    receiver: tokio::sync::broadcast::Receiver<UnifyOutputRaw>,
}

impl ServerState {
    pub fn new(receiver: tokio::sync::broadcast::Receiver<UnifyOutputRaw>) -> Self {
        Self { receiver }
    }
}

pub async fn background_fetching(sender: tokio::sync::broadcast::Sender<UnifyOutputRaw>) -> () {
    // testing currently - should be reading config file in future instead
    loop {
        let kind = RSSSourceType::enum_str("GoogleRssSearch").unwrap();
        let source = remap(kind);
        let query =
            "(oil price OR OPEC OR \"natural gas\" OR \"crude oil\" OR WTI OR Brent) when:1h";
        let url = source.get_url(query).unwrap();
        let content = get_raw((&url).parse().unwrap()).await.unwrap();
        let result = source.deserialize(&content).unwrap();
        let outputs = result.to_vec_unify();
        info!("Pushing {} outputs", outputs.len());
        for output in outputs {
            sender.send(output.to_raw()).unwrap();
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
    }
}

#[tokio::main]
async fn main() {
    let (sender, receiver) = tokio::sync::broadcast::channel::<UnifyOutputRaw>(1024);
    let state = Arc::new(Mutex::new(ServerState::new(receiver)));
    tokio::spawn(async move {
        background_fetching(sender).await;
    });
    let _clone = state.clone();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let app = Router::new()
        .route("/ws", any(news_ws_handler))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    let _ = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await;
}

static KEEPALIVE_BYTE: once_cell::sync::Lazy<Bytes> =
    once_cell::sync::Lazy::new(|| Bytes::from("keepalive"));

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
                                match socket.send(Message::Binary(v.data)).await {
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
        ()
    })
}
