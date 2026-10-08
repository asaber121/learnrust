use crate::state::AppState;
use axum::Router;

pub mod admin;
pub mod auth;
pub mod health;

pub fn api_router(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/health", health::router())
        .nest("/auth", auth::router())
        .nest("/admin", admin::router(state))
}
