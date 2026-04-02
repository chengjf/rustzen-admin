use crate::{
    common::error::{AppError, ServiceError},
    core::{extractor::CurrentUser, jwt, permission::PermissionService, session::SessionStore},
};

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;

/// JWT authentication middleware
///
/// Steps:
/// 1. Extract JWT from Authorization header
/// 2. Validate token and extract claims
/// 3. Inject CurrentUser and PgPool into request extensions
///
/// Note: Only handles authentication, not authorization
pub async fn auth_middleware(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let (mut parts, body) = request.into_parts();

    tracing::debug!("Auth for: {}", parts.uri.path());

    // Extract Bearer token from Authorization header
    let token = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| {
            tracing::debug!("Missing/invalid Authorization header for {}", parts.uri.path());
            AppError::from(ServiceError::InvalidToken)
        })?;

    // Verify JWT and extract claims
    let claims = jwt::verify_token(token).map_err(|e| {
        tracing::warn!("JWT verification failed for {}: {:?}", parts.uri.path(), e);
        ServiceError::InvalidToken
    })?;

    tracing::debug!(
        "JWT verified for user {} ({}) accessing {}",
        claims.user_id,
        claims.username,
        parts.uri.path()
    );

    // Check if session is still active in the permission cache.
    // This enforces token revocation: when a user logs out or is disabled,
    // the cache is cleared and subsequent requests are rejected even if the
    // JWT signature is still valid.
    if !PermissionService::is_session_active(claims.user_id) {
        // In-memory cache is empty (e.g. after server restart) — try to
        // restore the session from the database before rejecting the request.
        match SessionStore::get(&pool, claims.user_id).await {
            Ok(Some(session)) => {
                let permissions = session.permission_list();
                PermissionService::cache_user_permissions(claims.user_id, &permissions);
                tracing::info!(
                    "Restored session from DB for user_id={} ({}), {} permissions",
                    claims.user_id,
                    claims.username,
                    permissions.len()
                );
            }
            _ => {
                tracing::warn!(
                    "No active session for user_id={} ({}), rejecting request to {}",
                    claims.user_id,
                    claims.username,
                    parts.uri.path()
                );
                return Err(AppError::from(ServiceError::InvalidToken));
            }
        }
    }

    // Inject user and database pool into request extensions
    let current_user = CurrentUser::new(claims.user_id, claims.username.clone());
    parts.extensions.insert(current_user);
    parts.extensions.insert(pool);

    let request = Request::from_parts(parts, body);

    tracing::debug!("Auth completed for user {} ({})", claims.user_id, claims.username);

    Ok(next.run(request).await)
}
