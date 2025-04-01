use image::{
    codecs::jpeg::JpegEncoder, imageops::FilterType, io::Reader as ImageReader, GenericImageView,
};
use sqlx::Row;
use std::sync::Arc;
use std::{collections::HashMap, io::Cursor};

use axum::{
    extract::{Multipart, Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::{model::PhotoDataModel, schema::SearchSchema, AppState};

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
    let mut rechtsform_id: Option<Uuid> = None;
    let mut email: Option<String> = None;
    let mut telefon: Option<String> = None;
    let mut craft_id: Option<Uuid> = None;
    let mut experience: Option<i16> = None;
    let mut location: Option<String> = None;
    let mut lat: Option<f64> = None;
    let mut lng: Option<f64> = None;
    let mut website: Option<String> = None;
    let mut instagram: Option<String> = None;
    let mut skills: Option<Vec<String>> = None;
    let mut bio: Option<String> = None;
    let mut photos = Vec::new();
    let mut handwerks_karten_nummer: Option<String> = None;

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
                "rechtsform" => {
                    let rechtsform_result = sqlx::query!(
                        "SELECT id FROM rechtsformen WHERE explain_name = $1",
                        text
                    )
                    .fetch_optional(&data.db)
                    .await
                    .map_err(|e| {
                        eprintln!("Error fetching rechtsform ID: {:?}", e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({ "status": "fail", "message": "Internal Server Error" })),
                        )
                    })?;

                    if let Some(rechtsform_record) = rechtsform_result {
                        rechtsform_id = Some(rechtsform_record.id);
                    } else {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            Json(json!({
                                "status": "fail",
                                "message": "Invalid rechtsform name"
                            })),
                        ));
                    }
                }
                "email" => email = Some(text.to_lowercase()),
                "telefon" => telefon = Some(text),
                "craft" => {
                    let craft_result = sqlx::query!("SELECT id FROM crafts WHERE name = $1", text)
                        .fetch_optional(&data.db)
                        .await
                        .map_err(|e| {
                            eprintln!("Error fetching craft ID: {:?}", e);
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(
                                    json!({ "status": "fail", "message": "Internal Server Error" }),
                                ),
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
                }
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
                "location" => location = Some(text),
                "lng" => {
                    let value: f64 = text.parse().map_err(|e| {
                        eprintln!("Error parsing lng: {:?}", e);
                        (
                            StatusCode::BAD_REQUEST,
                            Json(
                                json!({ "status": "fail", "message": "Invalid experience format" }),
                            ),
                        )
                    })?;
                    lng = Some(value);
                }
                "lat" => {
                    let value: f64 = text.parse().map_err(|e| {
                        eprintln!("Error parsing lng: {:?}", e);
                        (
                            StatusCode::BAD_REQUEST,
                            Json(
                                json!({ "status": "fail", "message": "Invalid experience format" }),
                            ),
                        )
                    })?;
                    lat = Some(value);
                }
                "website" => website = Some(text),
                "instagram" => instagram = Some(text),
                "bio" => bio = Some(text),
                "handwerks_karten_nummer" => handwerks_karten_nummer = Some(text),
                "skills" => {
                    let skills_vec = serde_json::from_str(&text).map_err(|e| {
                        eprintln!("Error parsing skills: {:?}", e);
                        (
                            StatusCode::BAD_REQUEST,
                            Json(json!({ "status": "fail", "message": "Invalid skills format" })),
                        )
                    })?;
                    skills = Some(skills_vec);
                }

                _ => eprintln!("Unknown field: {}", field_name),
            }
        }
    }

    if name.is_none() || craft_id.is_none() || email.is_none() || location.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "fail",
                "message": "Missing required profile data"
            })),
        ));
    }

    let name = name.unwrap_or_default();
    let rechtsform_id = rechtsform_id.unwrap_or_default();
    let email = email.unwrap_or_default();
    let telefon = telefon.unwrap_or_default();
    let craft_id = craft_id.unwrap_or_default();
    let experience = experience.unwrap_or_default();
    let location = location.unwrap_or_default();
    let lat = lat.unwrap_or_default();
    let lng = lng.unwrap_or_default();
    let website = website.unwrap_or_default();
    let instagram = instagram.unwrap_or_default();
    let skills = skills.unwrap_or_default();
    let bio = bio.unwrap_or_default();
    let handwerks_karten_nummer = handwerks_karten_nummer.unwrap_or_default();

    let profile = sqlx::query!(
        r#"
        INSERT INTO profiles (
            viewer_id, name, rechtsform_id, email, telefon, craft_id, experience, location, lat, lng, website, instagram, bio, handwerks_karten_nummer, accepted
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        RETURNING id;
        "#,
        if is_admin { None } else { Some(viewer_id) },
        name,
        rechtsform_id,
        email,
        telefon,
        craft_id,
        experience,
        location,
        lat,
        lng,
        website,
        instagram,
        bio,
        handwerks_karten_nummer,
        if is_admin { true } else { false }
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
        let skill_ids = sqlx::query!("SELECT id, name FROM skills WHERE name = ANY($1)", &skills)
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

    println!("Compressing images...");
    for (file_name, _original_content_type, original_bytes) in photos {
        // 1) Decode raw bytes -> DynamicImage
        let dyn_img = match ImageReader::new(Cursor::new(&original_bytes)).with_guessed_format() {
            Ok(reader) => match reader.decode() {
                Ok(img) => img,
                Err(err) => {
                    eprintln!("Failed to decode image: {:?}", err);
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "status": "fail", "message": "Invalid image data" })),
                    ));
                }
            },
            Err(err) => {
                eprintln!("Failed to guess format: {:?}", err);
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "status": "fail", "message": "Invalid image data" })),
                ));
            }
        };

        // 2) Resize so that max width = 600 or max height = 600 (whichever is bigger).
        //    This preserves the aspect ratio.
        let (orig_w, orig_h) = dyn_img.dimensions();
        let max_dim = 800u32;

        // Compute scale factors for each dimension
        let scale_w = max_dim as f64 / orig_w as f64;
        let scale_h = max_dim as f64 / orig_h as f64;
        // We pick the smaller scale so that neither dimension exceeds 600
        let scale = scale_w.min(scale_h).min(1.0);
        // If the image is already smaller than 600 in both dimensions, scale=1.0 => no resize

        let new_w = (orig_w as f64 * scale).round() as u32;
        let new_h = (orig_h as f64 * scale).round() as u32;

        let resized_img = if new_w != orig_w || new_h != orig_h {
            dyn_img.resize_exact(new_w, new_h, FilterType::CatmullRom)
        } else {
            // no resize needed
            dyn_img
        };

        // 3) Encode as JPEG, iterating until we get <= 200 KB or we hit minimal quality
        let mut quality = 90; // start quality
        let mut compressed_bytes = Vec::new();
        const MAX_SIZE: usize = 400_000; // 400 KB
        const MIN_QUALITY: u8 = 10;

        loop {
            compressed_bytes.clear();

            // Create a JpegEncoder with the current quality
            let mut encoder = JpegEncoder::new_with_quality(&mut compressed_bytes, quality);

            // Encode the resized DynamicImage
            if let Err(e) = encoder.encode_image(&resized_img) {
                eprintln!("JPEG encode error: {:?}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "status": "fail", "message": "Failed to compress image" })),
                ));
            }

            if compressed_bytes.len() <= MAX_SIZE {
                // Good enough, break
                break;
            }

            if quality <= MIN_QUALITY {
                // We tried to get under 200 KB, but can't; accept current size
                println!(
                    "WARNING: Could not reduce below 200 KB even at Q={}",
                    quality
                );
                break;
            }

            // Decrease quality by 5 and try again
            quality = quality.saturating_sub(5);
        }

        // 4) Insert into DB with content_type = "image/jpeg"
        let jpeg_content_type = "image/jpeg";
        let query_result = sqlx::query!(
            r#"INSERT INTO photos (profile_id, file_name, content_type, photo_data)
               VALUES ($1, $2, $3, $4)"#,
            profile_id,
            &file_name,
            jpeg_content_type,
            &compressed_bytes
        )
        .execute(&data.db)
        .await;

        // 5) Error & duplicate handling
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
                println!(
                    "Photo inserted. final size={} KB, quality={}",
                    compressed_bytes.len() / 1000,
                    quality
                );
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

pub async fn get_profiles(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let rows = sqlx::query!(
        r#"
        SELECT p.id
        FROM profiles p
        WHERE accepted = true
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

    let response_data: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            json!({
                "id": row.id,
                "_links": {
                    "self": format!("{}/api/profile/{}", data.url, row.id)
                }
            })
        })
        .collect();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert(
        "Cache-Control",
        HeaderValue::from_static("public, max-age=10"),
    );

    Ok((
        headers,
        Json(json!({
            "status": "success",
            "data": response_data
        })),
    ))
}

pub async fn get_profile(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("get_profile");
    let query = sqlx::query!(
        r#"
        SELECT p.*, 
            c.name as craft_name,
            r.name as rechtsform_name,
            r.explain_name as rechtsform_explain_name,
            COALESCE(
                json_agg(s.name) FILTER (WHERE s.name IS NOT NULL), '[]'
            ) AS skills
        FROM profiles p
        LEFT JOIN rechtsformen r ON p.rechtsform_id = r.id
        LEFT JOIN crafts c ON p.craft_id = c.id
        LEFT JOIN profile_skill ps ON p.id = ps.profile_id
        LEFT JOIN skills s ON ps.skill_id = s.id
        WHERE p.id = $1
        GROUP BY p.id, c.name, r.name, r.explain_name
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

    let profile = json!({
        "status": "success",
        "data": {
            "profile": {
                "id": query.id,
                "viewer_id": query.viewer_id,
                "name": query.name,
                "rechtsform_name": query.rechtsform_name,
                "rechtsform_explain_name": query.rechtsform_explain_name,
                "email": query.email.to_lowercase(),
                "telefon": query.telefon,
                "craft": query.craft_name,
                "experience": query.experience,
                "location": query.location,
                "lat": query.lat,
                "lng": query.lng,
                "website": query.website,
                "instagram": query.instagram,
                "bio": query.bio,
                "handwerks_karten_nummer": query.handwerks_karten_nummer,
                "skills": skills
            }
        }
    });

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert(
        "Cache-Control",
        HeaderValue::from_static("public, max-age=10"),
    );

    Ok((headers, Json(profile)))
}

pub async fn get_profiles_by_search(
    State(data): State<Arc<AppState>>,
    Json(body): Json<SearchSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("get_profiles_by_search");

    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT DISTINCT profiles.id
        FROM profiles
        LEFT JOIN crafts ON profiles.craft_id = crafts.id
        LEFT JOIN profile_skill ON profiles.id = profile_skill.profile_id
        LEFT JOIN skills ON profile_skill.skill_id = skills.id
        WHERE accepted = true AND
        "#,
    );

    let mut has_condition = false;

    if let Some(name) = &body.name {
        let trimmed_name = name.trim();
        if !trimmed_name.is_empty() {
            query_builder.push("LOWER(TRIM(profiles.name)) LIKE LOWER(");
            query_builder.push_bind(format!("%{}%", trimmed_name));
            query_builder.push(")");
            has_condition = true;
        }
    }

    if let Some(craft_name) = &body.craft {
        let trimmed_craft = craft_name.trim();
        if !trimmed_craft.is_empty() {
            if has_condition {
                query_builder.push(" AND ");
            }
            query_builder.push("LOWER(crafts.name) LIKE LOWER(");
            query_builder.push_bind(format!("%{}%", trimmed_craft));
            query_builder.push(")");
            has_condition = true;
        }
    }

    // -- Distance filter (unchanged)
    if let (Some(lat_val), Some(lng_val), Some(range_val)) = (body.lat, body.lng, body.range) {
        if has_condition {
            query_builder.push(" AND ");
        }
        query_builder.push("
            (
                6371 * ACOS(
                    COS(RADIANS(");
        query_builder.push_bind(lat_val);
        query_builder.push(")) *
                    COS(RADIANS(profiles.lat)) *
                    COS(RADIANS(profiles.lng) - RADIANS(");
        query_builder.push_bind(lng_val);
        query_builder.push(")) +
                    SIN(RADIANS(");
        query_builder.push_bind(lat_val);
        query_builder.push(")) *
                    SIN(RADIANS(profiles.lat))
                )
            ) <= ");
        query_builder.push_bind(range_val);

        has_condition = true;
    }

    if let Some(skill_name) = &body.skill {
        let trimmed_skill = skill_name.trim();
        if !trimmed_skill.is_empty() {
            if has_condition {
                query_builder.push(" AND ");
            }
            query_builder.push("LOWER(TRIM(skills.name)) LIKE LOWER(");
            query_builder.push_bind(format!("%{}%", trimmed_skill));
            query_builder.push(")");
            has_condition = true;
        }
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

    let query = query_builder.build();
    let rows = query.fetch_all(&data.db).await.map_err(|e| {
        eprintln!("get_profiles_by_search error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": "Internal Server Error"
            })),
        )
    })?;

    let profile_ids: Vec<Uuid> = rows.iter().map(|row| row.get("id")).collect();

    let response_data: Vec<serde_json::Value> = profile_ids
        .into_iter()
        .map(|id| {
            json!({
                "id": id,
                "_links": {
                    "self": format!("{}/api/profile/{}", data.url, id)
                }
            })
        })
        .collect();

    Ok(Json(json!({
        "status": "success",
        "data": response_data
    })))
}
pub async fn get_photo(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(_params): Query<HashMap<String, String>>,
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

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_str(&photo.content_type).unwrap(),
    );
    headers.insert(
        "Content-Disposition",
        HeaderValue::from_str(&format!("inline; filename=\"{}\"", photo.file_name)).unwrap(),
    );

    headers.insert(
        "Cache-Control",
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    Ok((headers, photo.photo_data))
}
pub async fn get_photos_of_profile(
    State(data): State<Arc<AppState>>,
    Path(profile_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mut retries = 3;
    let mut delay = tokio::time::Duration::from_secs(1);
    
    loop {
        let result = sqlx::query!(
            r#"SELECT id FROM photos WHERE profile_id = $1"#,
            profile_id
        )
        .fetch_all(&data.db)
        .await;
        
        match result {
            Ok(rows) => {
                // Process rows and return success
                let photo_data = rows.iter().map(|row| {
                    json!({
                        "id": row.id,
                        "_links": {
                            "self": format!("{}/api/photos/{}", data.url, row.id)
                        }
                    })
                }).collect::<Vec<_>>();
                
                let mut headers = HeaderMap::new();
                headers.insert("Content-Type", HeaderValue::from_static("application/json"));
                headers.insert("Cache-Control", HeaderValue::from_static("public, max-age=60"));
                
                return Ok((headers, Json(json!({
                    "status": "success",
                    "data": photo_data
                }))));
            }
            Err(e) if retries > 0 => {
                eprintln!("get_photos_of_profile error (retrying): {:?}", e);
                retries -= 1;
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => {
                eprintln!("get_photos_of_profile error: {:?}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "status": "fail",
                        "message": "Internal Server Error"
                    })),
                ));
            }
        }
    }
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
    println!("update_profile");

    // Ensure profile exists and check permissions
    let existing_profile = sqlx::query!("SELECT viewer_id FROM profiles WHERE id = $1", profile_id)
        .fetch_optional(&data.db)
        .await
        .map_err(|e| {
            eprintln!("Error fetching profile: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": "error", "message": "Internal Server Error" })),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({ "status": "fail", "message": "Profile not found" })),
        ))?;

    if !is_admin && existing_profile.viewer_id != Some(viewer_id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "status": "fail",
                "message": "You don't have permission to update this profile"
            })),
        ));
    }

    let mut name: Option<String> = None;
    let mut rechtsform_id: Option<Uuid> = None;
    let mut email: Option<String> = None;
    let mut telefon: Option<String> = None;
    let mut craft_id: Option<Uuid> = None;
    let mut experience: Option<i16> = None;
    let mut location: Option<String> = None;
    let mut lat: Option<f64> = None;
    let mut lng: Option<f64> = None;
    let mut website: Option<String> = None;
    let mut instagram: Option<String> = None;
    let mut bio: Option<String> = None;
    let mut handwerks_karten_nummer: Option<String> = None;
    let mut skills: Option<Vec<String>> = None;
    let mut photos = Vec::new();
    let mut deleted_photos: Vec<Uuid> = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        eprintln!("create_profile Some fields error : {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "fail", "message": "Internal Server Error" })),
        )
    })? {
        let field_name = field.name().map(str::to_string).unwrap_or_default();

        if let Some(content_type) = field.content_type().map(str::to_string) {
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
            let file_name = field.file_name().map(str::to_string).unwrap_or_default();

            let photo_data = field.bytes().await.map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "status": "fail", "message": "Internal Server Error" })),
                )
            })?;

            println!("PUSHING PHOTO");
            photos.push((file_name, content_type, photo_data));
        } else {
            // Get text before moving field
            let text = field.text().await.map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "status": "fail", "message": "Internal Server Error" })),
                )
            })?;

            // Use stored `field_name` instead of calling `field.name()` again
            match field_name.as_str() {
                "name" => name = Some(text),
                "rechtsform_explain_name" => {
                    let rechtsform_result = sqlx::query!(
                        "SELECT id FROM rechtsformen WHERE explain_name = $1",
                        text
                    )
                    .fetch_optional(&data.db)
                    .await
                    .map_err(|e| {
                        eprintln!("Error fetching rechtsform ID: {:?}", e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({ "status": "fail", "message": "Internal Server Error" })),
                        )
                    })?;

                    if let Some(rechtsform_record) = rechtsform_result {
                        rechtsform_id = Some(rechtsform_record.id);
                    } else {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            Json(json!({
                                "status": "fail",
                                "message": "Invalid rechtsform name"
                            })),
                        ));
                    }
                }
                "email" => email = Some(text.to_lowercase()),
                "telefon" => telefon = Some(text),
                "craft" => {
                    let craft_result = sqlx::query!("SELECT id FROM crafts WHERE name = $1", text)
                        .fetch_optional(&data.db)
                        .await
                        .map_err(|e| {
                            eprintln!("Error fetching craft ID: {:?}", e);
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(
                                    json!({ "status": "fail", "message": "Internal Server Error" }),
                                ),
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
                }
                "experience" => {
                    experience = Some(text.parse::<i16>().map_err(|_| {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(json!({"status": "fail", "message": "Invalid experience format"})),
                        )
                    })?)
                }
                "lat" => {
                    lat = Some(text.parse::<f64>().map_err(|_| {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(json!({"status": "fail", "message": "Invalid experience format"})),
                        )
                    })?)
                }
                "lng" => {
                    lng = Some(text.parse::<f64>().map_err(|_| {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(json!({"status": "fail", "message": "Invalid experience format"})),
                        )
                    })?)
                }
                "location" => location = Some(text),
                "website" => website = Some(text),
                "instagram" => instagram = Some(text),
                "bio" => bio = Some(text),
                "handwerks_karten_nummer" => handwerks_karten_nummer = Some(text),
                "skills" => skills = Some(serde_json::from_str(&text).unwrap_or_default()),
                "deleted_photos" => {
                    deleted_photos = serde_json::from_str(&text).unwrap_or_default();
                    println!("User wants to delete photo IDs: {:?}", deleted_photos);
                }
                _ => eprintln!("Unknown field: {}", field_name),
            }
        }
    }

    let mut query_builder =
        QueryBuilder::<sqlx::Postgres>::new("UPDATE profiles SET updated_at = NOW()");

    if let Some(name) = name {
        query_builder.push(", name = ").push_bind(name);
    }
    if let Some(rechtsform_id) = rechtsform_id {
        query_builder
            .push(", rechtsform_id = ")
            .push_bind(rechtsform_id);
    }
    if let Some(craft_id) = craft_id {
        query_builder.push(", craft_id = ").push_bind(craft_id);
    }
    if let Some(email) = email {
        query_builder.push(", email = ").push_bind(email);
    }
    if let Some(telefon) = telefon {
        query_builder.push(", telefon = ").push_bind(telefon);
    }
    if let Some(location) = location {
        query_builder.push(", location = ").push_bind(location);
    }
    if let Some(lat) = lat {
        query_builder.push(", lat = ").push_bind(lat);
    }
    if let Some(lng) = lng {
        query_builder.push(", lng = ").push_bind(lng);
    }
    if let Some(website) = website {
        query_builder.push(", website = ").push_bind(website);
    }
    if let Some(instagram) = instagram {
        query_builder.push(", instagram = ").push_bind(instagram);
    }
    if let Some(bio) = bio {
        query_builder.push(", bio = ").push_bind(bio);
    }
    if let Some(handwerks_karten_nummer) = handwerks_karten_nummer {
        query_builder
            .push(", handwerks_karten_nummer = ")
            .push_bind(handwerks_karten_nummer);
    }
    if let Some(experience) = experience {
        query_builder.push(", experience = ").push_bind(experience);
    }

    query_builder.push(" WHERE id = ").push_bind(profile_id);
    query_builder.build().execute(&data.db).await.map_err(|e| {
        eprintln!("Error executing query: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "fail", "message": "Internal Server Error" })),
        )
    })?;

    sqlx::query!(
        "DELETE FROM profile_skill WHERE profile_id = $1",
        profile_id
    )
    .execute(&data.db)
    .await
    .map_err(|e| {
        eprintln!("Error deleting old skills: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "message": "Internal Server Error" })),
        )
    })?;

    if let Some(skills) = skills {
        let skill_ids = sqlx::query!("SELECT id, name FROM skills WHERE name = ANY($1)", &skills)
            .fetch_all(&data.db)
            .await
            .map_err(|e| {
                eprintln!("Error retrieving skill IDs: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "status": "error", "message": "Internal Server Error" })),
                )
            })?;

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

    for photo_id in &deleted_photos {
        let delete_result = sqlx::query!(
            "DELETE FROM photos WHERE id = $1 AND profile_id = $2",
            photo_id,
            profile_id
        )
        .execute(&data.db)
        .await;

        if let Err(e) = delete_result {
            eprintln!("Error deleting photo {photo_id}: {e}");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": "fail", "message": "Internal Server Error" })),
            ));
        }
    }

    for (file_name, _original_content_type, original_bytes) in photos {
        let dyn_img = match ImageReader::new(Cursor::new(&original_bytes)).with_guessed_format() {
            Ok(reader) => match reader.decode() {
                Ok(img) => img,
                Err(err) => {
                    eprintln!("Failed to decode image: {:?}", err);
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "status": "fail", "message": "Invalid image data" })),
                    ));
                }
            },
            Err(err) => {
                eprintln!("Failed to guess format: {:?}", err);
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "status": "fail", "message": "Invalid image data" })),
                ));
            }
        };

        let (orig_w, orig_h) = dyn_img.dimensions();
        let max_dim = 800u32;

        let scale_w = max_dim as f64 / orig_w as f64;
        let scale_h = max_dim as f64 / orig_h as f64;
        let scale = scale_w.min(scale_h).min(1.0);

        let new_w = (orig_w as f64 * scale).round() as u32;
        let new_h = (orig_h as f64 * scale).round() as u32;

        let resized_img = if new_w != orig_w || new_h != orig_h {
            dyn_img.resize_exact(new_w, new_h, FilterType::CatmullRom)
        } else {
            dyn_img
        };

        let mut quality = 90;
        let mut compressed_bytes = Vec::new();
        const MAX_SIZE: usize = 400_000;
        const MIN_QUALITY: u8 = 10;

        loop {
            compressed_bytes.clear();
            let mut encoder = JpegEncoder::new_with_quality(&mut compressed_bytes, quality);
            if let Err(e) = encoder.encode_image(&resized_img) {
                eprintln!("JPEG encode error: {:?}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "status": "fail", "message": "Failed to compress image" })),
                ));
            }

            if compressed_bytes.len() <= MAX_SIZE {
                break;
            }

            if quality <= MIN_QUALITY {
                println!(
                    "WARNING: Could not reduce below 200 KB even at Q={}",
                    quality
                );
                break;
            }

            quality = quality.saturating_sub(5);
        }

        let jpeg_content_type = "image/jpeg";
        let query_result = sqlx::query!(
            r#"INSERT INTO photos (profile_id, file_name, content_type, photo_data)
               VALUES ($1, $2, $3, $4)"#,
            profile_id,
            &file_name,
            jpeg_content_type,
            &compressed_bytes
        )
        .execute(&data.db)
        .await;

        match query_result {
            Err(e) => {
                if e.to_string().contains("duplicate key") {
                    println!("Duplicate photo found.");
                    continue;
                }
                eprintln!("upload_photos fail: insert into db: {:?}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "status": "fail", "message": "Internal Server Error" })),
                ));
            }
            Ok(_) => {
                println!(
                    "Photo inserted. final size={} KB, quality={}",
                    compressed_bytes.len() / 1000,
                    quality
                );
            }
        }
    }

    Ok((
        StatusCode::OK,
        Json(json!({"status": "success","message": "Profile updated successfully."})),
    ))
}

pub async fn delete_profile(
    State(data): State<Arc<AppState>>,
    AuthenticatedViewer {
        is_admin, ..
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
            eprintln!("failed getting profiles corresponding to user: {:?}", e);
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

pub async fn get_profile_id(
    State(data): State<Arc<AppState>>,
    AuthenticatedViewer {
        viewer_id, ..
    }: AuthenticatedViewer,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let profile_record = sqlx::query!(
        r#"
        SELECT id 
        FROM profiles
        WHERE viewer_id = $1
        "#,
        viewer_id
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|e| {
        eprintln!("Error fetching profile_id: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "fail", "message": "Internal Server Error" })),
        )
    })?;

    let profile_id = match profile_record {
        Some(record) => record.id,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "status": "fail",
                    "message": "Profile not found for this viewer"
                })),
            ))
        }
    };

    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "data": {
                "profile_id": profile_id
            }
        })),
    ))
}

pub async fn get_unaccepted_profiles(
    State(data): State<Arc<AppState>>,
    AuthenticatedViewer {
        is_admin, ..
    }: AuthenticatedViewer,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !is_admin {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "status": "fail",
                "message": "Forbidden."
            })),
        ));
    }

    let rows = sqlx::query!(
        r#"
        SELECT p.id
        FROM profiles p
        WHERE p.accepted = false
        "#
    )
    .fetch_all(&data.db)
    .await
    .map_err(|e| {
        eprintln!("get_unaccepted_profiles error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": "Internal Server Error"
            })),
        )
    })?;

    let profile_ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();

    let response_data: Vec<serde_json::Value> = profile_ids
        .into_iter()
        .map(|id| {
            json!({
                "id": id,
                "_links": {
                    "self": format!("{}/api/profile/{}", data.url, id)
                }
            })
        })
        .collect();

    Ok(Json(json!({
        "status": "success",
        "data": response_data
    })))
}

pub async fn get_profiles_without_viewer(
    State(data): State<Arc<AppState>>,
    AuthenticatedViewer {
        is_admin, ..
    }: AuthenticatedViewer,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !is_admin {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "status": "fail",
                "message": "Only admins can accept profiles."
            })),
        ));
    }

    let rows = sqlx::query!(
        r#"
        SELECT p.id
        FROM profiles p
        WHERE p.viewer_id IS NULL
        "#
    )
    .fetch_all(&data.db)
    .await
    .map_err(|e| {
        eprintln!("get_profiles_without_viewer error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": "Internal Server Error"
            })),
        )
    })?;

    let profile_ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();

    let response_data: Vec<serde_json::Value> = profile_ids
        .into_iter()
        .map(|id| {
            json!({
                "id": id,
                "_links": {
                    "self": format!("{}/api/profile/{}", data.url, id)
                }
            })
        })
        .collect();

    Ok(Json(json!({
        "status": "success",
        "data": response_data
    })))
}

pub async fn accept_profile(
    State(data): State<Arc<AppState>>,
    Path(profile_id): Path<Uuid>,
    AuthenticatedViewer {
        is_admin, ..
    }: AuthenticatedViewer,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if !is_admin {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "status": "fail",
                "message": "Only admins can accept profiles."
            })),
        ));
    }

    let result = sqlx::query!(
        r#"
        UPDATE profiles
        SET accepted = true
        WHERE id = $1
        "#,
        profile_id
    )
    .execute(&data.db)
    .await
    .map_err(|e| {
        eprintln!("Error updating profile accepted: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": "Internal Server Error"
            })),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "fail",
                "message": "Profile not found"
            })),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "message": "Profile accepted successfully"
        })),
    ))
}

pub async fn get_profile_email(
    State(data): State<Arc<AppState>>,
    Path(profile_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let record = sqlx::query!(
        r#"
        SELECT v.email
        FROM profiles p
        JOIN viewers v ON p.viewer_id = v.id
        WHERE p.id = $1
        "#,
        profile_id
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|e| {
        eprintln!("Error fetching profile email: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "fail",
                "message": "Internal Server Error"
            })),
        )
    })?;

    if let Some(rec) = record {
        Ok((
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "data": { "email": rec.email.to_lowercase() }
            })),
        ))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "fail",
                "message": "Profile not found or no owner associated"
            })),
        ))
    }
}

