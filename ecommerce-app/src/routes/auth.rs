use axum::{
    Router,
    routing::{get, post},
};

use crate::{handlers::auth, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/me", get(auth::me))
}
