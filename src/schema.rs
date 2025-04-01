use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

// #[derive(Deserialize, Default)]
// pub struct FilterOptions {
//     page: Option<usize>,
//     limit: Option<usize>,
// }

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
pub struct SearchSchema {
    pub name: Option<String>,
    pub craft: Option<String>,
    pub location: Option<String>,
    pub skill: Option<String>,
    pub range: Option<f64>, // in kilometers
    pub lat: Option<f64>,
    pub lng: Option<f64>,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateRechtsformSchema {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateRechtsformSchema {
    pub old_name: String,
    pub new_name: String,
}
