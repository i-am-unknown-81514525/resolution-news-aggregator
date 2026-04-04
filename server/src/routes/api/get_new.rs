use std::sync::Arc;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::Json;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::Deserialize;
use sqlx::{query, Postgres};
use tokio::sync::Mutex;
use tracing::log::warn;
use common::unify::UnifyOutput;
use crate::routes::api::history;
use crate::routes::Router;
use crate::ServerState;

pub fn routes() -> Router {
    Router::new()
        .route("/", get(get_new_handler))
}


#[derive(Deserialize)]
struct From {
    #[serde(default = "default_from")]
    from: u64,
}

fn default_from() -> u64 { 0 }

#[axum::debug_handler]
async fn get_new_handler(
    State(state): State<Arc<Mutex<ServerState>>>,
    Query(query): Query<From>,
) -> Response<Body> {
    let pool = state.lock().await.pool.clone();
    let result = sqlx::query!("SELECT idx FROM public.unify ORDER BY idx DESC LIMIT 1")
        .fetch_one(&pool).await;
    let latest = match result {
        Ok(record) => record.idx,
        Err(e) => {
            warn!("Error fetching latest records idx for unity: {}", e);
            let x = Vec::<UnifyOutput>::with_capacity(0);
            return Json(x).into_response()
        }
    };
    let from = query.from as i64;
    if from > latest {
        let x = Vec::<UnifyOutput>::with_capacity(0);
        return Json(x).into_response()
    }
    let result: Vec<UnifyOutput> = match sqlx::query_as::<Postgres, UnifyOutput>(
        "SELECT * FROM (
    SELECT * FROM public.unify
    ORDER BY idx ASC
    LIMIT 1000
) AS subquery
ORDER BY idx DESC;"
    )
        .bind(from)
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
