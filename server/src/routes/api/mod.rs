use axum::routing::any;
use crate::routes::Router;

mod history;
mod latest_id;

pub fn routes() -> Router {
    Router::new()
        .nest("/history", history::routes())
        .nest("/latest_idx", latest_id::routes())
}
