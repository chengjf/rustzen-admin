use crate::common::error::{AppError, ServiceError};

use axum::{extract::FromRequestParts, http::request::Parts};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Current authenticated user info injected by auth middleware.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    /// User ID from database.
    pub user_id: i64,
    /// Username.
    pub username: String,
    /// Permission codes loaded from DB at authentication time.
    #[serde(default)]
    pub permissions: HashSet<String>,
}

impl CurrentUser {
    pub fn new(user_id: i64, username: String, permissions: HashSet<String>) -> Self {
        Self { user_id, username, permissions }
    }
}

/// Axum extractor for CurrentUser.
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            parts.extensions.get::<CurrentUser>().cloned().ok_or_else(|| {
                tracing::error!(
                    "CurrentUser not found - auth middleware missing or user not authenticated"
                );
                AppError::from(ServiceError::InvalidToken)
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn extractor_returns_current_user_from_extensions() {
        let current_user = CurrentUser::new(
            42,
            "tester".to_string(),
            ["system:user:list".to_string()].into_iter().collect(),
        );
        let (mut parts, _) = Request::builder().uri("/test").body(()).unwrap().into_parts();
        parts.extensions.insert(current_user.clone());

        let extracted = CurrentUser::from_request_parts(&mut parts, &()).await.unwrap();
        assert_eq!(extracted.user_id, current_user.user_id);
        assert_eq!(extracted.username, current_user.username);
        assert_eq!(extracted.permissions, current_user.permissions);
    }

    #[tokio::test]
    async fn extractor_rejects_when_current_user_is_missing() {
        let (mut parts, _) = Request::builder().uri("/test").body(()).unwrap().into_parts();

        let error = CurrentUser::from_request_parts(&mut parts, &()).await.unwrap_err();
        let response = error.into_response();

        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }
}
