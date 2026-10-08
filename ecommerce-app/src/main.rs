mod config;
mod db;
mod dto;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;
mod state;
mod utils;

use axum::Router;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::{config::Config, state::AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = Config::from_env();

    let pool = db::connect(&config.database_url).await?;
    tracing::info!("connected to database");
    sqlx::migrate!("./migrations").run(&pool).await?;

    let addr = format!("{}:{}", config.host, config.port);
    let state = AppState { db: pool, config };

    let app = Router::new()
        .nest("/api/v1", routes::api_router(state.clone()))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
