use std::sync::Arc;

use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{model::UserSessionModel, AppState};

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

    // Development
    // let session_token_cookie = format!(
    //     "session_token={}; HttpOnly; Path=/; SameSite=Strict; Max-Age={}",
    //     session_token,
    //     60 * 60 * 24 * 7 // 1 week in seconds
    // );
    // let session_id_cookie = format!(
    //     "session_id={}; HttpOnly; Path=/; SameSite=Strict; Max-Age={}",
    //     session_id,
    //     60 * 60 * 24 * 7 // 1 week in seconds
    // );

    // Production

    let session_token_cookie = format!(
        "session_token={}; HttpOnly; Secure; Path=/; Domain={}; SameSite=Lax; Max-Age={}",
        session_token,
        data.domain,
        60 * 60 * 24 * 7 // 1 week in seconds
    );

    let session_id_cookie = format!(
        "session_id={}; HttpOnly; Secure; Path=/; Domain={}; SameSite=Lax; Max-Age={}",
        session_id,
        data.domain,
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

// pub struct Rating {
//     rating: f32,
//     review_count: u32,
// }

// pub async fn get_ratings_by_google_maps_link(google_maps_link: &str) -> Option<Rating> {
//     return match scrape_page(google_maps_link).await {
//         Ok(html) => {
//             if let Some((rating, review_count)) = extract_review_data(&html).await {
//                 Some(Rating {
//                     rating,
//                     review_count,
//                 })
//             } else {
//                 None
//             }
//         }
//         Err(e) => {
//             eprintln!("❌ Error: {}", e);
//             None
//         }
//     };
// }

// async fn extract_review_data(html: &str) -> Option<(f32, u32)> {
//     let document = Html::parse_document(html);
//
//     let rating_selector = Selector::parse(".fontDisplayLarge").unwrap();
//     let rating = document
//         .select(&rating_selector)
//         .next()
//         .map(|el| el.text().collect::<String>())
//         .map(|e| e.replace(",", "."))
//         .map(|el| el.parse::<f32>());
//
//     let review_count_selector = Selector::parse("button.GQjSyb span").unwrap();
//     let review_count = document
//         .select(&review_count_selector)
//         .next()
//         .map(|el| el.text().collect::<String>())
//         .map(|e| e.replace(",", "."))
//         .map(|el| el.split_whitespace().collect::<Vec<_>>()[0].parse::<u32>());
//
//     return if let (Some(Ok(rating)), Some(Ok(review_count))) = (rating, review_count) {
//         Some((rating, review_count))
//     } else {
//         None
//     };
// }

// async fn scrape_page(url: &str) -> Result<String, Box<dyn Error>> {
//     let config = BrowserConfig::builder()
//         .chrome_executable(
//             "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
//         )
//         .no_sandbox()
//         .build()?;
//
//     // Launch browser
//     let (mut browser, mut handler) = Browser::launch(config).await?;
//
//     // Keep event loop running to avoid crashes
//     tokio::spawn(async move { while let Some(_) = handler.next().await {} });
//
//     // Open a new tab
//     let page = browser.new_page(url).await?;
//     println!("🚀 Page loaded");
//
//     // Wait for the cookie pop-up to appear
//     sleep(Duration::from_secs(5)).await;
//
//     // Try to find and click the "Accept All" button
//     let accept_button_xpath = "//span[@jsname='V67aGc']/ancestor::button";
//     if let Some(accept_button) = page.find_element(accept_button_xpath).await.ok() {
//         println!("🍪 Found 'Accept All' button. Clicking...");
//         accept_button.click().await.ok();
//         sleep(Duration::from_secs(3)).await; // Wait for page to reload
//     } else {
//         println!("❌ 'Accept All' button not found.");
//     }
//
//     // Scrape the **actual** page content after accepting cookies
//     let html = page.content().await?;
//
//     // Close browser
//     browser.close().await?;
//
//     Ok(html)
// }
