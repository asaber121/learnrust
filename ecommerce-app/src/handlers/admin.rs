use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    dto::{
        admin::{
            CategoryResponse, CreateCategoryRequest, CreateUserRequest, Paginated, PaginationQuery,
            UpdateCategoryRequest, UpdateUserRequest, category_slug,
        },
        user::UserResponse,
    },
    error::{AppError, AppResult},
    middleware::admin::AdminUser,
    models::{category::Category, user::User},
    state::AppState,
    utils::password,
};

pub async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> AppResult<Json<Paginated<UserResponse>>> {
    let (page, limit, offset) = query.values()?;
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await?;
    let users = sqlx::query_as::<_, User>(
        "SELECT * FROM users ORDER BY created_at DESC, id LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(Paginated {
        items: users.into_iter().map(Into::into).collect(),
        page,
        per_page: limit as u64,
        total,
    }))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UserResponse>> {
    let user = find_user(&state, id).await?;
    Ok(Json(user.into()))
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
    body.validate()?;
    let email = body.email.trim().to_lowercase();
    let name = body.name.trim().to_string();
    let password = body.password;
    let password_hash = tokio::task::spawn_blocking(move || password::hash_password(&password))
        .await
        .map_err(|error| AppError::Internal(error.to_string()))??;

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, password_hash, name, role)
         VALUES ($1, $2, $3, $4)
         RETURNING *",
    )
    .bind(email)
    .bind(password_hash)
    .bind(name)
    .bind(body.role)
    .fetch_one(&state.db)
    .await
    .map_err(map_user_error)?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

pub async fn update_user(
    State(state): State<AppState>,
    Extension(admin): Extension<AdminUser>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    body.validate()?;
    if id == admin.id && body.role == Some(crate::models::user::UserRole::User) {
        return Err(AppError::Forbidden);
    }

    let password_hash = if let Some(password) = body.password {
        Some(
            tokio::task::spawn_blocking(move || password::hash_password(&password))
                .await
                .map_err(|error| AppError::Internal(error.to_string()))??,
        )
    } else {
        None
    };
    let email = body.email.map(|email| email.trim().to_lowercase());
    let name = body.name.map(|name| name.trim().to_string());

    let user = sqlx::query_as::<_, User>(
        "UPDATE users
         SET email = COALESCE($2, email),
             name = COALESCE($3, name),
             role = COALESCE($4, role),
             password_hash = COALESCE($5, password_hash),
             updated_at = now()
         WHERE id = $1
         RETURNING *",
    )
    .bind(id)
    .bind(email)
    .bind(name)
    .bind(body.role)
    .bind(password_hash)
    .fetch_optional(&state.db)
    .await
    .map_err(map_user_error)?
    .ok_or_else(|| AppError::NotFound("user not found".into()))?;

    Ok(Json(user.into()))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Extension(admin): Extension<AdminUser>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    if id == admin.id {
        return Err(AppError::Forbidden);
    }

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("user not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_categories(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> AppResult<Json<Paginated<CategoryResponse>>> {
    let (page, limit, offset) = query.values()?;
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM categories")
        .fetch_one(&state.db)
        .await?;
    let categories = sqlx::query_as::<_, Category>(
        "SELECT * FROM categories ORDER BY created_at DESC, id LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(Paginated {
        items: categories.into_iter().map(Into::into).collect(),
        page,
        per_page: limit as u64,
        total,
    }))
}

pub async fn get_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<CategoryResponse>> {
    let category = find_category(&state, id).await?;
    Ok(Json(category.into()))
}

pub async fn create_category(
    State(state): State<AppState>,
    Json(body): Json<CreateCategoryRequest>,
) -> AppResult<(StatusCode, Json<CategoryResponse>)> {
    body.validate()?;
    let name = body.name.trim().to_string();
    let slug = unique_slug(&state, &category_slug(&name), None).await?;
    let category = sqlx::query_as::<_, Category>(
        "INSERT INTO categories (name, slug, description)
         VALUES ($1, $2, $3)
         RETURNING *",
    )
    .bind(name)
    .bind(slug)
    .bind(body.description)
    .fetch_one(&state.db)
    .await
    .map_err(map_category_error)?;

    Ok((StatusCode::CREATED, Json(category.into())))
}

pub async fn update_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateCategoryRequest>,
) -> AppResult<Json<CategoryResponse>> {
    body.validate()?;
    let current = find_category(&state, id).await?;
    let current_name = current.name;
    let name = body
        .name
        .map(|name| name.trim().to_string())
        .unwrap_or_else(|| current_name.clone());
    let slug = if name != current_name {
        Some(unique_slug(&state, &category_slug(&name), Some(id)).await?)
    } else {
        None
    };
    let description = body.description.unwrap_or(current.description);

    let category = sqlx::query_as::<_, Category>(
        "UPDATE categories
         SET name = $2,
             slug = COALESCE($3, slug),
             description = $4,
             updated_at = now()
         WHERE id = $1
         RETURNING *",
    )
    .bind(id)
    .bind(name)
    .bind(slug)
    .bind(description)
    .fetch_optional(&state.db)
    .await
    .map_err(map_category_error)?
    .ok_or_else(|| AppError::NotFound("category not found".into()))?;

    Ok(Json(category.into()))
}

pub async fn delete_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let result = sqlx::query("DELETE FROM categories WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(map_category_error)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("category not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn find_user(state: &AppState, id: Uuid) -> AppResult<User> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".into()))
}

async fn find_category(state: &AppState, id: Uuid) -> AppResult<Category> {
    sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("category not found".into()))
}

async fn unique_slug(state: &AppState, base: &str, exclude_id: Option<Uuid>) -> AppResult<String> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1 FROM categories
            WHERE slug = $1 AND ($2::uuid IS NULL OR id <> $2)
         )",
    )
    .bind(base)
    .bind(exclude_id)
    .fetch_one(&state.db)
    .await?;

    if !exists {
        return Ok(base.to_string());
    }

    let suffix = Uuid::new_v4().simple().to_string();
    let candidate = format!("{base}-{}", &suffix[..8]);
    let still_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1 FROM categories
            WHERE slug = $1 AND ($2::uuid IS NULL OR id <> $2)
         )",
    )
    .bind(&candidate)
    .bind(exclude_id)
    .fetch_one(&state.db)
    .await?;
    if still_exists {
        return Err(AppError::Conflict(
            "could not generate a unique category slug".into(),
        ));
    }
    Ok(candidate)
}

fn map_user_error(error: sqlx::Error) -> AppError {
    match error {
        sqlx::Error::Database(ref db_error) if db_error.is_unique_violation() => {
            AppError::Conflict("email is already in use".into())
        }
        other => AppError::Database(other),
    }
}

fn map_category_error(error: sqlx::Error) -> AppError {
    match error {
        sqlx::Error::Database(ref db_error) if db_error.is_unique_violation() => {
            AppError::Conflict("category name or slug is already in use".into())
        }
        sqlx::Error::Database(ref db_error) if db_error.is_foreign_key_violation() => {
            AppError::Conflict("category is in use by one or more products".into())
        }
        other => AppError::Database(other),
    }
}
