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
    pub version: i16,
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
    pub version: i16,
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
    pub version: i16,
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
    pub version: i16,
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
    pub name: String,
    pub rechtsform_id: Uuid,
    pub email: String,
    pub telefon: String,
    pub craft_id: Uuid,
    pub experience: i16,
    pub location: String,
    pub lat: f32,
    pub lng: f32,
    pub website: Option<String>,
    pub instagram: Option<String>,
    pub bio: String,
    pub handwerks_karten_nummer: String,
    pub version: i16,
    #[serde(rename = "createdAt")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct PhotoModel {
    pub id: Uuid,
    pub file_name: String,
    pub content_type: String,
    pub photo_data: Vec<u8>,
    pub version: i16,
    #[serde(rename = "createdAt")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct PhotoDataModel {
    pub file_name: String,
    pub content_type: String,
    pub photo_data: Vec<u8>,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct SkillModel {
    pub name: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct CraftModel {
    pub name: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct RechtsformModel {
    pub name: String,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct ExplainRechtsformModel {
    pub explain_name: String,
}
