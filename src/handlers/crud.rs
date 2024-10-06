use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    model::ViewerModel,
    schema::{CreateViewerSchema, FilterOptions, UpdateViewerSchema},
    AppState,
};

pub async fn health_checker_handler() -> impl IntoResponse {
    let response = json!({
        "status": "success",
        "message": "API is running"
    });
    Json(response)
}

pub async fn viewer_list_handler(
    State(data): State<Arc<AppState>>,
    opts: Option<Query<FilterOptions>>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)>{
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    let query_result = sqlx::query_as!(
        ViewerModel,
        "SELECT * FROM viewers ORDER by id LIMIT $1 OFFSET $2",
        limit as i32,
        offset as i32
    )
    .fetch_all(&data.db)
    .await;

    if query_result.is_err() {
        let error_response = json!({
            "status": "fail",
            "message": "something went wrong while fetching all viewer items"

        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let viewers = query_result.unwrap();

    let json_response = json!({
        "status": "success",
        "results": viewers.len(),
        "viewers": viewers
    });

    Ok(Json(json_response))
}

pub async fn create_viewer_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateViewerSchema>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("create_viewer_handler");
    let query_result = sqlx::query_as!(
        ViewerModel,
        "INSERT INTO viewers (email, hashed, salt) VALUES ($1, $2, $3) RETURNING *",
        body.email.to_string(),
        body.hashed.to_string(),
        body.salt.to_string()
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(viewer) => {

            println!("create_viewer_handler: POST successful");
            let viewer_response = json!({"status": "success","data": json!({
                "viewer": viewer
            })});

            return Ok((StatusCode::CREATED, Json(viewer_response)));
        }
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violates unique constraint")
            {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": "Viewer with that title already exists",
                });
                println!("create_viewer_handler: POST failed: duplicate key");
                return Err((StatusCode::CONFLICT, Json(error_response)));
            }

            println!("create_viewer_handler: POST failed");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
        }
    }
}

pub async fn get_viewer_handler(
    State(data): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(
        ViewerModel,
        "SELECT * FROM viewers WHERE id = $1",
        id
    ).fetch_one(&data.db)
    .await;

    match query_result {
        Ok(viewer) => {
            let viewer_response = json!({"status": "success", "data": json!({ "viewer": viewer})});
            Ok(Json(viewer_response))
        },
        Err(e) => {
            let error_response = json!({"status": "fail", "message": format!("Viewer with id {} not found: {:?}", id, e)});
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}

pub async fn delete_viewer_handler(
    State(data): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let rows_affected = sqlx::query!(
        "DELETE FROM viewers WHERE id = $1", id)
        .execute(&data.db)
        .await
        .unwrap()
        .rows_affected();

    if rows_affected == 0 {
        let error_response = json!({
            "status": "fail",
            "message": format!("Viewer with id: {}, not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn edit_viewer_handler(
    State(data): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
    Json(body): Json<UpdateViewerSchema>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(
        ViewerModel,
        "SELECT * FROM viewers WHERE id = $1",
        id
    ).fetch_one(&data.db).await;

    if query_result.is_err() {
        let error_response = json!({
            "status": "fail",
            "message": format!("Viewer with id: {} not found.", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let now = chrono::Utc::now();
    let viewer = query_result.unwrap();

    let query_result = sqlx::query_as!(
        ViewerModel,
        "UPDATE viewers SET email = $1, hashed = $2, salt = $3, updated_at = $4 WHERE id = $5 RETURNING *",
        body.email.to_owned().unwrap_or(viewer.email),
        body.hashed.to_owned().unwrap_or(viewer.hashed),
        body.salt.to_owned().unwrap_or(viewer.salt),
        now,
        id
    )
    .fetch_one(&data.db).await;

    match query_result {
        Ok(viewer) => {
            let viewer_response = json!({
                "status": "success",
                "data": json!({ "viewer": viewer})
            });
            Ok(Json(viewer_response))

        },
        Err(e) => {
            let error_response = json!({
                "status": "fail",
                "message": format!("something went wrong while updating viewer with id: {}, {:?}", id, e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
