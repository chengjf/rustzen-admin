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
