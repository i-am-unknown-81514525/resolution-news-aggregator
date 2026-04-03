mod plugins;
mod value_enum;
mod config;
mod model;

use std::collections::HashMap;
use axum::{Router, body::Bytes, extract::ws::{Message, WebSocketUpgrade}, response::{IntoResponse, Response}, routing::any, Json};
use axum_extra::TypedHeader;
use indexmap::IndexMap;

use crate::plugins::source::{RSSSource, RSSSourceType, remap};
use crate::value_enum::EnumFromStr;
use axum::body::Body;
use axum::extract::ws::Utf8Bytes;
use axum::extract::{ConnectInfo, Query, State};
use common::unify::{UnifyOutput, UnifyOutputRaw};
use plugins::net::rss_fetch::get_raw;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::config::{Config};
use ahash::RandomState;
use rand::Rng;

use std::{env, thread};
use std::fmt::format;
use std::path::PathBuf;
use cfg_if::cfg_if;
use sqlx::{query, Error, Executor, PgPool, Postgres};
use tracing::log::warn;
use model::{Model, get_model};
use crate::model::{embedding_thread, Task};
use pgvector::Vector;

struct ServerState {
    // conns: Arc<Mutex<Vec<Arc<Mutex<WebSocket>>>>>,
    receiver: tokio::sync::broadcast::Receiver<UnifyOutputRaw>,
    hash_data: Arc<RwLock<HashMap<String, u64, RandomState>>>,
    sender: tokio::sync::mpsc::UnboundedSender<Task>,
    pool: PgPool,
}

impl ServerState {
    pub fn new(receiver: tokio::sync::broadcast::Receiver<UnifyOutputRaw>, sender: tokio::sync::mpsc::UnboundedSender<Task>, pool: PgPool) -> Self {
        Self {
            receiver,
            hash_data: Arc::new(RwLock::new(HashMap::default())),
            sender,
            pool
        }
    }
}


#[derive(Error, Debug)]
pub(crate) enum ApplicationError {
    #[error("Missing Rss Type {0}")]
    MissingRssType(String),
    #[error("Failed to obtain RSS URL (rss_type: {0}, query: {1}) ")]
    FailedToObtainRssUrl(String, String),
    #[error("Invalid URL {0}")]
    InvalidUrl(String),
    #[error("RssFetchError")]
    RssFetchError(#[from] plugins::source::RssFetchError),
}

pub async fn fetch_with_config(config: &Config) -> Result<Vec<UnifyOutput>, ApplicationError> {
    let kind = RSSSourceType::enum_str(&config.rss_type).map_err(
        |e| {
            tracing::warn!("Missing RSS type: {}", e);
            ApplicationError::MissingRssType(config.rss_type.clone())
        }
    )?;
    let source = remap(kind);
    let url = source.get_url(&config.query).ok_or_else(|| {
        tracing::warn!("Failed to obtain RSS URL (rss_type: {}, query: {})", config.rss_type, config.query);
        ApplicationError::FailedToObtainRssUrl(config.rss_type.clone(), config.query.clone())
    })?;
    let content = get_raw(url.parse().map_err(|e| {
        tracing::warn!("Failed to parse url: {}", e);
        ApplicationError::InvalidUrl(url.clone())

    })?).await.map_err(|e| {
        tracing::warn!("Request failed: {}", e);
        ApplicationError::RssFetchError(e)
    })?;
    source.get_unify(&content).map_err(|e| {
        tracing::warn!("Failed to deserialize: {}", e);
        ApplicationError::RssFetchError(e)
    })
}

pub async fn background_fetching(
    config: &Config,
    sender: tokio::sync::broadcast::Sender<UnifyOutputRaw>,
    state: Arc<Mutex<ServerState>>,
) -> () {
    // testing currently - should be reading config file in future instead
    let pool = state.lock().await.pool.clone();
    loop {
        let span = tracing::info_span!("background_fetching");

        span.in_scope(async || {
            let outputs = fetch_with_config(config).await;
            if let Err(err) = outputs {
                warn!("Error fetching the {}, error: {}", config, err);
                return;
            }
            let Ok(outputs) = outputs else {unreachable!()};
            info!("Pushing {} outputs from {}", outputs.len(), config);
            let mut proc_id: Vec<(i64, UnifyOutput)> = Vec::new();
            for output in outputs {
                let result = sqlx::query!("INSERT INTO
    public.unify(id, organisation, title, description, time, source, score, link, hash_key)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
ON CONFLICT DO NOTHING
RETURNING idx",
                    output.id,
                    output.organisation,
                    output.title,
                    output.description,
                    output.time.naive_utc(),
                    serde_json::to_string(&output.source).ok(),
                    output.score.map(|x| x as f64),
                    output.link,
                    output.hash_key.as_slice(),
                ).fetch_one(&pool).await;
                match result {
                    Ok(record) => {
                        proc_id.push((record.idx, output));
                    },
                    Err(Error::RowNotFound) => {}
                    Err(err) => {
                        if err.to_string().contains(&" One of the strings in hash_key is already assigned to another record") {
                            continue;
                        }
                        warn!("Database failure on creating record: {}", err);
                    }
                }
            }
            if proc_id.len() == 0 {
                return;
            }
            let task = Task::create(proc_id.iter().map(|x| x.1.clone()).clone().collect());
            state.lock().await.sender.send(task.0).unwrap();
            let results = match task.1.await {
                Ok(v) => v,
                Err(e) => vec![None; proc_id.len()]
            };
            for (i, result) in results.iter().enumerate() {
                let pair = &proc_id[i];
                if let Some(embedding) = result {
                    let query = sqlx::query::<Postgres>(
                        "UPDATE public.unify
SET embedding = $2
WHERE idx = $1")
                        .bind(pair.0)
                        .bind(Vector::from(embedding.clone()))
                        .execute(&pool).await;
                    if let Err(err) = query {
                        warn!("Database failure on updating embedding: {}", err);
                    }
                }
            }
            let ids = proc_id.iter().map(|x| x.0).collect::<Vec<i64>>();
            let multi = match sqlx::query_as::<Postgres, UnifyOutput>(
                "SELECT * FROM public.unify WHERE idx = ANY($1)"
            )
                .bind(ids.as_slice()) // You use .bind() for runtime queries
                .fetch_all(&pool)
                .await {
                Ok(r) => r,
                Err(e) => {
                    warn!("Fail to read from db: {}", e);
                    return;
                }
            };
            for item in multi {
                let recv_count = sender.send(item.to_raw()).unwrap_or(0);
                info!("Pushed document {} to {} receivers", item.id, recv_count);
            }
        }).await;
        tokio::time::sleep(Duration::from_secs(config.update_interval as u64)).await;
    }
}

#[tokio::main]
async fn main() {
    let postgres_username = env::var("POSTGRES_USER").unwrap_or("postgres".to_string());
    let postgres_password = env::var("POSTGRES_PASSWORD").unwrap_or("please-change-7a9ebb7fc05ac78b8cb04bf8".to_string());
    let url = env::var("DATABASE_URL").unwrap_or("database:5432".to_string());
    let pool: PgPool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&format!("postgresql://{}:{}@{}", postgres_username, postgres_password, url))
        .await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let (sender, receiver) = tokio::sync::broadcast::channel::<UnifyOutputRaw>(1024);
    let (model_sender, model_receiver) = tokio::sync::mpsc::unbounded_channel::<Task>();
    let state = Arc::new(Mutex::new(ServerState::new(receiver, model_sender, pool.clone())));


    let model = get_model();

    thread::spawn(move || { embedding_thread(model, model_receiver); });

    let config_str = String::from_utf8(std::fs::read("config.toml").unwrap()).unwrap();
    let configs: crate::config::Configs = toml::from_str(&config_str).unwrap();

    for config in configs.configs {
        let _clone = state.clone();
        let sender_clone = sender.clone();
        let conf_clone = config.clone();

        tokio::spawn(async move {
            background_fetching(&conf_clone, sender_clone, _clone).await;
        });
    }

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();


    let _clone = state.clone();
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

fn default_page() -> u64 {
    0
}
fn default_size() -> u64 {
    1000
}

#[axum::debug_handler]
async fn history_handler(
    State(state): State<Arc<Mutex<ServerState>>>,
    Query(query): Query<Pagination>,
) -> Response<Body> {
    if query.size > 1000 || query.size == 0 {
        return Response::builder()
            .status(400)
            .body(Body::from(format!(
                "Query size must be less than or equal to 1000 and above 0, received {}",
                query.size
            )))
            .unwrap();
    }
    if (query.page + 1) * query.size > (usize::MAX as u64)
        || (query.page) * query.size > (usize::MAX as u64)
    {
        return Response::builder()
            .status(400)
            .body(Body::from("Overflow protection"))
            .unwrap();
    }
    let pool = state.lock().await.pool.clone();
    let result: Vec<UnifyOutput> = match sqlx::query_as::<Postgres, UnifyOutput>(
        "SELECT * FROM public.unify ORDER BY idx DESC LIMIT $1 OFFSET $2"
    )
        .bind(query.size as i64)
        .bind((query.page*query.size) as i64)
        .fetch_all(&pool)
        .await {
        Ok(r) => r,
        Err(e) => {
            warn!("Fail to read from db: {}", e);
            return Response::builder()
                .status(500)
                .body(Body::from("Fail to read from database"))
                .unwrap();
        }
    };
    Json(result).into_response()
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
