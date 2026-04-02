use super::model::{AuthUserEntity, LoginCredentialsEntity};
use crate::common::error::ServiceError;

use chrono::Utc;
use sqlx::PgPool;

/// Auth db for authentication-specific database operations
pub struct AuthRepository;

impl AuthRepository {
    /// Check user by username for authentication (only essential fields)
    pub async fn get_login_credentials(
        pool: &PgPool,
        username: &str,
    ) -> Result<Option<LoginCredentialsEntity>, ServiceError> {
        sqlx::query_as::<_, LoginCredentialsEntity>("SELECT * FROM get_login_credentials($1)")
            .bind(username)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "Database error in get_login_credentials, username={}: {:?}",
                    username,
                    e
                );
                ServiceError::DatabaseQueryFailed
            })
    }

    /// Find user by ID for authentication (returns AuthUserEntity)
    /// Optimized version using the helper function from 004_user_info_optimization.sql
    pub async fn get_user_by_id(
        pool: &PgPool,
        id: i64,
    ) -> Result<Option<AuthUserEntity>, ServiceError> {
        sqlx::query_as::<_, AuthUserEntity>("SELECT * FROM get_user_basic_info($1)")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error in get_user_by_id, user_id={}: {:?}", id, e);
                ServiceError::DatabaseQueryFailed
            })
    }

    /// Update last login timestamp
    pub async fn update_last_login(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        sqlx::query("UPDATE users SET last_login_at = $1, updated_at = $1 WHERE id = $2")
            .bind(Utc::now().naive_utc())
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error in update_last_login, user_id={}: {:?}", id, e);
                ServiceError::DatabaseQueryFailed
            })?;
        Ok(())
    }

    /// Get all permission keys for a
    /// user by user ID.
    /// Returns a list of permission strings (e.g., "system:user:list").
    pub async fn get_user_permissions(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<Vec<String>, ServiceError> {
        sqlx::query_scalar("SELECT get_user_permissions($1)")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "Database error in get_user_permissions, user_id={}: {:?}",
                    user_id,
                    e
                );
                ServiceError::DatabaseQueryFailed
            })
    }

    /// Increment failed login attempts for a user.
    /// When the count reaches 5, the account is automatically locked for 30 minutes.
    /// Note: does not change `status` — auto-lockout is tracked via `locked_until` only.
    pub async fn increment_failed_attempts(pool: &PgPool, user_id: i64) -> Result<(), ServiceError> {
        sqlx::query(
            r#"
            UPDATE users
            SET failed_login_attempts = failed_login_attempts + 1,
                locked_until = CASE
                    WHEN failed_login_attempts + 1 >= 5
                    THEN NOW() + INTERVAL '30 minutes'
                    ELSE locked_until
                END
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!(
                "Database error in increment_failed_attempts, user_id={}: {:?}",
                user_id,
                e
            );
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(())
    }

    /// Reset failed login attempts and remove any auto-lock after a successful login.
    pub async fn reset_failed_attempts(pool: &PgPool, user_id: i64) -> Result<(), ServiceError> {
        sqlx::query(
            "UPDATE users SET failed_login_attempts = 0, locked_until = NULL WHERE id = $1",
        )
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!(
                "Database error in reset_failed_attempts, user_id={}: {:?}",
                user_id,
                e
            );
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(())
    }

    pub async fn update_avatar(
        pool: &PgPool,
        user_id: i64,
        avatar_url: &str,
    ) -> Result<(), ServiceError> {
        sqlx::query("UPDATE users SET avatar_url = $1 WHERE id = $2")
            .bind(avatar_url)
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error in update_avatar, user_id={}: {:?}", user_id, e);
                ServiceError::DatabaseQueryFailed
            })?;
        Ok(())
    }
}
