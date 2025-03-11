use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

use crate::{model::{SkillModel, CraftModel}, AppState};

pub async fn get_skills(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query = sqlx::query_as!(SkillModel, "SELECT name FROM skills")
        .fetch_all(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": "Internal Server Error"
                })),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(json!({
        "status": "success",
        "data": query
        })),
    ))
}

pub async fn get_crafts(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query = sqlx::query_as!(CraftModel, "SELECT name FROM crafts")
        .fetch_all(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": "Internal Server Error"
                })),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(json!({
        "status": "success",
        "data": query
        })),
    ))
}
