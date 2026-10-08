use axum::{
    extract::{Request, State},
    http::{HeaderMap, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::{error::AppError, models::user::UserRole, state::AppState, utils::jwt};

pub async fn require_admin(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let claims = bearer_claims(request.headers(), &state)?;
    let role = sqlx::query_scalar::<_, UserRole>("SELECT role FROM users WHERE id = $1")
        .bind(claims.sub)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if role != UserRole::Admin {
        return Err(AppError::Forbidden);
    }

    request
        .extensions_mut()
        .insert(AdminUser { id: claims.sub });
    Ok(next.run(request).await)
}

#[derive(Debug, Clone, Copy)]
pub struct AdminUser {
    pub id: Uuid,
}

fn bearer_claims(headers: &HeaderMap, state: &AppState) -> Result<jwt::Claims, AppError> {
    let header = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::Unauthorized)?;
    let token = header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    jwt::verify_token(token, &state.config.jwt_secret)
}
