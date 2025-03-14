use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{model::UserSessionModel, schema::SessionVerifySchema, AppState};

pub async fn verify_session_token(
    State(data): State<Arc<AppState>>,
    Json(body): Json<SessionVerifySchema>,
) -> bool {
    let query_result = sqlx::query_as!(
        UserSessionModel,
        "SELECT * FROM user_sessions WHERE viewer_id = (SELECT id FROM viewers WHERE email = $1)",
        &body.email
    )
    .fetch_one(&data.db)
    .await;

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

pub async fn log_user_in(
    viewer_id: &Uuid,
    data: Arc<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let session_token = Uuid::new_v4();
    let salt = Uuid::new_v4().to_string();
    let salted = format!("{}{}", session_token, salt);
    let mut hasher = Sha256::new();
    hasher.update(salted.as_bytes());
    let hashed_session_token = hex::encode(hasher.finalize());

    println!("login: session_token: {}", session_token);
    // Create Session Token
    let session_id = Uuid::new_v4();
    let query_result = sqlx::query_as!(
        UserSessionModel,
        "INSERT INTO user_sessions (id, viewer_id, hashed_session_token, salt) VALUES ($1, $2, $3, $4) RETURNING *",
        &session_id,
        viewer_id,
        &hashed_session_token,
        &salt,
    ).fetch_one(&data.db).await;

    if let Err(_) = query_result {
        let error_response = json!({
            "status": "fail",
            "message": "Internal Server Error."
        });
        println!("login: fail: creating session token.");
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    // Set the session token as an HTTP-only cookie
    let session_token_cookie = format!(
        "session_token={}; HttpOnly; Secure; Path=/; SameSite=Strict; Max-Age={}",
        session_token,
        60 * 60 * 24 * 7 // 1 week in seconds
    );

    let session_id_cookie = format!(
        "session_id={}; HttpOnly; Secure; Path=/; SameSite=Strict; Max-Age={}",
        session_id,
        60 * 60 * 24 * 7 // 1 week in seconds
    );
    let mut headers = axum::http::HeaderMap::new();
    headers.append(header::SET_COOKIE, session_token_cookie.parse().unwrap());
    headers.append(header::SET_COOKIE, session_id_cookie.parse().unwrap());

    let update_result = sqlx::query!(
        "UPDATE viewers SET last_login = NOW() WHERE id = $1",
        viewer_id
    )
    .execute(&data.db)
    .await;

    if let Err(e) = update_result {
        eprintln!("Error updating last_login: {:?}", e);
        let error_response = json!({
            "status": "error",
            "message": "Failed to update last login timestamp"
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let has_profile = match sqlx::query!(
        "SELECT id FROM profiles WHERE viewer_id = $1 LIMIT 1",
        viewer_id
    )
    .fetch_optional(&data.db)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("{:?}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    let response = Json(json!({
        "status": "success",
        "data": "User logged in.",
        "hasProfile": has_profile
    }));
    println!("Login successful.");
    Ok((StatusCode::OK, headers, response).into_response())
}
