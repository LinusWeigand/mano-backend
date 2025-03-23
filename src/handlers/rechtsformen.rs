use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

use crate::{
    model::{RechtsformModel, ExplainRechtsformModel},
    schema::{CreateRechtsformSchema, UpdateRechtsformSchema},
    AppState,
};

use super::auth::AuthenticatedViewer;

pub async fn get_rechtsformen(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query = sqlx::query_as!(RechtsformModel, "SELECT name FROM rechtsformen")
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
pub async fn get_explain_rechtsformen(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query = sqlx::query_as!(ExplainRechtsformModel, "SELECT explain_name FROM rechtsformen")
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

pub async fn create_rechtsform(
    State(data): State<Arc<AppState>>,
    AuthenticatedViewer {
        viewer_id: _viewer_id,
        is_admin,
    }: AuthenticatedViewer,
    Json(body): Json<CreateRechtsformSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !is_admin {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "status": "fail",
                "message": "Only admins can accept profiles."
            })),
        ));
    }
    let new_rechtsform = sqlx::query!(
        r#"
        INSERT INTO rechtsformen (name) 
        VALUES ($1)
        RETURNING id, name;
        "#,
        body.name
    )
    .fetch_one(&data.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate key") {
            (
                StatusCode::CONFLICT,
                Json(json!({
                    "status": "fail",
                    "message": "Rechtsform already exists"
                })),
            )
        } else {
            eprintln!("Error creating rechtsform: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": "Internal Server Error"
                })),
            )
        }
    })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "status": "success",
            "data": {
                "id": new_rechtsform.id,
                "name": new_rechtsform.name
            }
        })),
    ))
}

pub async fn update_rechtsform(
    State(data): State<Arc<AppState>>,
    AuthenticatedViewer {
        viewer_id: _viewer_id,
        is_admin,
    }: AuthenticatedViewer,
    Json(body): Json<UpdateRechtsformSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !is_admin {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "status": "fail",
                "message": "Only admins can accept profiles."
            })),
        ));
    }
    let updated_rechtsform = sqlx::query!(
        r#"
        UPDATE rechtsformen SET updated_at = NOW(), name = $1
        WHERE name = $2
        RETURNING id, name;
        "#,
        body.new_name,
        body.old_name
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|e| {
        eprintln!("Error updating rechtsform: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "message": "Internal Server Error" })),
        )
    })?;

    if let Some(rechtsform) = updated_rechtsform {
        Ok((
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "data": { "id": rechtsform.id, "name": rechtsform.name }
            })),
        ))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(
                json!({ "status": "fail", "message": "Rechtsform with provided old name not found" }),
            ),
        ))
    }
}
