use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
};
use crate::{
    handlers::crud::{
        create_viewer_handler, delete_viewer_handler, edit_viewer_handler, get_viewer_handler,
        health_checker_handler, viewer_list_handler,
    },
    AppState,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .route("/api/viewers", post(create_viewer_handler))
        .route("/api/viewers", get(viewer_list_handler))
        .route(
            "/api/viewers/:id",
            get(get_viewer_handler)
                .patch(edit_viewer_handler)
                .delete(delete_viewer_handler),
        )
        .with_state(app_state)
}
