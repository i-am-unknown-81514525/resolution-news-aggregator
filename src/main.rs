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
use std::sync::{Arc, Mutex};
use axum::extract::{ConnectInfo, State};
use axum::extract::ws::CloseFrame;
use tower_http::{
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::sync::mpsc;
use crate::unify::UnifyOutput;
use crate::plugins::source::{RSSSource, RSSSourceType, remap};
use crate::value_enum::EnumFromStr;
use plugins::net::rss_fetch::{fetch_rss, get_raw};

struct ServerState {
    conns: Vec<WebSocket>,
    sender: mpsc::UnboundedSender<UnifyOutput>,
    receiver: mpsc::UnboundedReceiver<UnifyOutput>,
}

impl ServerState {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self { conns: Vec::new() , sender, receiver }
    }

    pub async fn background_reading() -> () {

    }

    pub async fn background_fetching() -> () {
        // testing currently - should be reading config file in future instead
        let kind = RSSSourceType::enum_str("GoogleRssSearch").unwrap();
        let query = "(oil price OR OPEC OR \"natural gas\" OR \"crude oil\" OR WTI OR Brent) when:1h";
        let url = remap(kind).get_url(query).unwrap();
        let content = get_raw((&url).parse().unwrap()).await.unwrap();
        let result = remap(kind).deserialize(&content).unwrap();
        
    }
}



#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let mut state = Arc::new(Mutex::new(ServerState::new()));
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
        state.get_mut()?.conns.push(socket);
        Ok(())
    })
}