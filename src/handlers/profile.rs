use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use base64::decode;
use lettre::message::Body;
use serde_json::json;

use crate::{
    model::{ProfileModel, ViewerModel},
    schema::CreateProfilSchema,
    AppState,
};

pub async fn create_profile(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateProfilSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("create_profile");
    // Decode the base64 strings to bytes for portfolio

    let query_result = sqlx::query_as!(
        ViewerModel,
        "SELECT * FROM viewers WHERE email = $1",
        &body.email,
    )
    .fetch_one(&data.db)
    .await;

    if let Err(e) = query_result {
        println!("create_profile: fail: get viewer_id from email: {:?}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": "Internal Server Error"
            })),
        ));
    }
    let viewer_id = query_result.unwrap().id;

    let query_result = sqlx::query_as!(
        ProfileModel,
        r#"
        INSERT INTO profiles (
            viewer_id, name, craft, location, website, instagram, skills, bio, experience 
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *;
        "#,
        viewer_id,
        body.name,
        body.craft,
        body.location,
        body.website,
        body.instagram,
        &body.skills,
        body.bio,
        body.experience,
    )
    .fetch_one(&data.db)
    .await;

    if let Err(e) = query_result {
        if e.to_string()
            .contains("duplicate key value violates unique constraint")
        {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Profile with the provided details already exists",
            });
            println!("create_profile_handler: POST failed: duplicate key");
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }
        println!("create_profile_handler: POST failed: {:?}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": "Internal Server Error"
            })),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "message": "Profil erstellt."
        })),
    ))
}
