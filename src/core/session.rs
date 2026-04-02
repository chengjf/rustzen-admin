use crate::common::error::ServiceError;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// Persisted session record loaded from the database.
#[derive(Debug, sqlx::FromRow)]
pub struct UserSessionEntity {
    pub user_id: i64,
    /// JSON array of permission code strings, e.g. ["system:user:list", ...]
    pub permissions: serde_json::Value,
    pub is_system: bool,
    pub expires_at: DateTime<Utc>,
}

impl UserSessionEntity {
    /// Deserialise the JSONB permission array into a `Vec<String>`.
    pub fn permission_list(&self) -> Vec<String> {
        serde_json::from_value(self.permissions.clone()).unwrap_or_default()
    }
}

/// Database-backed session store.
///
/// Each user has at most one active session row (enforced by UNIQUE on
/// `user_id`).  The in-memory `PermissionCacheManager` is the primary
/// authority during normal operation; this store is the persistent fallback
/// used to rebuild the cache after a server restart.
pub struct SessionStore;

impl SessionStore {
    /// Insert or update the session for a user.
    ///
    /// Called at login time so that the session survives a server restart.
    pub async fn upsert(
        pool: &PgPool,
        user_id: i64,
        is_system: bool,
        permissions: &[String],
        expires_at: DateTime<Utc>,
    ) -> Result<(), ServiceError> {
        let permissions_json = serde_json::to_value(permissions)
            .unwrap_or(serde_json::Value::Array(vec![]));

        sqlx::query(
            r#"
            INSERT INTO user_sessions (user_id, permissions, is_system, expires_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id) DO UPDATE SET
                permissions = EXCLUDED.permissions,
                is_system   = EXCLUDED.is_system,
                expires_at  = EXCLUDED.expires_at,
                updated_at  = NOW()
            "#,
        )
        .bind(user_id)
        .bind(permissions_json)
        .bind(is_system)
        .bind(expires_at)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("SessionStore::upsert failed for user_id={}: {:?}", user_id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        tracing::debug!("Session persisted for user_id={}, expires={}", user_id, expires_at);

        // Opportunistically remove expired sessions from other users on each login.
        // This keeps the table small without a dedicated background job.
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            if let Err(e) =
                sqlx::query("DELETE FROM user_sessions WHERE expires_at < NOW()")
                    .execute(&pool_clone)
                    .await
            {
                tracing::warn!("Failed to clean up expired sessions: {:?}", e);
            }
        });

        Ok(())
    }

    /// Fetch an unexpired session for the given user.
    ///
    /// Returns `None` if no session exists or if it has already expired.
    pub async fn get(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<Option<UserSessionEntity>, ServiceError> {
        sqlx::query_as::<_, UserSessionEntity>(
            "SELECT user_id, permissions, is_system, expires_at \
             FROM user_sessions \
             WHERE user_id = $1 AND expires_at > NOW()",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("SessionStore::get failed for user_id={}: {:?}", user_id, e);
            ServiceError::DatabaseQueryFailed
        })
    }

    /// Delete the session for the given user (called on logout).
    pub async fn delete(pool: &PgPool, user_id: i64) -> Result<(), ServiceError> {
        sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| {
                tracing::error!("SessionStore::delete failed for user_id={}: {:?}", user_id, e);
                ServiceError::DatabaseQueryFailed
            })?;

        tracing::debug!("Session deleted for user_id={}", user_id);
        Ok(())
    }
}
