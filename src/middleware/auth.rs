use crate::{
    common::error::{AppError, ServiceError},
    core::{extractor::CurrentUser, session::SessionStore},
};

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;
use std::collections::HashSet;

/// Session-based authentication middleware.
///
/// Steps:
/// 1. Extract the opaque session token from the Authorization header.
/// 2. Look up the session in the database (rejects expired sessions).
/// 3. Verify the user's current status (disabled/locked users are rejected immediately).
/// 4. Load the user's permissions fresh from the database.
/// 5. Inject `CurrentUser` (with permissions) and `PgPool` into request extensions.
pub async fn auth_middleware(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let (mut parts, body) = request.into_parts();

    tracing::debug!("Auth for: {}", parts.uri.path());

    // Extract Bearer token from Authorization header.
    let token = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| {
            tracing::debug!("Missing/invalid Authorization header for {}", parts.uri.path());
            AppError::from(ServiceError::InvalidToken)
        })?;

    // Look up session by token.
    let session = SessionStore::get_by_token(&pool, token).await?.ok_or_else(|| {
        tracing::debug!("No active session for token on {}", parts.uri.path());
        AppError::from(ServiceError::InvalidToken)
    })?;

    let user_id = session.user_id;

    // Fetch user status, username, and is_system flag in one query.
    let user_row = sqlx::query!(
        "SELECT username, status, locked_until, is_system \
         FROM users \
         WHERE id = $1 AND deleted_at IS NULL",
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch user status for user_id={}: {:?}", user_id, e);
        AppError::from(ServiceError::DatabaseQueryFailed)
    })?
    .ok_or_else(|| {
        tracing::warn!("User user_id={} not found or deleted", user_id);
        AppError::from(ServiceError::InvalidToken)
    })?;

    // Reject disabled / locked users immediately.
    if user_row.status != Some(1) {
        tracing::warn!(
            "User user_id={} has non-active status={:?}, rejecting",
            user_id,
            user_row.status
        );
        return Err(AppError::from(ServiceError::InvalidToken));
    }
    if let Some(locked_until) = user_row.locked_until {
        if locked_until > chrono::Utc::now() {
            tracing::warn!("User user_id={} is temporarily locked", user_id);
            return Err(AppError::from(ServiceError::InvalidToken));
        }
    }

    // Load permissions fresh from the database on every request.
    let permissions: HashSet<String> = if user_row.is_system.unwrap_or(false) {
        let mut s = HashSet::new();
        s.insert("*".to_string());
        s
    } else {
        let perms: Vec<String> = sqlx::query_scalar("SELECT get_user_permissions($1)")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to load permissions for user_id={}: {:?}", user_id, e);
                AppError::from(ServiceError::DatabaseQueryFailed)
            })?;
        perms.into_iter().collect()
    };

    tracing::debug!(
        "Auth OK for user_id={} ({}) on {}, {} permissions",
        user_id,
        user_row.username,
        parts.uri.path(),
        permissions.len()
    );

    let current_user = CurrentUser::new(user_id, user_row.username, permissions);
    parts.extensions.insert(current_user);
    parts.extensions.insert(pool);

    Ok(next.run(Request::from_parts(parts, body)).await)
}
