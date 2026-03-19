mod plugins;
mod value_enum;

use indexmap::IndexMap;
use axum::{
    body::Bytes,
    extract::ws::{Message, WebSocketUpgrade},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use axum_extra::TypedHeader;
use tokio;

use crate::plugins::source::{remap, RSSSource, RSSSourceType};
use common::unify::{ToVecUnify, UnifyOutputRaw};
use crate::value_enum::EnumFromStr;
use axum::extract::{ConnectInfo, Query, State};
use plugins::net::rss_fetch::get_raw;
use std::sync::Arc;
use std::time::Duration;
use std::net::SocketAddr;
use axum::body::Body;
use axum::extract::ws::Utf8Bytes;
use serde::Deserialize;
use sqlx::query;
use tokio::sync::{Mutex, RwLock};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::plugins::parser::common::DocumentID;

struct ServerState {
    // conns: Arc<Mutex<Vec<Arc<Mutex<WebSocket>>>>>,
    receiver: tokio::sync::broadcast::Receiver<UnifyOutputRaw>,
    history: Arc<RwLock<IndexMap<String, Arc<UnifyOutputRaw>>>>
}

impl ServerState {
    pub fn new(receiver: tokio::sync::broadcast::Receiver<UnifyOutputRaw>) -> Self {
        Self { receiver, history: Arc::new(RwLock::new(IndexMap::with_capacity(1000))) }
    }
}

pub async fn background_fetching(sender: tokio::sync::broadcast::Sender<UnifyOutputRaw>, state: Arc<Mutex<ServerState>>) -> () {
    // testing currently - should be reading config file in future instead
    loop {
        let span = tracing::info_span!("background_fetching");
        let _guard = span.enter();
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
            let raw = output.to_raw();
            if !state.lock().await.history.clone().read().await.contains_key(&output.id) {
                let ptr = state.lock().await.history.clone();
                let mut lock = ptr.write().await;
                lock.entry(output.id.clone()).or_insert(Arc::new(raw.clone()));
            }
            let recv_count = sender.send(raw).unwrap_or(0);
            info!("Pushed document {} to {} receivers", output.id, recv_count);
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
    }
}

#[tokio::main]
async fn main() {
    let (sender, receiver) = tokio::sync::broadcast::channel::<UnifyOutputRaw>(1024);
    let state = Arc::new(Mutex::new(ServerState::new(receiver)));
    let _clone = state.clone();
    tokio::spawn(async move {
        background_fetching(sender, _clone).await;
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
        .route("/api/history", any(history_handler))
        .with_state(_clone)
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

#[derive(Deserialize)]
struct Pagination {
    #[serde(default = "default_page")]
    page: u64,
    #[serde(default = "default_size")]
    size: u64,
}

fn default_page() -> u64 { 0 }
fn default_size() -> u64 { 100 }

#[axum::debug_handler]
async fn history_handler(
    State(state): State<Arc<Mutex<ServerState>>>,
    Query(query): Query<Pagination>
) -> Response<Body> {
    if query.size > 100 || query.size == 0 {
        return Response::builder()
            .status(400)
            .body(Body::from(format!("Query size must be less than or equal to 100 and above 0, received {}", query.size)))
            .unwrap();
    }
    if (query.page + 1) * query.size > (usize::MAX as u64) || (query.page) * query.size > (usize::MAX as u64) {
        return Response::builder()
            .status(400)
            .body(Body::from("Overflow protection"))
            .unwrap();
    }
    let result: Vec<_> = {
        let state_guard = state.lock().await;
        let history_guard = state_guard.history.read().await;

        history_guard
            .iter()
            .skip((query.page * query.size) as usize)
            .take(query.size as usize)
            .map(|x| x.1.clone())
            .collect()
    };
    let mut resp = String::with_capacity(81920);
    resp.push('[');
    for (i, item) in result.iter().enumerate() {
        if i > 0 {
            resp.push(',');
        }
        resp.push_str(&item.data);
    }
    resp.push(']');
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(resp))
        .unwrap()
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
        ()
    })
}
