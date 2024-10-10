use crate::{
    handlers::{
        auth::{get_viewer, login, pre_register, pre_reset_password, register, reset_password},
        crud::{
            create_viewer_handler, delete_viewer_handler, edit_viewer_handler,
            health_checker_handler, viewer_list_handler,
        },
        profile::{
            create_profile, get_photo, get_photo_metadata, get_photos_of_profile, get_profiles,
        },
        // profile::create_profile,
    },
    AppState,
};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .route("/api/viewers", post(create_viewer_handler))
        .route("/api/pre-register", post(pre_register))
        .route("/api/login", post(login))
        .route("/api/pre-reset-password", post(pre_reset_password))
        .route("/api/reset-password", post(reset_password))
        .route("/api/register", post(register))
        .route("/api/viewers", get(viewer_list_handler))
        .route("/api/profiles", get(get_profiles))
        .route("/api/profile", post(create_profile))
        .route("/api/photos", get(get_photo_metadata))
        .route("/api/photos/:id", get(get_photo))
        .route("/api/profile-photos/:id", get(get_photos_of_profile))
        // .route("/api/profile", post(create_profile))
        .route(
            "/api/viewers/:id",
            get(get_viewer)
                .patch(edit_viewer_handler)
                .delete(delete_viewer_handler),
        )
        .with_state(app_state)
}
