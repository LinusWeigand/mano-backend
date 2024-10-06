use std::{ptr::hash, result, sync::Arc};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{model::ViewerModel, schema::CreateViewerSchema, AppState};

pub async fn pre_register(
    email: &str,
    password: &str,
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateViewerSchema>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let salt = Uuid::new_v4().to_string();
    let salted = format!("{}{}", password, salt);

    let mut hasher = Sha256::new();
    hasher.update(salted.as_bytes());
    let result = hex::encode(hasher.finalize());

    let query_result = sqlx::query_as!(
        ViewerModel,
        "INSERT INTO viewers (email, hashed, salt) VALUES ($1, $2, $3) RETURNING *",
        body.email,
        result,
        salt
    ).fetch_one(&data.db).await;
    
    match query_result {
        Ok(viewer) => {
            println!("Pre_register: POST successful");
            let viewer_response = json!({"status": "success", "data": json!({
                "viewer": viewer
            })});
            Ok((StatusCode::CREATED, Json(viewer_response)))
        },
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violates unique constraint")
                {
                    let error_response = json!({
                        "status": "fail",
                        "message": "Viewer with that title already exists"
                    });
                    println!("Pre_register: POST fail: duplicate key");
                    return Err((StatusCode::CONFLICT, Json(error_response)));
                }
            let error_response = json!({
                "status": "error",
                "message": format!("{:?}", e)
            });
            println!("Pre_register: POST fail: duplicate key");
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn register(

) -> Result<impl IntoResponse, (StatusCode, serde_json::Value)> {
    Ok(())
}