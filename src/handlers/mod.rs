use std::sync::Arc;

use axum::{response::IntoResponse, Json};
use serde_json::json;

use crate::AppState;

use axum::extract::State;

pub mod auth;
pub mod craft;
pub mod profile;
// pub mod rating;
pub mod favorits;
pub mod rechtsformen;
pub mod skill;

pub async fn health_checker_handler() -> impl IntoResponse {
    let response = json!({
        "status": "success",
        "message": "API is running"
    });
    Json(response)
}

pub async fn health_checker_handler2(State(data): State<Arc<AppState>>) -> impl IntoResponse {
    let response = json!({
        "status": "success",
        "message": format!("{}", data.url)
    });
    Json(response)
}
