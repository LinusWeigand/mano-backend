use axum::{
    extract::{Path, State},
    http::{
        header::{self, SET_COOKIE},
        HeaderMap, StatusCode,
    },
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    model::{PreRegisteredModel, ResetPasswordModel, UserSessionModel, ViewerModel},
    schema::{
        LoginSchema, PreRegisterSchema, PreResetPasswordSchema, RegisterSchema, ResetPasswordSchema,
    },
    AppState,
};

pub async fn pre_register(
    State(data): State<Arc<AppState>>,
    Json(body): Json<PreRegisterSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let salt = Uuid::new_v4().to_string();
    let salted = format!("{}{}", body.password, salt);
    let mut hasher = Sha256::new();
    hasher.update(salted.as_bytes());
    let hashed_password = hex::encode(hasher.finalize());

    let query_result = sqlx::query_as!(
        ViewerModel,
        "INSERT INTO viewers (email, first_name, last_name, hashed, salt) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        body.email,
        body.first_name,
        body.last_name,
        hashed_password,
        salt
    )
    .fetch_one(&data.db)
    .await;

    // Check for errors
    if let Err(e) = query_result {
        if e.to_string()
            .contains("duplicate key value violates unique constraint")
        {
            let error_response = json!({
                "status": "fail",
                "message": "Viewer with that email already exists"
            });
            println!("Pre_register: POST fail: duplicate key");
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }
        let error_response = json!({
            "status": "error",
            "message": format!("{:?}", e)
        });
        println!("Pre_register: POST fail: duplicate key");
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let viewer = query_result.unwrap();
    let verification_code = Uuid::new_v4().to_string();
    let salt = Uuid::new_v4().to_string();
    let salted = format!("{}{}", verification_code, salt);
    let mut hasher = Sha256::new();
    hasher.update(salted.as_bytes());
    let verification_code_hashed = hex::encode(hasher.finalize());
    let viewer_id = viewer.id;

    let query_result = sqlx::query_as!(
        PreRegisteredModel,
        "INSERT INTO pre_registered (viewer_id, verification_code_hashed, salt) VALUES ($1, $2, $3) RETURNING *",
        viewer_id,
        verification_code_hashed,
        salt
    ).fetch_one(&data.db).await;

    if let Err(e) = query_result {
        let error_response = json!({
            "status": "error",
            "message": format!("{:?}", e)
        });
        println!("Pre_register: POST fail: duplicate key");
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    //Send Email to verify
    let subject = "E-Mail verifizieren";
    let body = format!(
        "Klicke diesen Link um deine E-Mail zu verifizieren: {}?vc={}&e={}",
        &data.url,
        urlencoding::encode(&verification_code),
        urlencoding::encode(&viewer.email)
    );
    let email_result = data.email_manager.send_email(&viewer.email, subject, &body);

    if let Err(e) = email_result {
        let error_response = json!({
            "status": "error",
            "message": "Internal Server Error"
        });
        println!("pre_register: E-Mail failed: {:?}", e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let viewer_reponse = json!({
        "status": "success",
        "data" : json!({
            "viewer": viewer
        })
    });
    Ok((StatusCode::OK, Json(viewer_reponse)))
}

pub async fn register(
    State(data): State<Arc<AppState>>,
    Json(body): Json<RegisterSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(
        PreRegisteredModel,
        "SELECT * FROM pre_registered WHERE viewer_id = (SELECT id FROM viewers WHERE email = $1)",
        body.email
    )
    .fetch_one(&data.db)
    .await;

    if let Err(e) = query_result {
        let error_response = json!({
            "status": "fail",
            "message": "Verification failed: No matching record found."
        });
        println!("register: fail: kein Token gefunden.");
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let pre_registered_entry = query_result.unwrap();
    let salted = format!("{}{}", body.verification_code, pre_registered_entry.salt);
    let mut hasher = Sha256::new();
    hasher.update(salted.as_bytes());
    let hashed_verification_code = hex::encode(hasher.finalize());

    if hashed_verification_code != pre_registered_entry.verification_code_hashed {
        let error_response = json!({
            "status": "fail",
            "message": "Verification code does not match."
        });
        println!("register: fail: Verification code does not match");
        return Err((StatusCode::FORBIDDEN, Json(error_response)));
    }

    if pre_registered_entry.was_used {
        let error_response = json!({
            "status": "fail",
            "message": "Verification Token used already"
        });
        println!("register: fail: Verification Token used already");
        return Err((StatusCode::CONFLICT, Json(error_response)));
    }

    //Set verified to true
    let query_result = sqlx::query_as!(
        ViewerModel,
        "UPDATE viewers SET verified = TRUE WHERE email = $1 RETURNING *",
        body.email
    )
    .fetch_one(&data.db)
    .await;

    if let Err(e) = query_result {
        let error_response = json!({
            "status": "fail",
            "message": "Internal Server Error"
        });
        println!("register: fail: failed to set verified to true");
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    //Set was_used to true
    let query_result = sqlx::query_as!(
        PreRegisteredModel,
        "UPDATE pre_registered SET was_used = TRUE WHERE id = $1 RETURNING *",
        pre_registered_entry.id
    )
    .fetch_one(&data.db)
    .await;

    if let Err(_) = query_result {
        let error_response = json!({
            "status": "fail",
            "message": "Internal Server Error"
        });
        println!("register: fail: failed to set pre_registered was_used to true.");
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let response = json!({
        "status": "success",
        "message": "User verified"
    });

    Ok((StatusCode::OK, Json(response)))
}

pub async fn login(
    State(data): State<Arc<AppState>>,
    Json(body): Json<LoginSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("login attempt.");
    let query_result = sqlx::query_as!(
        ViewerModel,
        "SELECT * FROM viewers WHERE email = $1",
        &body.email
    )
    .fetch_one(&data.db)
    .await;

    if let Err(_) = query_result {
        let error_response = json!({
            "status": "fail",
            "message": "User not found."
        });
        println!("login: fail: user not found.");
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let viewer = query_result.unwrap();
    let salted = format!("{}{}", &body.password, &viewer.salt);
    let mut hasher = Sha256::new();
    hasher.update(salted.as_bytes());
    let hashed_password = hex::encode(hasher.finalize());

    if hashed_password != viewer.hashed {
        let error_response = json!({
            "status": "success",
            "message": "Password incorrect"
        });
        println!("login: fail: password incorrect");
        return Err((StatusCode::FORBIDDEN, Json(error_response)));
    }

    let session_token = Uuid::new_v4();
    let salt = Uuid::new_v4().to_string();
    let salted = format!("{}{}", session_token, salt);
    let mut hasher = Sha256::new();
    hasher.update(salted.as_bytes());
    let hashed_session_token = hex::encode(hasher.finalize());

    // Create Session Token
    let query_result = sqlx::query_as!(
        UserSessionModel,
        "INSERT INTO user_sessions (viewer_id, hashed_session_token, salt) VALUES ($1, $2, $3) RETURNING *",
        &viewer.id,
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
    let cookie = format!(
        "session_token={}; HttpOnly; Secure; Path=/; SameSite=Strict; Max-Age={}",
        session_token,
        60 * 60 * 24 * 7 // 1 week in seconds
    );

    let response = json!({
        "status": "success",
        "data": "User logged in."
    });
    println!("Login successful.");
    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie)],
        Json(response),
    )
        .into_response())
}

pub async fn pre_reset_password(
    State(data): State<Arc<AppState>>,
    Json(body): Json<PreResetPasswordSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("pre_reset_password");
    let query_result = sqlx::query_as!(
        ViewerModel,
        "SELECT * FROM viewers WHERE email = $1",
        &body.email
    )
    .fetch_one(&data.db)
    .await;

    if let Err(e) = query_result {
        let error_response = json!({
            "status": "fail",
            "message": format!("User with email {} not found", &body.email)
        });
        println!(
            "pre_reset_password: fail: User witt email {} not found",
            &body.email
        );
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let viewer = query_result.unwrap();
    let reset_password_token = Uuid::new_v4().to_string();
    let salt = Uuid::new_v4().to_string();
    let salted = format!("{}{}", reset_password_token, salt);
    let mut hasher = Sha256::new();
    hasher.update(salted.as_bytes());
    let hashed_reset_password_token = hex::encode(hasher.finalize());

    // Create Reset Password Token
    let query_result = sqlx::query_as!(
        ResetPasswordModel,
        "INSERT INTO reset_password (viewer_id, hashed_reset_password_token, salt) VALUES ($1, $2, $3) RETURNING *",
        &viewer.id,
        &hashed_reset_password_token,
        &salt,
    ).fetch_one(&data.db).await;

    if let Err(e) = query_result {
        let error_response = json!({
            "status": "fail",
            "message": "Internal Server Error"
        });
        println!("pre_reset_password: fail: Something went wrong while trying to insert a resert password Token.");
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let subject = "Passwort zurücksetzen";
    let body = format!(
        "Klicke diesen Link um dein Passwort zurückzusetzen: {}/reset-password?c={}&e={}",
        &data.url,
        urlencoding::encode(&reset_password_token),
        urlencoding::encode(&viewer.email)
    );
    let email_result = data.email_manager.send_email(&viewer.email, subject, &body);

    if let Err(e) = email_result {
        let error_response = json!({
            "status": "error",
            "message": "Internal Server Error"
        });
        println!("pre_reset_password: E-Mail failed: {:?}", e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let response = json!({
        "status": "success",
        "message": "Zurücksetzungs E-Mail gesendet."
    });

    Ok((StatusCode::OK, Json(response)))
}

pub async fn reset_password(
    State(data): State<Arc<AppState>>,
    Json(body): Json<ResetPasswordSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("pre_reset_password");
    let query_result = sqlx::query_as!(
        ResetPasswordModel,
        "SELECT * FROM reset_password WHERE viewer_id = (SELECT id FROM viewers WHERE email = $1)",
        &body.email
    )
    .fetch_one(&data.db)
    .await;

    if let Err(e) = query_result {
        let error_response = json!({
            "status": "fail",
            "message": "Passwort Reset failed: No matching record found."
        });
        println!("reset_password: fail: kein Token gefunden.");
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let reset_password_entry = query_result.unwrap();
    let salted = format!("{}{}", body.reset_password_token, reset_password_entry.salt);
    let mut hasher = Sha256::new();
    hasher.update(salted.as_bytes());
    let hashed_reset_password_token = hex::encode(hasher.finalize());

    if hashed_reset_password_token != reset_password_entry.hashed_reset_password_token {
        let error_response = json!({
            "status": "fail",
            "message": "Reset Password token does not match."
        });
        println!("reset_password: fail: Reset passwort token does not match");
        return Err((StatusCode::FORBIDDEN, Json(error_response)));
    }

    let salt = Uuid::new_v4().to_string();
    let salted = format!("{}{}", body.password, salt);
    let mut hasher = Sha256::new();
    hasher.update(salted.as_bytes());
    let hashed_password = hex::encode(hasher.finalize());

    // Update viewer row
    let query_result = sqlx::query_as!(
        ViewerModel,
        "UPDATE viewers SET hashed = $1, salt = $2 RETURNING *",
        hashed_password,
        salt
    )
    .fetch_one(&data.db)
    .await;

    if let Err(e) = query_result {
        let error_response = json!({
            "status": "fail",
            "message": "Internal Server Error"
        });
        println!("reset_password: fail: something went wrong while trying to update viewers hashed and salt: {:?}, ", e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    //Set was_used to true
    let query_result = sqlx::query_as!(
        ResetPasswordModel,
        "UPDATE reset_password SET was_used = TRUE WHERE id = $1 RETURNING *",
        reset_password_entry.id
    )
    .fetch_one(&data.db)
    .await;

    if let Err(_) = query_result {
        let error_response = json!({
            "status": "fail",
            "message": "Internal Server Error"
        });
        println!("register: fail: failed to set reset_password was_used to true.");
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let response = json!({
        "status": "success",
        "message": "Password reset."
    });

    Ok((StatusCode::OK, Json(response)))
}

pub async fn get_viewer(
    State(data): State<Arc<AppState>>,
    Path(email): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result =
        sqlx::query_as!(ViewerModel, "SELECT * FROM viewers WHERE email = $1", email)
            .fetch_one(&data.db)
            .await;

    match query_result {
        Ok(viewer) => {
            let viewer_response = json!({"status": "success", "data": json!({ "viewer": viewer})});
            println!("get_viewer: viewer found.");
            Ok(Json(viewer_response))
        }
        Err(e) => {
            let error_response = json!({"status": "fail", "message": format!("Viewer with id {} not found: {:?}", email, e)});
            println!("get_viewer: viewer not found.");
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}
