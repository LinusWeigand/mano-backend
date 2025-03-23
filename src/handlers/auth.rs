use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, Path, State},
    http::{
        header::{self},
        request::Parts,
        StatusCode,
    },
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    model::{PreRegisteredModel, ResetPasswordModel, ViewerModel},
    schema::{
        LoginSchema, PreRegisterSchema, PreResetPasswordSchema, RegisterSchema, ResetPasswordSchema,
    },
    utils, AppState,
};

pub async fn auth_status(
    State(data): State<Arc<AppState>>,
    AuthenticatedViewer {
        viewer_id,
        is_admin,
    }: AuthenticatedViewer,
) -> impl IntoResponse {
    let email = match sqlx::query_as!(
        ViewerModel,
        "SELECT * FROM viewers WHERE id = $1",
        viewer_id
    )
    .fetch_one(&data.db)
    .await
    {
        Ok(v) => v.email,
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("{:?}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

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

    Ok(Json(json!({
        "isLoggedIn": true,
        "hasProfile": has_profile,
        "email": email,
    })))
}

pub async fn is_admin(
    State(data): State<Arc<AppState>>,
    AuthenticatedViewer {
        viewer_id,
        is_admin,
    }: AuthenticatedViewer,
) -> impl IntoResponse {
    let email = match sqlx::query_as!(
        ViewerModel,
        "SELECT * FROM viewers WHERE id = $1",
        viewer_id
    )
    .fetch_one(&data.db)
    .await
    {
        Ok(v) => v.email,
        Err(e) => {
            let error_response = json!({
                "status": "error",
                "message": format!("{:?}", e)
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    };

    Ok(Json(json!({
        "isLoggedIn": true,
        "is_admin": is_admin,
    })))
}

pub async fn logout(
    State(data): State<Arc<AppState>>,
    AuthenticatedViewer {
        viewer_id,
        is_admin,
    }: AuthenticatedViewer,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Development
    let session_token_cookie = "session_token=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0";
    let session_id_cookie = "session_id=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0";

    // TODO: Production
    // let session_token_cookie =
    //     "session_token=; HttpOnly; Secure; Path=/; SameSite=Strict; Max-Age=0";
    // let session_id_cookie = "session_id=; HttpOnly; Secure; Path=/; SameSite=Strict; Max-Age=0";

    let mut headers = axum::http::HeaderMap::new();
    headers.append(header::SET_COOKIE, session_token_cookie.parse().unwrap());
    headers.append(header::SET_COOKIE, session_id_cookie.parse().unwrap());

    let rows_affected =
        match sqlx::query!("DELETE FROM user_sessions WHERE viewer_id = $1", viewer_id)
            .execute(&data.db)
            .await
        {
            Ok(v) => v.rows_affected(),
            Err(e) => {
                return Ok(Json(json!({
                    "status": "fail",
                    "message": "No session found"
                }))
                .into_response());
            }
        };

    if rows_affected == 0 {
        return Ok(Json(json!({
            "status": "fail",
            "message": "No session found"
        }))
        .into_response());
    }

    Ok(Json(json!({
        "status": "success",
        "message": "Logged out successfully"
    }))
    .into_response())
}

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
        body.email.to_lowercase(),
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
        println!("Pre_register: POST fail");
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
    let email_result = data.email_manager.send_verify_email(
        &viewer.email,
        &data.url,
        &verification_code,
        &viewer.first_name,
    );

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
        body.email.to_lowercase(),
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
    let viewer_id = &pre_registered_entry.viewer_id;
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

    utils::log_user_in(viewer_id, data).await
}

pub async fn login(
    State(data): State<Arc<AppState>>,
    Json(body): Json<LoginSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("login attempt.");
    let query_result = sqlx::query_as!(
        ViewerModel,
        "SELECT * FROM viewers WHERE email = $1",
        &body.email.to_lowercase()
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

    utils::log_user_in(&viewer.id, data).await
}

pub async fn pre_reset_password(
    State(data): State<Arc<AppState>>,
    Json(body): Json<PreResetPasswordSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("pre_reset_password");
    let query_result = sqlx::query_as!(
        ViewerModel,
        "SELECT * FROM viewers WHERE email = $1",
        &body.email.to_lowercase()
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

    println!("Pre_reset_password: insert into reset_password:");
    println!(
        "token: {}, hashed: {}",
        &reset_password_token, &hashed_reset_password_token
    );

    // Delete all old Reset Password Token
    let _ = sqlx::query_as!(
        ResetPasswordModel,
        "DELETE FROM reset_password WHERE viewer_id = (SELECT id FROM viewers WHERE email = $1)",
        &body.email
    )
    .execute(&data.db)
    .await;

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

    //Send Email to reset password
    let email_result = data.email_manager.send_reset_password_email(
        &viewer.email,
        &data.url,
        &reset_password_token,
        &viewer.first_name,
    );

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
    let viewer_id_result = sqlx::query!("SELECT id FROM viewers WHERE email = $1", &body.email)
        .fetch_optional(&data.db)
        .await;

    let viewer_id = match viewer_id_result {
        Ok(Some(record)) => record.id,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({ "status": "fail", "message": "User not found" })),
            ));
        }
        Err(e) => {
            eprintln!("Error fetching viewer_id: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": "fail", "message": "Internal Server Error" })),
            ));
        }
    };
    let query_result = sqlx::query_as!(
        ResetPasswordModel,
        "SELECT * FROM reset_password WHERE viewer_id = $1",
        viewer_id
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
    let salted = format!(
        "{}{}",
        &body.reset_password_token, &reset_password_entry.salt
    );
    let mut hasher = Sha256::new();
    hasher.update(salted.as_bytes());
    let hashed_reset_password_token = hex::encode(hasher.finalize());

    if hashed_reset_password_token != reset_password_entry.hashed_reset_password_token {
        let error_response = json!({
            "status": "fail",
            "message": "Reset Password token does not match."
        });
        println!("reset_password: fail: Reset passwort token does not match");
        println!(
            "code: {}, hashed: {}",
            &body.reset_password_token, &hashed_reset_password_token
        );
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

    let update_result = sqlx::query!(
        "UPDATE viewers SET updated_at = NOW() WHERE id = $1",
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
    let query_result = sqlx::query_as!(
        ViewerModel,
        "SELECT * FROM viewers WHERE email = $1",
        email.to_lowercase()
    )
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

pub struct AuthenticatedViewer {
    pub viewer_id: Uuid,
    pub is_admin: bool,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedViewer
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, data: &S) -> Result<Self, Self::Rejection> {
        let data = Arc::from_ref(data);

        let jar = CookieJar::from_request_parts(parts, &data)
            .await
            .map_err(|e| {
                println!("verify user fail: {:?}", e);
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!( {
                                "status": "fail",
                                "message": "Unauthorized - Missing cookies."
                    })),
                )
            })?;

        let session_token = jar.get("session_token");
        if session_token.is_none() {
            println!("verify user fail: no session token found in cookie");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "status": "fail",
                    "message": "Unauthorized - Missing session token."
                })),
            ));
        }
        let session_token = session_token.unwrap().value();

        let session_id = jar.get("session_id");
        if session_id.is_none() {
            println!("verify user fail: no session id found in cookie");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "status": "fail",
                    "message": "Unauthorized - Missing session id."
                })),
            ));
        }
        let session_id = session_id.unwrap().value();
        let session_id = match Uuid::parse_str(session_id) {
            Ok(id) => id,
            Err(_) => {
                println!("verify user fail: session_id in cookie not a uuid");
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "status": "fail",
                        "message": "Invalid session ID."
                    })),
                ));
            }
        };

        let query = sqlx::query!(
            "SELECT viewer_id, salt, hashed_session_token FROM user_sessions WHERE id = $1",
            session_id
        )
        .fetch_one(&data.db)
        .await
        .map_err(|e| {
            println!(
                "verify user fail no user_session with session_id: {} found. {:?}",
                &session_id, e
            );
            (
                StatusCode::UNAUTHORIZED,
                Json(json!( {
                    "status": "fail",
                    "message": "Unauthorized - No session token found."
                })),
            )
        })?;
        let viewer_id = query.viewer_id;
        let salted = format!("{}{}", session_token, query.salt);
        let mut hasher = Sha256::new();
        hasher.update(salted.as_bytes());
        let hashed_session_token = hex::encode(hasher.finalize());

        if hashed_session_token != query.hashed_session_token {
            println!("verify user fail: session token do not match");
            println!("session_token_cookie: {}", session_token);
            println!("hashed_session_token_cookie: {}", hashed_session_token);
            println!("hashed_session_token_db: {}", query.hashed_session_token);
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "status": "fail",
                    "message": "session token do not match"
                })),
            ));
        }

        let query = sqlx::query!("SELECT is_admin FROM viewers WHERE id = $1", &viewer_id)
            .fetch_one(&data.db)
            .await
            .map_err(|e| {
                println!(
                    "verify user fail no viewer with viewer_id: {} found. {:?}",
                    &viewer_id, e
                );
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!( {
                            "status": "fail",
                            "message": "Unauthorized - No user found."
                    })),
                )
            })?;

        Ok(AuthenticatedViewer {
            viewer_id,
            is_admin: query.is_admin,
        })
    }
}
