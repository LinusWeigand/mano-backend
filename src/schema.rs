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
pub struct PreRegisterSchema {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterSchema {
    pub verification_code: String,
    pub email: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateViewerSchema {
    pub email: Option<String>,
    pub hashed: Option<String>,
    pub salt: Option<String>,
    pub verified: Option<bool>,
}
