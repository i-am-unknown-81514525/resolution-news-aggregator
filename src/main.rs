use futures_util::{SinkExt, StreamExt};
mod unify;
mod value_enum;
mod plugins;

use tokio;
use axum::{
    body::Bytes,
    extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::any,
    Router
};
use axum_extra::TypedHeader;

use std::ops::ControlFlow;
use std::{net::SocketAddr, path::PathBuf};
use std::sync::{Arc};
use tokio::sync::Mutex;
use axum::extract::{ConnectInfo, State};
use axum::extract::ws::CloseFrame;
use tower_http::{
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::sync::mpsc;
use crate::unify::{ToVecUnify, UnifyOutput};
use crate::plugins::source::{RSSSource, RSSSourceType, remap};
use crate::value_enum::EnumFromStr;
use plugins::net::rss_fetch::{fetch_rss, get_raw};

struct ServerState {
    conns: Vec<WebSocket>,
}

impl ServerState {
    pub fn new() -> Self {

        Self { conns: Vec::new()}
    }
}

pub async fn background_reading(state: Arc<Mutex<ServerState>>, receiver: &mut mpsc::UnboundedReceiver<UnifyOutput>) -> () {
    loop {
        let item = receiver.recv().await;
        if item.is_none() {
            panic!("Lost connection to fetch backend");
        }
        if let Some(i) = item {
            let content = serde_json::to_string(&i);
            if let Ok(content) = content {
                for socket in state.lock().await.conns.iter_mut() {
                    let r = socket.send(Message::Text(Utf8Bytes::from(&content))).await;
                    if r.is_err() {
                        dbg!(r.unwrap_err());
                    }
                }
            }
        }
    }
}

pub async fn background_fetching(sender: mpsc::UnboundedSender<UnifyOutput>) -> () {
    // testing currently - should be reading config file in future instead
    loop {
        let kind = RSSSourceType::enum_str("GoogleRssSearch").unwrap();
        let source = remap(kind);
        let query = "(oil price OR OPEC OR \"natural gas\" OR \"crude oil\" OR WTI OR Brent) when:1h";
        let url = source.get_url(query).unwrap();
        let content = get_raw((&url).parse().unwrap()).await.unwrap();
        let result = source.deserialize(&content).unwrap();
        let outputs = result.to_vec_unify();
        for output in outputs {
            sender.send(output).unwrap();
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
    }
}


#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let mut state = Arc::new(Mutex::new(ServerState::new()));
    let (sender, mut receiver) = mpsc::unbounded_channel::<UnifyOutput>();
    tokio::spawn(async move {
        background_fetching(sender).await;
    });
    let clone = state.clone();
    tokio::spawn(async move {
        background_reading(clone, &mut receiver).await;
    });
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
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    let _ = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
        .await;
}

async fn news_ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(mut state): State<Arc<Mutex<ServerState>>>
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        // let (mut sender, mut receiver) = socket.split();
        // sender.send(Message::Close(Some(CloseFrame {
        //     code: axum::extract::ws::close_code::NORMAL,
        //     reason: Default::default(),
        // }))).await.ok();
        let mut state = state.lock().await;
        state.conns.push(socket);
        ()
    })
}