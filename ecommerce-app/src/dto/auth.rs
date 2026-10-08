use serde::{Deserialize, Serialize};

use crate::{dto::user::UserResponse, error::AppError};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

impl RegisterRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        let email = self.email.trim();
        if email.len() < 3 || !email.contains('@') {
            return Err(AppError::BadRequest("invalid email address".into()));
        }
        if self.password.len() < 8 {
            return Err(AppError::BadRequest(
                "password must be at least 8 characters".into(),
            ));
        }
        if self.name.trim().is_empty() {
            return Err(AppError::BadRequest("name is required".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}
