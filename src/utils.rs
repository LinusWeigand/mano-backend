use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{model::UserSessionModel, schema::SessionVerifySchema, AppState};


pub async fn verify_session_token(
    State(data): State<Arc<AppState>>,
    Json(body): Json<SessionVerifySchema>
) -> bool {

    let query_result = sqlx::query_as!(
        UserSessionModel,
        "SELECT * FROM user_sessions WHERE viewer_id = (SELECT id FROM viewers WHERE email = $1)",
        &body.email
    ).fetch_one(&data.db).await;
    
    if let Err(_) = query_result {
        println!("verify_session_token: fail: Session Token nicht gefunden.");
        return false;
    }

    let user_session = query_result.unwrap();
    let salted = format!("{}{}", body.session_token, user_session.salt);
    let mut hasher = Sha256::new();
    hasher.update(salted.as_bytes());
    let hashed_session_token = hex::encode(hasher.finalize());

    if hashed_session_token != user_session.hashed_session_token {
        println!("verify_session_token: fail: Session Token inkorrekt.");
        return false;
    }

    true
}