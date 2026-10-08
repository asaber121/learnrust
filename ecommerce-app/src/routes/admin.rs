use axum::{Router, middleware, routing::get};

use crate::{handlers::admin, middleware::admin::require_admin, state::AppState};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/users", get(admin::list_users).post(admin::create_user))
        .route(
            "/users/{id}",
            get(admin::get_user)
                .patch(admin::update_user)
                .delete(admin::delete_user),
        )
        .route(
            "/categories",
            get(admin::list_categories).post(admin::create_category),
        )
        .route(
            "/categories/{id}",
            get(admin::get_category)
                .patch(admin::update_category)
                .delete(admin::delete_category),
        )
        .route_layer(middleware::from_fn_with_state(state, require_admin))
}
