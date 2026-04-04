mod api;
mod ws;

use std::sync::Arc;
use tokio::sync::{Mutex};
use axum::Router as R;
use crate::ServerState;

type Router = R<Arc<Mutex<ServerState>>>;

pub fn routes() -> Router {
    Router::new()
        .nest("/ws", ws::routes())
        .nest("/api", api::routes())
}