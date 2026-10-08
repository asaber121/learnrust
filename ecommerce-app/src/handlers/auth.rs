use axum::{Json, extract::State, http::StatusCode};

use crate::{
    dto::{
        auth::{AuthResponse, LoginRequest, RegisterRequest},
        user::UserResponse,
    },
    error::{AppError, AppResult},
    middleware::auth::AuthUser,
    models::user::User,
    state::AppState,
    utils::{jwt, password},
};

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> AppResult<(StatusCode, Json<AuthResponse>)> {
    body.validate()?;

    let email = body.email.trim().to_lowercase();
    let name = body.name.trim().to_string();

    // Argon2 is CPU-heavy: run it on a blocking thread so it doesn't stall the async runtime.
    let pw = body.password;
    let password_hash = tokio::task::spawn_blocking(move || password::hash_password(&pw))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??;

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, password_hash, name)
         VALUES ($1, $2, $3)
         RETURNING *",
    )
    .bind(&email)
    .bind(&password_hash)
    .bind(&name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
            AppError::Conflict("email is already registered".into())
        }
        other => AppError::Database(other),
    })?;

    let token = jwt::create_token(
        user.id,
        user.role,
        &state.config.jwt_secret,
        state.config.jwt_expires_hours,
    )?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: user.into(),
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let email = body.email.trim().to_lowercase();

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    let pw = body.password;
    let hash = user.password_hash.clone();
    let valid = tokio::task::spawn_blocking(move || password::verify_password(&pw, &hash))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??;

    if !valid {
        return Err(AppError::InvalidCredentials);
    }

    let token = jwt::create_token(
        user.id,
        user.role,
        &state.config.jwt_secret,
        state.config.jwt_expires_hours,
    )?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

pub async fn me(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<UserResponse>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(auth.id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?; // token is valid but the user was deleted

    Ok(Json(user.into()))
}
