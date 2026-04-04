use axum::routing::any;
use crate::routes::Router;

mod history;

pub fn routes() -> Router {
    Router::new()
        .nest("/history", history::routes())
}
