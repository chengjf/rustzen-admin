use crate::common::error::ServiceError;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Active session record loaded from the database.
#[derive(Debug, sqlx::FromRow)]
pub struct UserSessionEntity {
    pub session_token: String,
    pub user_id: i64,
    pub expires_at: DateTime<Utc>,
}

/// Database-backed session store.
///
/// Each user has at most one active session row (enforced by UNIQUE on `user_id`).
/// The session token is the sole credential; JWT is no longer used.
pub struct SessionStore;

impl SessionStore {
    /// Create (or replace) a session for a user.
    ///
    /// Generates a new 64-character hex session token (256 bits of entropy),
    /// persists it, and returns the token to be handed to the client.
    pub async fn create(
        pool: &PgPool,
        user_id: i64,
        expires_at: DateTime<Utc>,
        client_ip: &str,
        user_agent: &str,
    ) -> Result<String, ServiceError> {
        let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());

        sqlx::query(
            r#"
            INSERT INTO user_sessions (session_token, user_id, expires_at, client_ip, user_agent, last_access_at)
            VALUES ($1, $2, $3, $4::inet, $5, NOW())
            ON CONFLICT (user_id) DO UPDATE SET
                session_token  = EXCLUDED.session_token,
                expires_at     = EXCLUDED.expires_at,
                last_access_at = NOW(),
                client_ip      = EXCLUDED.client_ip,
                user_agent     = EXCLUDED.user_agent
            "#,
        )
        .bind(&token)
        .bind(user_id)
        .bind(expires_at)
        .bind(client_ip)
        .bind(user_agent)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("SessionStore::create failed for user_id={}: {:?}", user_id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        tracing::debug!("Session created for user_id={}, expires={}", user_id, expires_at);

        // Opportunistically remove expired sessions on each login.
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            if let Err(e) = sqlx::query("DELETE FROM user_sessions WHERE expires_at < NOW()")
                .execute(&pool_clone)
                .await
            {
                tracing::warn!("Failed to clean up expired sessions: {:?}", e);
            }
        });

        Ok(token)
    }

    /// Look up an unexpired session by its token.
    ///
    /// Returns `None` if no matching session exists or if it has expired.
    pub async fn get_by_token(
        pool: &PgPool,
        token: &str,
    ) -> Result<Option<UserSessionEntity>, ServiceError> {
        sqlx::query_as::<_, UserSessionEntity>(
            "SELECT session_token, user_id, expires_at \
             FROM user_sessions \
             WHERE session_token = $1 AND expires_at > NOW()",
        )
        .bind(token)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("SessionStore::get_by_token failed: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })
    }

    /// Delete the session for the given user (called on logout, password change, etc.).
    pub async fn delete_by_user_id(pool: &PgPool, user_id: i64) -> Result<(), ServiceError> {
        sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "SessionStore::delete_by_user_id failed for user_id={}: {:?}",
                    user_id,
                    e
                );
                ServiceError::DatabaseQueryFailed
            })?;

        tracing::debug!("Session deleted for user_id={}", user_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    async fn superadmin_id(pool: &PgPool) -> i64 {
        sqlx::query_scalar::<_, i64>("SELECT id FROM users WHERE username = 'superadmin'")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[sqlx::test]
    async fn create_and_get_by_token_round_trip(pool: PgPool) {
        let user_id = superadmin_id(&pool).await;
        let expires_at = Utc::now() + chrono::Duration::hours(1);

        let token = SessionStore::create(&pool, user_id, expires_at, "127.0.0.1", "session-test")
            .await
            .unwrap();

        let session = SessionStore::get_by_token(&pool, &token).await.unwrap().unwrap();
        assert_eq!(session.user_id, user_id);
        assert_eq!(session.session_token, token);
        assert!(session.expires_at > Utc::now());
    }

    #[sqlx::test]
    async fn create_replaces_existing_session_for_same_user(pool: PgPool) {
        let user_id = superadmin_id(&pool).await;

        let first = SessionStore::create(
            &pool,
            user_id,
            Utc::now() + chrono::Duration::minutes(30),
            "127.0.0.1",
            "session-test-1",
        )
        .await
        .unwrap();
        let second = SessionStore::create(
            &pool,
            user_id,
            Utc::now() + chrono::Duration::hours(1),
            "127.0.0.2",
            "session-test-2",
        )
        .await
        .unwrap();

        assert_ne!(first, second);
        assert!(SessionStore::get_by_token(&pool, &first).await.unwrap().is_none());
        assert!(SessionStore::get_by_token(&pool, &second).await.unwrap().is_some());
    }

    #[sqlx::test]
    async fn get_by_token_returns_none_for_expired_session(pool: PgPool) {
        let user_id = superadmin_id(&pool).await;
        let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());

        sqlx::query(
            "INSERT INTO user_sessions (session_token, user_id, expires_at, client_ip, user_agent, last_access_at)
             VALUES ($1, $2, $3, '127.0.0.1', 'expired-test', NOW())",
        )
        .bind(&token)
        .bind(user_id)
        .bind(Utc::now() - chrono::Duration::minutes(1))
        .execute(&pool)
        .await
        .unwrap();

        assert!(SessionStore::get_by_token(&pool, &token).await.unwrap().is_none());
    }

    #[sqlx::test]
    async fn delete_by_user_id_removes_active_session(pool: PgPool) {
        let user_id = superadmin_id(&pool).await;
        let token = SessionStore::create(
            &pool,
            user_id,
            Utc::now() + chrono::Duration::hours(1),
            "127.0.0.1",
            "delete-test",
        )
        .await
        .unwrap();

        SessionStore::delete_by_user_id(&pool, user_id).await.unwrap();

        assert!(SessionStore::get_by_token(&pool, &token).await.unwrap().is_none());
    }
}
