use std::sync::Arc;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::Json;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::Deserialize;
use sqlx::Postgres;
use tokio::sync::Mutex;
use tracing::log::warn;
use common::unify::UnifyOutput;
use crate::routes::api::history;
use crate::routes::Router;
use crate::ServerState;

pub fn routes() -> Router {
    Router::new()
        .route("/", get(history_handler))
}


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
