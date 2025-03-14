use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use uuid::Uuid;

use crate::{
    model::SkillModel,
    schema::{CreateSkillSchema, UpdateSkillSchema},
    AppState,
};

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

pub async fn create_skill(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateSkillSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let new_skill = sqlx::query!(
        r#"
        INSERT INTO skills (name)
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
                    "message": "Skill already exists"
                })),
            )
        } else {
            eprintln!("Error creating skill: {:?}", e);
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
                "id": new_skill.id,
                "name": new_skill.name
            }
        })),
    ))
}

pub async fn update_skill(
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateSkillSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("update skill");
    let updated_skill = sqlx::query!(
        r#"
        UPDATE skills SET name = $1
        WHERE name = $2
        RETURNING id, name;
        "#,
        body.new_name,
        body.old_name
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|e| {
        eprintln!("Error updating skill: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "message": "Internal Server Error" })),
        )
    })?;

    println!("query done");

    if let Some(skill) = updated_skill {
        Ok((
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "data": { "id": skill.id, "name": skill.name }
            })),
        ))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "status": "fail", "message": "Skill with provided old name not found" })),
        ))
    }
}
