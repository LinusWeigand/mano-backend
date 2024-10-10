use std::sync::Arc;

use axum::{
    extract::{Multipart, Path, Request, State},
    http::{
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
        StatusCode,
    },
    response::IntoResponse,
    Json,
};
use base64::decode;
use lettre::message::Body;
use serde_json::json;
use uuid::Uuid;

use crate::{
    model::{PhotoDataModel, PhotoMetadataModel, PhotoModel, ProfileModel, ViewerModel},
    schema::CreateProfilSchema,
    AppState,
};

pub async fn create_profile2(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateProfilSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("create_profile");

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

pub async fn create_profile(
    State(data): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("create profile");

    let mut name: Option<String> = None;
    let mut craft: Option<String> = None;
    let mut location: Option<String> = None;
    let mut website: Option<String> = None;
    let mut instagram: Option<String> = None;
    let mut skills: Option<Vec<String>> = None;
    let mut bio: Option<String> = None;
    let mut experience: Option<i16> = None;
    let mut photos = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        eprintln!("upload_photos: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": "Internal Server Error"
            })),
        )
    })? {
        let field_name = field.file_name().unwrap_or("").to_string();

        if let Some(content_type) = field.content_type() {
            //This is a file field

            let content_type = content_type.to_string();

            if !content_type.starts_with("image/") {
                eprintln!("Unsupported media type");
                return Err((
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    Json(json!({
                        "status": "fail",
                        "message": "unsupported media type"
                    })),
                ));
            }

            let file_name = field.file_name().unwrap_or("").to_string();

            let photo_data = field.bytes().await.map_err(|e| {
                eprintln!("upload_photos: Error reading image field: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "status": "fail",
                        "message": "Internal Server Error"
                    })),
                )
            })?;

            println!(
                "File_name: {}, content_type: {}: ",
                &file_name, &content_type
            );

            photos.push((file_name, content_type.to_string(), photo_data));
        } else {
            // This is a text field
            let text = field.text().await.map_err(|e| {
                eprintln!(
                    "upload_photos: Error reading text field {} : {:?}",
                    field_name, e
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "status": "fail",
                        "message": "Internal Server Error"
                    })),
                )
            })?;

            match field_name.as_str() {
                "name" => name = Some(text),
                "craft" => craft = Some(text),
                "location" => location = Some(text),
                "website" => website = Some(text),
                "instagram" => instagram = Some(text),
                "skills" => {
                    // Parse skills as a JSON array
                    let skills_vec = serde_json::from_str(&text).map_err(|e| {
                        eprintln!("Error parsing skills: {:?}", e);
                        (
                            StatusCode::BAD_REQUEST,
                            Json(json!({ "status": "fail", "message": "Invalid skills format" })),
                        )
                    })?;
                    skills = Some(skills_vec);
                }
                "bio" => bio = Some(text),
                "experience" => {
                    let exp: i16 = text.parse().map_err(|e| {
                        eprintln!("Error parsing experience: {:?}", e);
                        (
                            StatusCode::BAD_REQUEST,
                            Json(
                                json!({ "status": "fail", "message": "Invalid experience format" }),
                            ),
                        )
                    })?;
                    experience = Some(exp);
                }
                _ => eprintln!("Unknown field: {}", field_name),
            }
        }
    }

    if name.is_none() || craft.is_none() || location.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "fail",
                "message": "Missing required profile data"
            })),
        ));
    }

    let name = name.unwrap_or_default();
    let craft = craft.unwrap_or_default();
    let location = location.unwrap_or_default();
    let website = website.unwrap_or_default();
    let instagram = instagram.unwrap_or_default();
    let skills = skills.unwrap_or_default();
    let bio = bio.unwrap_or_default();
    let experience = experience.unwrap_or_default();

    let email = "linus@couchtec.com".to_string();

    let query = sqlx::query!("SELECT id FROM viewers WHERE email = $1", email)
        .fetch_one(&data.db)
        .await
        .map_err(|e| {
            eprintln!(
                "create-profile: Error getting viewer id with email {} : {:?}",
                &email, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "fail",
                    "message": "Internal Server Error"
                })),
            )
        })?;

    let viewer_id = query.id;

    let query = sqlx::query!(
        r#"
        INSERT INTO profiles (
            viewer_id, name, craft, location, website, instagram, skills, bio, experience
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id;
        "#,
        viewer_id,
        name,
        craft,
        location,
        website,
        instagram,
        &skills,
        bio,
        experience
    )
    .fetch_one(&data.db)
    .await
    .map_err(|e| {
        eprintln!("Error inserting profile: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "message": "Internal Server Error" })),
        )
    })?;

    let profile_id = query.id;

    let mut num_photos_inserted = 0;
    let mut num_duplicates = 0;

    for (file_name, content_type, photo_data) in photos {
        let query_result = sqlx::query!(
            "INSERT INTO photos (
                profile_id, file_name, content_type, photo_data
            ) VALUES (
                $1, $2, $3, $4
            )",
            profile_id,
            &file_name,
            &content_type,
            &photo_data.as_ref()
        )
        .execute(&data.db)
        .await;

        match query_result {
            Err(e) => {
                if e.to_string().contains("duplicate key") {
                    println!("Duplicate photo found.");
                    num_duplicates += 1;
                    continue;
                }
                eprintln!("upload_photos fail: insert into db: {:?}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "status": "fail", "message": "Internal Server Error" })),
                ));
            }
            Ok(_) => {
                println!("Photo inserted.");
                num_photos_inserted += 1;
            }
        }
    }

    println!(
        "Photos inserted: {}, Duplicates skipped: {}",
        num_photos_inserted, num_duplicates
    );

    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "message": "Profile created."
        })),
    ))
}

pub async fn get_photos(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let photos = sqlx::query_as!(
        PhotoModel,
        "SELECT id, file_name, content_type, photo_data, created_at FROM photos"
    )
    .fetch_all(&data.db)
    .await
    .map_err(|e| {
        eprintln!("get_photos: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": "Internal Server Error"
            })),
        )
    })?;
    Ok(())
}

pub async fn get_photo_metadata(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let photos = sqlx::query_as!(
        PhotoMetadataModel,
        "SELECT id, file_name, content_type FROM photos"
    )
    .fetch_all(&data.db)
    .await
    .map_err(|e| {
        eprintln!("get_photos_metadata: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": "Internal Server Error"
            })),
        )
    })?;

    let photo_responses: Vec<serde_json::Value> = photos
        .into_iter()
        .map(|photo| {
            json!({
                "id": photo.id,
                "file_name": photo.file_name,
                "content_type": photo.content_type,
                "url": format!("{}/api/photos/{}", &data.url, photo.id)
            })
        })
        .collect();

    Ok(Json(json!({
        "status": "success",
        "data": photo_responses
    })))
}

pub async fn get_photo(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let photo = sqlx::query_as!(
        PhotoDataModel,
        "SELECT file_name, content_type, photo_data FROM photos WHERE id = $1",
        id
    )
    .fetch_one(&data.db)
    .await
    .map_err(|e| {
        eprintln!("get_photos_metadata: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": "Internal Server Error"
            })),
        )
    })?;

    let headers = [
        (CONTENT_TYPE, photo.content_type.clone()),
        (
            CONTENT_DISPOSITION,
            format!("inline; filename=\"{}\"", photo.file_name),
        ),
    ];
    Ok((headers, photo.photo_data))
}
