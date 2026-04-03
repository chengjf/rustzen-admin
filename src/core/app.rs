use crate::{
    common::api::{ApiResponse, AppResult},
    core::{
        config::CONFIG,
        db::{create_default_pool, test_connection},
    },
    features::{
        auth::api::{protected_auth_routes, public_auth_routes},
        dashboard::api::dashboard_routes,
        system::system_routes,
    },
    middleware::{auth::auth_middleware, log::log_middleware},
};

use axum::{
    Router,
    middleware,
    routing::get,
};
use serde_json::json;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing;

#[tracing::instrument(name = "create_server")]
pub async fn create_server() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Initializing database connection pool...");
    let pool = create_default_pool().await?;
    test_connection(&pool).await?;

    tracing::info!("Setting up API routes...");
    let protected_api = Router::new()
        .nest("/auth", protected_auth_routes())
        .nest("/dashboard", dashboard_routes())
        .nest("/system", system_routes())
        .route_layer(middleware::from_fn_with_state(pool.clone(), log_middleware))
        .route_layer(middleware::from_fn_with_state(pool.clone(), auth_middleware));

    let public_api = Router::new().nest("/auth", public_auth_routes());

    let uploads_service = ServeDir::new("uploads")
        .not_found_service(ServeDir::new("uploads").append_index_html_on_directories(true));

    let app = Router::new()
        .route("/api/summary", get(summary))
        .route("/api/health", get(health))
        .nest("/api", public_api.merge(protected_api))
        .nest_service("/uploads", uploads_service)
        .with_state(pool)
        .fallback(crate::core::web_embed::web_embed_file_handler);

    let app = app.into_make_service_with_connect_info::<SocketAddr>();

    let addr = format!("{}:{}", CONFIG.app_host, CONFIG.app_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server started, listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> AppResult<serde_json::Value> {
    Ok(ApiResponse::success(json!({"status": "ok"})))
}

async fn summary() -> AppResult<serde_json::Value> {
    Ok(ApiResponse::success(json!({
        "message": "Welcome to rustzen-admin API",
        "description": "A backend management system built with Rust, Axum, SQLx, and PostgreSQL.",
        "github": "https://github.com/idaibin/rustzen-admin"
    })))
}
