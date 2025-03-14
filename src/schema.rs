use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateViewerSchema {
    pub email: String,
    pub hashed: String,
    pub salt: String,
    pub verified: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginSchema {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PreRegisterSchema {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterSchema {
    pub verification_code: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PreResetPasswordSchema {
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResetPasswordSchema {
    pub email: String,
    pub password: String,
    pub reset_password_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateViewerSchema {
    pub email: Option<String>,
    pub hashed: Option<String>,
    pub salt: Option<String>,
    pub verified: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionVerifySchema {
    pub email: String,
    pub session_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateProfilSchema {
    pub email: String,
    pub name: String,
    pub craft: String,
    pub location: String,
    pub website: Option<String>,
    pub instagram: Option<String>,
    pub skills: Vec<String>,
    pub bio: String,
    pub register_number: String,
    pub experience: i16,
    pub google_rating: Option<String>,
    pub myhammer_rating: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchSchema {
    pub name: Option<String>,
    pub craft: Option<String>,
    pub location: Option<String>,
    pub skill: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateCraftSchema {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateCraftSchema {
    pub old_name: String,
    pub new_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateSkillSchema {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateSkillSchema {
    pub old_name: String,
    pub new_name: String,
}
