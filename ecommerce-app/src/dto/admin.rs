use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{category::Category, user::UserRole},
};

pub const DEFAULT_PAGE: u64 = 1;
pub const DEFAULT_PER_PAGE: u64 = 20;
pub const MAX_PER_PAGE: u64 = 100;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

impl PaginationQuery {
    pub fn values(&self) -> Result<(u64, i64, i64), AppError> {
        let page = self.page.unwrap_or(DEFAULT_PAGE);
        let per_page = self.per_page.unwrap_or(DEFAULT_PER_PAGE).min(MAX_PER_PAGE);
        if page == 0 {
            return Err(AppError::BadRequest("page must be at least 1".into()));
        }
        if per_page == 0 {
            return Err(AppError::BadRequest("per_page must be at least 1".into()));
        }
        let limit =
            i64::try_from(per_page).map_err(|_| AppError::BadRequest("invalid per_page".into()))?;
        let offset = page
            .checked_sub(1)
            .and_then(|value| value.checked_mul(per_page))
            .and_then(|value| i64::try_from(value).ok())
            .ok_or_else(|| AppError::BadRequest("page is too large".into()))?;
        Ok((page, limit, offset))
    }
}

#[derive(Debug, Serialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub page: u64,
    pub per_page: u64,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub role: UserRole,
}

impl CreateUserRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_email(&self.email)?;
        validate_name(&self.name)?;
        validate_password(&self.password)
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub password: Option<String>,
    pub name: Option<String>,
    pub role: Option<UserRole>,
}

impl UpdateUserRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.email.is_none()
            && self.password.is_none()
            && self.name.is_none()
            && self.role.is_none()
        {
            return Err(AppError::BadRequest(
                "at least one user field must be provided".into(),
            ));
        }
        if let Some(email) = &self.email {
            validate_email(email)?;
        }
        if let Some(name) = &self.name {
            validate_name(name)?;
        }
        if let Some(password) = &self.password {
            validate_password(password)?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
}

impl CreateCategoryRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_name(&self.name)?;
        if let Some(description) = &self.description {
            validate_description(description)?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub description: Option<Option<String>>,
}

impl UpdateCategoryRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.name.is_none() && self.description.is_none() {
            return Err(AppError::BadRequest(
                "at least one category field must be provided".into(),
            ));
        }
        if let Some(name) = &self.name {
            validate_name(name)?;
        }
        if let Some(Some(description)) = &self.description {
            validate_description(description)?;
        }
        Ok(())
    }
}

fn deserialize_nullable<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

fn validate_email(email: &str) -> Result<(), AppError> {
    let email = email.trim();
    if email.len() < 3 || email.len() > 254 || !email.contains('@') {
        return Err(AppError::BadRequest("invalid email address".into()));
    }
    Ok(())
}

fn validate_name(name: &str) -> Result<(), AppError> {
    if name.trim().is_empty() || name.trim().len() > 200 {
        return Err(AppError::BadRequest(
            "name must be between 1 and 200 characters".into(),
        ));
    }
    Ok(())
}

fn validate_description(description: &str) -> Result<(), AppError> {
    if description.len() > 5000 {
        return Err(AppError::BadRequest(
            "description must be at most 5000 characters".into(),
        ));
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < 8 || password.len() > 128 {
        return Err(AppError::BadRequest(
            "password must be between 8 and 128 characters".into(),
        ));
    }
    Ok(())
}

pub fn category_slug(name: &str) -> String {
    let mut slug = String::new();
    let mut previous_hyphen = false;
    for character in name.trim().to_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            previous_hyphen = false;
        } else if !previous_hyphen && !slug.is_empty() {
            slug.push('-');
            previous_hyphen = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "category".into()
    } else {
        slug
    }
}

#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Category> for CategoryResponse {
    fn from(category: Category) -> Self {
        Self {
            id: category.id,
            name: category.name,
            slug: category.slug,
            description: category.description,
            created_at: category.created_at,
            updated_at: category.updated_at,
        }
    }
}
