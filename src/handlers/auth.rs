use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::AppState;

pub async fn pre_register(
    email: &str,
    password: &str
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    Ok(())
}