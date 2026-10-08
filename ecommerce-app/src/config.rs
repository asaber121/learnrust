use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expires_hours: i64,
}

impl Config {
    pub fn from_env() -> Self {
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env");
        assert!(
            jwt_secret.len() >= 32,
            "JWT_SECRET must be at least 32 characters"
        );
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env"),
            jwt_secret,
            jwt_expires_hours: env::var("JWT_EXPIRES_HOURS")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(24),
        }
    }
}
