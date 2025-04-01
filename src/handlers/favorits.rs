use std::sync::Arc;

use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use uuid::Uuid;

use crate::AppState;

use super::auth::AuthenticatedViewer;


pub async fn get_favorite_profiles(
    State(data): State<Arc<AppState>>,
    AuthenticatedViewer { viewer_id, .. }: AuthenticatedViewer,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Query the favorites table for the current viewer
    let rows = sqlx::query!(
        r#"
        SELECT profile_id
        FROM favorites
        WHERE viewer_id = $1
        "#,
        viewer_id
    )
    .fetch_all(&data.db)
    .await
    .map_err(|e| {
        eprintln!("get_favorite_profiles error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": "Internal Server Error"
            })),
        )
    })?;

    // Build response data; each favorite is represented with a link to the profile.
    let favorites: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            json!({
                "id": row.profile_id,
                "_links": {
                    "self": format!("{}/api/profile/{}", data.url, row.profile_id)
                }
            })
        })
        .collect();

    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "data": favorites
        })),
    ))
}

pub async fn add_favorite(
    State(data): State<Arc<AppState>>,
    AuthenticatedViewer { viewer_id, .. }: AuthenticatedViewer,
    Path(profile_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Try inserting the favorite; if the combination already exists, ON CONFLICT will do nothing.
    let result = sqlx::query!(
        r#"
        INSERT INTO favorites (viewer_id, profile_id)
        VALUES ($1, $2)
        ON CONFLICT (viewer_id, profile_id) DO NOTHING
        "#,
        viewer_id,
        profile_id
    )
    .execute(&data.db)
    .await
    .map_err(|e| {
        eprintln!("add_favorite error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": "Internal Server Error"
            })),
        )
    })?;

    // If no row was inserted, the favorite already exists.
    if result.rows_affected() == 0 {
        return Ok((
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "message": "Profile is already in your favorites"
            })),
        ));
    }

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "status": "success",
            "message": "Profile added to favorites"
        })),
    ))
}

pub async fn remove_favorite(
    State(data): State<Arc<AppState>>,
    AuthenticatedViewer { viewer_id, .. }: AuthenticatedViewer,
    Path(profile_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let result = sqlx::query!(
        r#"
        DELETE FROM favorites
        WHERE viewer_id = $1 AND profile_id = $2
        "#,
        viewer_id,
        profile_id
    )
    .execute(&data.db)
    .await
    .map_err(|e| {
        eprintln!("remove_favorite error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": "Internal Server Error"
            })),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "fail",
                "message": "Favorite not found"
            })),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "message": "Favorite removed successfully"
        })),
    ))
}
