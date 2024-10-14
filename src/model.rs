use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct ViewerModel {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub hashed: String,
    pub salt: String,
    pub verified: bool,
    pub is_admin: bool,
    #[serde(rename = "createdAt")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "lastLogin")]
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct PreRegisteredModel {
    pub id: Uuid,
    pub viewer_id: Uuid,
    pub verification_code_hashed: String,
    pub salt: String,
    pub was_used: bool,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "expiresAt")]
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct UserSessionModel {
    pub id: Uuid,
    pub viewer_id: Uuid,
    pub hashed_session_token: String,
    pub salt: String,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "expiresAt")]
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct ResetPasswordModel {
    pub id: Uuid,
    pub viewer_id: Uuid,
    pub hashed_reset_password_token: String,
    pub salt: String,
    pub was_used: bool,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "expiresAt")]
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct ProfileModel {
    pub id: Uuid,
    pub viewer_id: Uuid,
    pub google_ratings: Option<String>,
    pub name: String,
    pub craft: String,
    pub location: String,
    pub website: Option<String>,
    pub instagram: Option<String>,
    pub skills: Vec<String>,
    pub bio: String,
    pub experience: i16,
    // pub portfolio: Vec<Vec<String>>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct ProfileUpdateModel {
    pub google_ratings: Option<String>,
    pub name: Option<String>,
    pub craft: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub instagram: Option<String>,
    pub skills: Option<Vec<String>>,
    pub bio: Option<String>,
    pub experience: Option<i16>,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct PhotoModel {
    pub id: Uuid,
    pub file_name: String,
    pub content_type: String,
    pub photo_data: Vec<u8>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct PhotoMetadataModel {
    pub id: Uuid,
    // pub profile_id: Uuid,
    pub file_name: String,
    pub content_type: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct PhotoDataModel {
    pub file_name: String,
    pub content_type: String,
    // pub profile_id: Uuid,
    pub photo_data: Vec<u8>,
}
