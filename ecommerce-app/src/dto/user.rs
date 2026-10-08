use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::models::user::{User, UserRole};

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            name: u.name,
            role: u.role,
            created_at: u.created_at,
        }
    }
}
