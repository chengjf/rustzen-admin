use crate::{
    common::api::{ApiResponse, AppResult},
    core::{
        config::CONFIG,
        db::{create_default_pool, test_connection},
        password::PasswordUtils,
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
    http::{HeaderValue, Method, header},
    middleware,
    routing::get,
};
use serde_json::json;
use std::{env, net::SocketAddr};
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing;

const SUPERADMIN_USERNAME: &str = "superadmin";
const SUPERADMIN_EMAIL: &str = "superadmin@example.com";
const SUPERADMIN_REAL_NAME: &str = "超级管理员";
const SUPERADMIN_PASSWORD_PLACEHOLDER: &str = "__BOOTSTRAP_PASSWORD_REQUIRED__";

#[tracing::instrument(name = "create_server")]
pub async fn create_server() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Initializing database connection pool...");
    let pool = create_default_pool().await?;
    test_connection(&pool).await?;
    bootstrap_superadmin(&pool).await?;

    let app = build_app(pool);
    let app = app.into_make_service_with_connect_info::<SocketAddr>();

    let addr = format!("{}:{}", CONFIG.app_host, CONFIG.app_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server started, listening on http://{}", addr);

    axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;
    Ok(())
}

pub fn build_app(pool: sqlx::PgPool) -> Router {
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
    let cors_layer = build_cors_layer();

    Router::new()
        .route("/api/summary", get(summary))
        .route("/api/health", get(health))
        .nest("/api", public_api.merge(protected_api))
        .nest_service("/uploads", uploads_service)
        .layer(cors_layer)
        .with_state(pool)
        .fallback(crate::core::web_embed::web_embed_file_handler)
}

async fn health(
    axum::extract::State(pool): axum::extract::State<sqlx::PgPool>,
) -> AppResult<serde_json::Value> {
    test_connection(&pool).await?;
    Ok(ApiResponse::success(json!({"status": "ok", "database": "ok"})))
}

async fn summary() -> AppResult<serde_json::Value> {
    Ok(ApiResponse::success(json!({
        "message": "Welcome to rustzen-admin API",
        "description": "A backend management system built with Rust, Axum, SQLx, and PostgreSQL.",
        "github": "https://github.com/idaibin/rustzen-admin"
    })))
}

fn build_cors_layer() -> CorsLayer {
    let origins = env::var("RUSTZEN_CORS_ALLOWED_ORIGINS")
        .ok()
        .map(|value| {
            value
                .split(',')
                .filter_map(|item| {
                    let trimmed = item.trim();
                    (!trimmed.is_empty()).then(|| HeaderValue::from_str(trimmed).ok()).flatten()
                })
                .collect::<Vec<_>>()
        })
        .filter(|origins| !origins.is_empty())
        .unwrap_or_else(|| {
            vec![
                HeaderValue::from_static("http://localhost:5173"),
                HeaderValue::from_static("http://127.0.0.1:5173"),
            ]
        });

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }

    tracing::info!("Shutdown signal received, stopping server gracefully");
}

async fn bootstrap_superadmin(pool: &sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let existing: Option<(i64, String)> = sqlx::query_as(
        "SELECT id, password_hash FROM users WHERE username = $1 AND deleted_at IS NULL",
    )
    .bind(SUPERADMIN_USERNAME)
    .fetch_optional(pool)
    .await?;

    let superadmin_id = match existing {
        Some((user_id, password_hash)) => {
            if password_hash == SUPERADMIN_PASSWORD_PLACEHOLDER {
                let password = env::var("RUSTZEN_BOOTSTRAP_SUPERADMIN_PASSWORD")
                    .ok()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| PasswordUtils::generate_password(20));
                let password_hash = PasswordUtils::hash_password(&password)?;
                sqlx::query(
                    "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2",
                )
                .bind(password_hash)
                .bind(user_id)
                .execute(pool)
                .await?;
                tracing::warn!(
                    "Superadmin password initialized on startup. username={}, password={}",
                    SUPERADMIN_USERNAME,
                    password
                );
            }
            user_id
        }
        None => {
            let password = env::var("RUSTZEN_BOOTSTRAP_SUPERADMIN_PASSWORD")
                .ok()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| PasswordUtils::generate_password(20));
            let password_hash = PasswordUtils::hash_password(&password)?;
            let user_id: i64 = sqlx::query_scalar(
                "INSERT INTO users (username, email, password_hash, real_name, status, is_system)
                 VALUES ($1, $2, $3, $4, 1, TRUE)
                 RETURNING id",
            )
            .bind(SUPERADMIN_USERNAME)
            .bind(SUPERADMIN_EMAIL)
            .bind(password_hash)
            .bind(SUPERADMIN_REAL_NAME)
            .fetch_one(pool)
            .await?;
            tracing::warn!(
                "Superadmin user created on startup. username={}, password={}",
                SUPERADMIN_USERNAME,
                password
            );
            user_id
        }
    };

    let _ = superadmin_id;
    Ok(())
}
