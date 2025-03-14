use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

use crate::{
    model::CraftModel,
    schema::{CreateCraftSchema, UpdateCraftSchema},
    AppState,
};

use super::auth::AuthenticatedViewer;

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

pub async fn create_craft(
    State(data): State<Arc<AppState>>,
    AuthenticatedViewer { viewer_id: _viewer_id, is_admin }: AuthenticatedViewer,
    Json(body): Json<CreateCraftSchema>,
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
    let new_craft = sqlx::query!(
        r#"
        INSERT INTO crafts (name) 
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
                    "message": "Craft already exists"
                })),
            )
        } else {
            eprintln!("Error creating craft: {:?}", e);
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
                "id": new_craft.id,
                "name": new_craft.name
            }
        })),
    ))
}

pub async fn update_craft(
    State(data): State<Arc<AppState>>,
    AuthenticatedViewer { viewer_id: _viewer_id, is_admin }: AuthenticatedViewer,
    Json(body): Json<UpdateCraftSchema>,
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
    let updated_craft = sqlx::query!(
        r#"
        UPDATE crafts SET updated_at = NOW(), name = $1
        WHERE name = $2
        RETURNING id, name;
        "#,
        body.new_name,
        body.old_name
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|e| {
        eprintln!("Error updating craft: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "message": "Internal Server Error" })),
        )
    })?;

    if let Some(craft) = updated_craft {
        Ok((
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "data": { "id": craft.id, "name": craft.name }
            })),
        ))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "status": "fail", "message": "Craft with provided old name not found" })),
        ))
    }
}
