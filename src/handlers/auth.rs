use std::sync::Arc;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{model::{PreRegisteredModel, ViewerModel}, schema::{PreRegisterSchema, RegisterSchema}, AppState};

pub async fn pre_register(
    State(data): State<Arc<AppState>>,
    Json(body): Json<PreRegisterSchema>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    let salt = Uuid::new_v4().to_string();
    let salted = format!("{}{}", body.password, salt);
    let mut hasher = Sha256::new();
    hasher.update(salted.as_bytes());
    let hashed_password = hex::encode(hasher.finalize());


    let query_result = sqlx::query_as!(
        ViewerModel,
        "INSERT INTO viewers (email, hashed, salt) VALUES ($1, $2, $3) RETURNING *",
        body.email,
        hashed_password,
        salt
    ).fetch_one(&data.db).await;

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
    let salted = format!("{}{}",verification_code, salt);
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
    let subject = "Verify E-Mail";
    let body =  format!("Click this link to verify your email: {}/{}", &data.url, &verification_code);
    let result = data.email_manager.send_email(&viewer.email, subject, &body);

    if let Err(e) = result {
        let error_response = json!({
            "status": "error",
            "message": format!("{:?}", e)
        });
        println!("Pre_register: E-Mail failed: duplicate key");
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
    Json(body): Json<RegisterSchema>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    let query_result = sqlx::query_as!(
        PreRegisteredModel,
        "SELECT * FROM pre_registered WHERE viewer_id = (SELECT id FROM viewers WHERE email = $1)",
        body.email
    ).fetch_one(&data.db).await;


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
    ).fetch_one(&data.db).await;

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
    ).fetch_one(&data.db).await;

    if let Err(e) = query_result {
        let error_response = json!({
            "status": "fail",
            "message": "Internal Server Error"
        });
        println!("register: fail: failed to delete pre_registered row.");
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let response = json!({
        "status": "success",
        "message": "User verified"
    });

    Ok((StatusCode::OK, Json(response)))
}