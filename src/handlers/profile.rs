use std::sync::Arc;

use axum::{
    extract::{Multipart, Path, State},
    http::{
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
        StatusCode,
    },
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::{
    model::{PhotoDataModel, PhotoMetadataModel, ProfileModel},
    schema::SearchSchema,
    AppState,
};

use super::auth::AuthenticatedViewer;

pub async fn create_profile(
    State(data): State<Arc<AppState>>,
    AuthenticatedViewer {
        viewer_id,
        is_admin,
    }: AuthenticatedViewer,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("create profile");

    // Check if viewer already has a profile when not being an admin
    if !is_admin {
        let query = sqlx::query!("SELECT * FROM profiles WHERE viewer_id = $1", &viewer_id)
            .fetch_all(&data.db)
            .await
            .map_err(|e| {
                eprintln!("create_profile error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "status": "fail",
                        "message": "Internal Server Error"
                    })),
                )
            })?;
        if query.len() > 0 {
            return Err((
                StatusCode::CONFLICT,
                Json(json!({
                    "status": "fail",
                    "message": "Profil already exists for that user."
                })),
            ));
        }
    }

    println!("No conflicting profiles...");

    let mut name: Option<String> = None;
    let mut craft_id: Option<Uuid> = None;
    let mut location: Option<String> = None;
    let mut website: Option<String> = None;
    let mut instagram: Option<String> = None;
    let mut skills: Option<Vec<String>> = None;
    let mut bio: Option<String> = None;
    let mut experience: Option<i32> = None;
    let mut google_ratings: Option<String> = None;
    let mut photos = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        eprintln!("create_profile Some fields error : {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": "Internal Server Error"
            })),
        )
    })? {
        let field_name = field.name().unwrap_or("").to_string();

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
                "craft" => {
                    let craft_result = sqlx::query!(
                        "SELECT id FROM crafts WHERE name = $1",
                        text
                    )
                    .fetch_optional(&data.db)
                    .await
                    .map_err(|e| {
                        eprintln!("Error fetching craft ID: {:?}", e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({ "status": "fail", "message": "Internal Server Error" })),
                        )
                    })?;

                    if let Some(craft_record) = craft_result {
                        craft_id = Some(craft_record.id);
                    } else {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            Json(json!({
                                "status": "fail",
                                "message": "Invalid craft name"
                            })),
                        ));
                    }
                },
                "location" => location = Some(text),
                "website" => website = Some(text),
                "google_ratings" => google_ratings = Some(text),
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
                    let exp: i32 = text.parse().map_err(|e| {
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

    if name.is_none() || craft_id.is_none() || location.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "fail",
                "message": "Missing required profile data"
            })),
        ));
    }

    let name = name.unwrap_or_default();
    let craft_id = craft_id.unwrap_or_default();
    let location = location.unwrap_or_default();
    let website = website.unwrap_or_default();
    let google_ratings = google_ratings.unwrap_or_default();
    let instagram = instagram.unwrap_or_default();
    let skills = skills.unwrap_or_default();
    let bio = bio.unwrap_or_default();
    let experience = experience.unwrap_or_default();

    let profile = sqlx::query!(
        r#"
        INSERT INTO profiles (
            viewer_id, name, craft_id, location, website, google_ratings, instagram, bio, experience
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id;
        "#,
        viewer_id,
        name,
        craft_id,
        location,
        website,
        google_ratings,
        instagram,
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
    let profile_id = profile.id;

    println!("Inserting skills...");

    // Insert Skills
    if !skills.is_empty() {
        let skill_ids = sqlx::query!(
            "SELECT id, name FROM skills WHERE name = ANY($1)",
            &skills
        )
        .fetch_all(&data.db)
        .await
        .map_err(|e| {
            eprintln!("Error retrieving skill IDs: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": "error", "message": "Internal Server Error" })),
            )
        })?;

        // Ensure that all provided skill names exist in the database
        let found_skill_names: Vec<String> = skill_ids.iter().map(|s| s.name.clone()).collect();
        let missing_skills: Vec<String> = skills
            .iter()
            .filter(|s| !found_skill_names.contains(s))
            .cloned()
            .collect();

        if !missing_skills.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "fail",
                    "message": "Some skills are not recognized",
                    "missing_skills": missing_skills
                })),
            ));
        }

        // Insert skill associations into profile_skill
        for skill in skill_ids {
            sqlx::query!(
                "INSERT INTO profile_skill (profile_id, skill_id) VALUES ($1, $2)",
                profile_id,
                skill.id
            )
            .execute(&data.db)
            .await
            .map_err(|e| {
                eprintln!("Error inserting into profile_skill: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "status": "error", "message": "Internal Server Error" })),
                )
            })?;
        }
    }


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


pub async fn get_profile(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query = sqlx::query!(
        r#"
        SELECT p.*, 
            COALESCE(
                json_agg(s.name) FILTER (WHERE s.name IS NOT NULL), '[]'
            ) AS skills
        FROM profiles p
        LEFT JOIN profile_skill ps ON p.id = ps.profile_id
        LEFT JOIN skills s ON ps.skill_id = s.id
        WHERE p.id = $1
        GROUP BY p.id
        "#,
        id
    )
    .fetch_one(&data.db)
    .await
    .map_err(|e| {
        eprintln!("get_profile error: {:?}", e);
        (
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "fail",
                "message": "Profile not found"
            })),
        )
    })?;

    let skills: Vec<String> = query
        .skills
        .as_ref()
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "data": {
                "profile": {
                    "id": query.id,
                    "viewer_id": query.viewer_id,
                    "name": query.name,
                    "craft_id": query.craft_id,
                    "location": query.location,
                    "website": query.website,
                    "google_ratings": query.google_ratings,
                    "instagram": query.instagram,
                    "bio": query.bio,
                    "experience": query.experience,
                    "skills": skills 
                }
            }
        })),
    ))
}

pub async fn get_profiles(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let profiles = sqlx::query!(
        r#"
        SELECT p.*, 
            COALESCE(
                json_agg(s.name) FILTER (WHERE s.name IS NOT NULL), '[]'
            ) AS skills
        FROM profiles p
        LEFT JOIN profile_skill ps ON p.id = ps.profile_id
        LEFT JOIN skills s ON ps.skill_id = s.id
        GROUP BY p.id
        "#
    )
    .fetch_all(&data.db)
    .await
    .map_err(|e| {
        eprintln!("get_profiles error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": "Internal Server Error"
            })),
        )
    })?;

    let profiles_json: Vec<serde_json::Value> = profiles
        .iter()
        .map(|p| {
            let skills: Vec<String> = p
                .skills
                .as_ref()
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();

            json!({
                "id": p.id,
                "viewer_id": p.viewer_id,
                "name": p.name,
                "craft_id": p.craft_id,
                "location": p.location,
                "website": p.website,
                "google_ratings": p.google_ratings,
                "instagram": p.instagram,
                "bio": p.bio,
                "experience": p.experience,
                "skills": skills
            })
        })
        .collect();

    Ok(Json(json!({
        "status": "success",
        "data": profiles_json
    })))
}

pub async fn get_profiles_by_search(
    State(data): State<Arc<AppState>>,
    Json(body): Json<SearchSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("get_profiles_by_search");

    let mut query_builder = QueryBuilder::new(
        "SELECT DISTINCT profiles.* FROM profiles
        LEFT JOIN profile_skills ON profiles.id = profile_skills.profile_id
        LEFT JOIN skills ON profile_skills.skill_id = skills.id
        WHERE "
    );

    let mut has_condition = false;

    if let Some(name) = body.name.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        query_builder.push("TRIM(profiles.name) = ");
        query_builder.push_bind(name);
        has_condition = true;
    }

    if let Some(craft_name) = body.craft.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let craft_query = sqlx::query!(
            "SELECT id FROM crafts WHERE name = $1",
            craft_name
        )
        .fetch_optional(&data.db)
        .await
        .map_err(|e| {
            eprintln!("Error fetching craft ID: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": "fail", "message": "Internal Server Error" })),
            )
        })?;

        if let Some(craft_record) = craft_query {
            if has_condition {
                query_builder.push(" AND ");
            }
            query_builder.push("profiles.craft_id = ");
            query_builder.push_bind(craft_record.id);
            has_condition = true;
        } else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "fail",
                    "message": "Invalid craft name"
                })),
            ));
        }
    }

    if let Some(location) = body.location.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        if has_condition {
            query_builder.push(" AND ");
        }
        query_builder.push("TRIM(profiles.location) = ");
        query_builder.push_bind(location);
        has_condition = true;
    }

    if let Some(skill_name) = body.skill.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        if has_condition {
            query_builder.push(" AND ");
        }
        query_builder.push("TRIM(skills.name) = ");
        query_builder.push_bind(skill_name);
        has_condition = true;
    }

    if !has_condition {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "fail",
                "message": "At least one search field must be provided"
            })),
        ));
    }

    let query = query_builder.build_query_as::<ProfileModel>();

    let profiles = query.fetch_all(&data.db).await.map_err(|e| {
        eprintln!("get_profiles_by_search : {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": "Internal Server Error"
            })),
        )
    })?;
    println!("got profiles: {:?}", profiles);

    Ok(Json(json!({
        "status": "success",
        "data": profiles
    })))
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

pub async fn get_photos_of_profile(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let rows = sqlx::query!(
        r#"
            SELECT 
                photos.id
            FROM 
                photos
            WHERE 
                photos.profile_id = $1;
        "#,
        id
    )
    .fetch_all(&data.db)
    .await
    .map_err(|e| {
        eprintln!("get_photos_by_profile: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": "Internal Server Error"
            })),
        )
    })?;

    let photo_urls: Vec<String> = rows
        .iter()
        .map(|row| format!("{}/api/photos/{}", &data.url, row.id))
        .collect();

    Ok(Json(json!({ "status": "success", "data": photo_urls })))
}



pub async fn delete_profile(
    State(data): State<Arc<AppState>>,
    AuthenticatedViewer {
        viewer_id,
        is_admin,
    }: AuthenticatedViewer,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !is_admin {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "status": "fail",
                "message": "User not authorized to delete profile"
            })),
        ));
    }

    let query = sqlx::query!("DELETE FROM profiles WHERE id = $1", &id)
        .execute(&data.db)
        .await
        .map_err(|e| {
            eprintln!("failed getting profiles corresponding to user");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "fail",
                    "message": "Internal Server Error."
                })),
            )
        })?;

    if query.rows_affected() < 1 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "fail",
                "message": format!("No profile with id: {} found.", &id)
            })),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "message": "Profile deleted."
        })),
    ))
}

pub async fn update_profile(
    State(data): State<Arc<AppState>>,
    AuthenticatedViewer {
        viewer_id,
        is_admin,
    }: AuthenticatedViewer,
    Path(profile_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("update profile");

    // Fetch the profile to ensure it exists and to check permissions
    let existing_profile =
        sqlx::query!("SELECT viewer_id FROM profiles WHERE id = $1", &profile_id)
            .fetch_one(&data.db)
            .await
            .map_err(|e| {
                eprintln!("update_profile: {:?}", e);
                (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "status": "fail",
                        "message": "Profile not found"
                    })),
                )
            })?;

    // Check if the user has permission to update this profile
    if !is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "status": "fail",
                "message": "You don't have permission to update this profile"
            })),
        ));
    }

    // Initialize optional fields
    let mut name: Option<String> = None;
    let mut craft: Option<String> = None;
    let mut location: Option<String> = None;
    let mut website: Option<String> = None;
    let mut instagram: Option<String> = None;
    let mut skills: Option<Vec<String>> = None;
    let mut bio: Option<String> = None;
    let mut experience: Option<i32> = None;
    let mut google_ratings: Option<String> = None;
    let mut photos = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        eprintln!("update_profile fail : {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": "Internal Server Error"
            })),
        )
    })? {
        let field_name = field.name().unwrap_or("").to_string();

        if let Some(content_type) = field.content_type() {
            // This is a file field (photo)
            let content_type = content_type.to_string();

            if !content_type.starts_with("image/") {
                eprintln!("Unsupported media type");
                return Err((
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    Json(json!({
                        "status": "fail",
                        "message": "Unsupported media type"
                    })),
                ));
            }

            let file_name = field.file_name().unwrap_or("").to_string();

            let photo_data = field.bytes().await.map_err(|e| {
                eprintln!("update_profile: Error reading image field: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "status": "fail",
                        "message": "Internal Server Error"
                    })),
                )
            })?;

            println!("File_name: {}, content_type: {}", &file_name, &content_type);

            photos.push((file_name, content_type, photo_data));
        } else {
            // This is a text field
            let text = field.text().await.map_err(|e| {
                eprintln!(
                    "update_profile: Error reading text field {}: {:?}",
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
                "google_ratings" => google_ratings = Some(text),
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
                    let exp: i32 = text.parse().map_err(|e| {
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

    // Build the update query dynamically based on which fields are provided
    let mut query_builder = sqlx::QueryBuilder::<sqlx::Postgres>::new("UPDATE profiles SET ");
    let mut has_updates = false;

    if let Some(ref name) = name {
        query_builder.push("name = ");
        query_builder.push_bind(name);
        has_updates = true;
    }
    if let Some(ref craft) = craft {
        if has_updates {
            query_builder.push(", ");
        }
        query_builder.push("craft = ");
        query_builder.push_bind(craft);
        has_updates = true;
    }
    if let Some(ref location) = location {
        if has_updates {
            query_builder.push(", ");
        }
        query_builder.push("location = ");
        query_builder.push_bind(location);
        has_updates = true;
    }
    if let Some(ref website) = website {
        if has_updates {
            query_builder.push(", ");
        }
        query_builder.push("website = ");
        query_builder.push_bind(website);
        has_updates = true;
    }
    if let Some(ref google_ratings) = google_ratings {
        if has_updates {
            query_builder.push(", ");
        }
        query_builder.push("google_ratings = ");
        query_builder.push_bind(google_ratings);
        has_updates = true;
    }
    if let Some(ref instagram) = instagram {
        if has_updates {
            query_builder.push(", ");
        }
        query_builder.push("instagram = ");
        query_builder.push_bind(instagram);
        has_updates = true;
    }
    if let Some(ref skills) = skills {
        if has_updates {
            query_builder.push(", ");
        }
        query_builder.push("skills = ");
        query_builder.push_bind(skills);
        has_updates = true;
    }
    if let Some(ref bio) = bio {
        if has_updates {
            query_builder.push(", ");
        }
        query_builder.push("bio = ");
        query_builder.push_bind(bio);
        has_updates = true;
    }
    if let Some(experience) = experience {
        if has_updates {
            query_builder.push(", ");
        }
        query_builder.push("experience = ");
        query_builder.push_bind(experience);
        has_updates = true;
    }

    if !has_updates {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "fail",
                "message": "No fields to update"
            })),
        ));
    }

    // Finish the query
    query_builder.push(" WHERE id = ");
    query_builder.push_bind(&profile_id);
    query_builder.push(";");

    // let query = query_builder.build_query_as::<ProfileUpdateModel>();
    // Execute the update query
    let result = query_builder.build().execute(&data.db).await.map_err(|e| {
        eprintln!("Error updating profile: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "message": "Internal Server Error" })),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "status": "fail", "message": "Profile not found" })),
        ));
    }

    // Handle updating photos
    let mut num_photos_inserted = 0;
    let num_duplicates = 0;

    for (file_name, content_type, photo_data) in photos {
        let query_result = sqlx::query!(
            "INSERT INTO photos (
                profile_id, file_name, content_type, photo_data
            ) VALUES (
                $1, $2, $3, $4
            ) ON CONFLICT (profile_id, file_name) DO UPDATE SET
                content_type = EXCLUDED.content_type,
                photo_data = EXCLUDED.photo_data",
            profile_id,
            &file_name,
            &content_type,
            &photo_data.as_ref()
        )
        .execute(&data.db)
        .await;

        match query_result {
            Err(e) => {
                eprintln!("update_profile fail: insert into db: {:?}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "status": "fail", "message": "Internal Server Error" })),
                ));
            }
            Ok(_) => {
                println!("Photo inserted or updated.");
                num_photos_inserted += 1;
            }
        }
    }

    println!(
        "Photos inserted or updated: {}, Duplicates skipped: {}",
        num_photos_inserted, num_duplicates
    );

    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "message": "Profile updated."
        })),
    ))
}
