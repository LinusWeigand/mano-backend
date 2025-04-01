use crate::{
    handlers::{
        auth::{
            auth_status, get_viewer, is_admin, login, logout, pre_register, pre_reset_password,
            register, reset_password,
        },
        craft::{create_craft, get_crafts, update_craft},
        favorits::{add_favorite, get_favorite_profiles, remove_favorite},
        health_checker_handler, health_checker_handler2,
        profile::{
            accept_profile, create_profile, delete_profile, get_photo, get_photos_of_profile,
            get_profile, get_profile_email, get_profile_id, get_profiles, get_profiles_by_search,
            get_profiles_without_viewer, get_unaccepted_profiles, update_profile,
        },
        rechtsformen::{
            create_rechtsform, get_explain_rechtsformen, get_rechtsformen, update_rechtsform,
        },
        skill::{create_skill, get_skills, update_skill},
    },
    AppState,
};
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);
    Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .route("/api/healthchecker2", get(health_checker_handler2))
        .route("/api/pre-register", post(pre_register))
        .route("/api/login", post(login))
        .route("/api/pre-reset-password", post(pre_reset_password))
        .route("/api/reset-password", post(reset_password))
        .route("/api/register", post(register))
        .route("/api/auth/status", get(auth_status))
        .route("/api/auth/admin", get(is_admin))
        .route("/api/auth/logout", get(logout))
        .route("/api/skills", get(get_skills))
        .route("/api/skills", post(create_skill))
        .route("/api/skills", put(update_skill))
        .route("/api/crafts", get(get_crafts))
        .route("/api/crafts", post(create_craft))
        .route("/api/crafts", put(update_craft))
        .route("/api/rechtsformen", get(get_rechtsformen))
        .route("/api/rechtsformen/explain", get(get_explain_rechtsformen))
        .route("/api/rechtsformen", post(create_rechtsform))
        .route("/api/rechtsformen", put(update_rechtsform))
        .route("/api/profiles", get(get_profiles))
        .route("/api/profiles/unaccepted", get(get_unaccepted_profiles))
        .route("/api/profiles/unverified", get(get_profiles_without_viewer))
        .route("/api/profile/:id", get(get_profile))
        .route("/api/profile/:id", delete(delete_profile))
        .route("/api/profile/:id", put(update_profile))
        .route("/api/profile/email/:id", get(get_profile_email))
        .route("/api/profile/accept/:id", post(accept_profile))
        .route("/api/profile", post(create_profile))
        .route("/api/profile-id", get(get_profile_id))
        .route("/api/profiles/search", post(get_profiles_by_search))
        .route("/api/photos/:id", get(get_photo))
        .route("/api/viewers/:email", get(get_viewer))
        .route("/api/profile-photos/:id", get(get_photos_of_profile))
        .route("/api/favorites", get(get_favorite_profiles))
        .route("/api/favorites/:id", post(add_favorite))
        .route("/api/favorites/:id", delete(remove_favorite))
        .layer(cors)
        .with_state(app_state)
}
