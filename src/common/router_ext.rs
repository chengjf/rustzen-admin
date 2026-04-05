use crate::{
    common::error::{AppError, ServiceError},
    core::{extractor::CurrentUser, permission::PermissionsCheck},
};

use axum::{Router, extract::Request, middleware::Next, response::Response, routing::MethodRouter};
use sqlx::PgPool;

/// Router extension for permission-based routing.
pub trait RouterExt<S> {
    fn route_with_permission(
        self,
        path: &str,
        method_router: MethodRouter<S>,
        permissions_check: PermissionsCheck,
    ) -> Self;
}

impl RouterExt<PgPool> for Router<PgPool> {
    fn route_with_permission(
        self,
        path: &str,
        method_router: MethodRouter<PgPool>,
        permissions_check: PermissionsCheck,
    ) -> Self {
        tracing::debug!(
            "Registering route '{}' with permission: {}",
            path,
            permissions_check.description()
        );

        self.route(
            path,
            method_router.layer(axum::middleware::from_fn(move |req: Request, next: Next| {
                let permissions_check = permissions_check.clone();
                async move { permission_middleware(req, next, permissions_check).await }
            })),
        )
    }
}

/// Permission validation middleware.
///
/// Reads `CurrentUser.permissions` (already loaded from DB by `auth_middleware`)
/// and checks them against `permissions_check` synchronously — no extra DB round-trip.
async fn permission_middleware(
    request: Request,
    next: Next,
    permissions_check: PermissionsCheck,
) -> Result<Response, AppError> {
    let current_user = request.extensions().get::<CurrentUser>().cloned().ok_or_else(|| {
        tracing::error!("CurrentUser not found - auth middleware missing?");
        AppError::from(ServiceError::InvalidToken)
    })?;

    let has_permission = permissions_check.check(&current_user.permissions);

    if !has_permission {
        tracing::warn!(
            "Permission denied: user_id={} ({}) lacks: {}",
            current_user.user_id,
            current_user.username,
            permissions_check.description()
        );
        return Err(AppError::from(ServiceError::PermissionDenied));
    }

    tracing::debug!(
        "Permission granted for user_id={} ({})",
        current_user.user_id,
        permissions_check.description()
    );

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::extractor::CurrentUser;
    use axum::{Json, middleware, response::IntoResponse, routing::get};
    use axum_test::TestServer;
    use serde_json::json;
    use sqlx::PgPool;
    use std::collections::HashSet;

    async fn ok_handler() -> impl IntoResponse {
        Json(json!({ "ok": true }))
    }

    fn user_with_permissions(codes: &[&str]) -> CurrentUser {
        CurrentUser::new(
            7,
            "router-ext-user".to_string(),
            codes.iter().map(|code| (*code).to_string()).collect::<HashSet<_>>(),
        )
    }

    fn server_with_check(
        pool: PgPool,
        check: PermissionsCheck,
        current_user: Option<CurrentUser>,
    ) -> TestServer {
        let router = Router::new()
            .route_with_permission("/protected", get(ok_handler), check)
            .layer(middleware::from_fn(move |mut req: Request, next: Next| {
                let current_user = current_user.clone();
                async move {
                    if let Some(cu) = current_user {
                        req.extensions_mut().insert(cu);
                    }
                    Ok::<_, AppError>(next.run(req).await)
                }
            }))
            .with_state(pool);

        TestServer::new(router).unwrap()
    }

    // ── Single ──────────────────────────────────────────────────────

    /// Single: user holds the required permission → 200.
    #[sqlx::test]
    async fn route_with_permission_allows_user_with_required_permission(pool: PgPool) {
        let server = server_with_check(
            pool,
            PermissionsCheck::Single("system:user:list"),
            Some(user_with_permissions(&["system:user:list"])),
        );
        let response = server.get("/protected").await;
        response.assert_status_ok();
        assert_eq!(response.json::<serde_json::Value>()["ok"], true);
    }

    /// Single: user lacks the required permission → 403.
    #[sqlx::test]
    async fn route_with_permission_rejects_user_without_required_permission(pool: PgPool) {
        let server = server_with_check(
            pool,
            PermissionsCheck::Single("system:user:list"),
            Some(user_with_permissions(&["system:role:list"])),
        );
        let response = server.get("/protected").await;
        response.assert_status(axum::http::StatusCode::FORBIDDEN);
    }

    /// Single: no CurrentUser extension at all → 401.
    #[sqlx::test]
    async fn route_with_permission_rejects_when_current_user_missing(pool: PgPool) {
        let server =
            server_with_check(pool, PermissionsCheck::Single("system:user:list"), None);
        let response = server.get("/protected").await;
        response.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    }

    // ── Any ─────────────────────────────────────────────────────────

    /// Any: user holds one of the listed permissions → 200.
    #[sqlx::test]
    async fn route_with_any_permission_grants_when_one_matches(pool: PgPool) {
        let server = server_with_check(
            pool,
            PermissionsCheck::Any(vec!["system:user:create", "system:user:delete"]),
            Some(user_with_permissions(&["system:user:delete"])),
        );
        let response = server.get("/protected").await;
        response.assert_status_ok();
    }

    /// Any: user holds none of the listed permissions → 403.
    #[sqlx::test]
    async fn route_with_any_permission_denies_when_none_match(pool: PgPool) {
        let server = server_with_check(
            pool,
            PermissionsCheck::Any(vec!["system:user:create", "system:user:delete"]),
            Some(user_with_permissions(&["system:role:list"])),
        );
        let response = server.get("/protected").await;
        response.assert_status(axum::http::StatusCode::FORBIDDEN);
    }

    // ── All ──────────────────────────────────────────────────────────

    /// All: user holds every required permission → 200.
    #[sqlx::test]
    async fn route_with_all_permissions_grants_when_all_present(pool: PgPool) {
        let server = server_with_check(
            pool,
            PermissionsCheck::All(vec!["system:user:create", "system:user:delete"]),
            Some(user_with_permissions(&["system:user:create", "system:user:delete"])),
        );
        let response = server.get("/protected").await;
        response.assert_status_ok();
    }

    /// All: user is missing one required permission → 403.
    #[sqlx::test]
    async fn route_with_all_permissions_denies_when_one_missing(pool: PgPool) {
        let server = server_with_check(
            pool,
            PermissionsCheck::All(vec!["system:user:create", "system:user:delete"]),
            Some(user_with_permissions(&["system:user:create"])), // missing delete
        );
        let response = server.get("/protected").await;
        response.assert_status(axum::http::StatusCode::FORBIDDEN);
    }
}
