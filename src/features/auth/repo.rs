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
    pub async fn increment_failed_attempts(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<(), ServiceError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::password::PasswordUtils,
        features::system::{
            role::repo::RoleRepository,
            user::repo::{CreateUserCommand, UserRepository},
        },
    };
    use sqlx::PgPool;

    /// Seeded by 0105_seed.sql: username="superadmin", status=1 (Normal), is_system=true
    #[sqlx::test]
    async fn get_credentials_returns_some_for_existing_user(pool: PgPool) {
        let result = AuthRepository::get_login_credentials(&pool, "superadmin").await;
        assert!(result.is_ok(), "query should not fail");
        let credentials = result.unwrap();
        assert!(credentials.is_some(), "superadmin should be found");
        let creds = credentials.unwrap();
        assert_eq!(creds.status, 1); // UserStatus::Normal
        assert!(creds.is_system);
    }

    #[sqlx::test]
    async fn get_credentials_returns_none_for_missing_user(pool: PgPool) {
        let result = AuthRepository::get_login_credentials(&pool, "nobody_xyz_404").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[sqlx::test]
    async fn get_user_by_id_returns_some_for_superadmin(pool: PgPool) {
        // First get the id via a raw query since we don't know it upfront
        let row: (i64,) = sqlx::query_as("SELECT id FROM users WHERE username = 'superadmin'")
            .fetch_one(&pool)
            .await
            .expect("superadmin must exist");

        let result = AuthRepository::get_user_by_id(&pool, row.0).await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().username, "superadmin");
    }

    #[sqlx::test]
    async fn get_user_by_id_returns_none_for_missing(pool: PgPool) {
        let result = AuthRepository::get_user_by_id(&pool, 999_999).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[sqlx::test]
    async fn reset_and_increment_failed_attempts(pool: PgPool) {
        let (user_id,): (i64,) =
            sqlx::query_as("SELECT id FROM users WHERE username = 'superadmin'")
                .fetch_one(&pool)
                .await
                .unwrap();

        // Reset first to get a clean baseline
        AuthRepository::reset_failed_attempts(&pool, user_id).await.unwrap();

        let (before,): (i16,) =
            sqlx::query_as("SELECT failed_login_attempts FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(before, 0);

        // Increment twice
        AuthRepository::increment_failed_attempts(&pool, user_id).await.unwrap();
        AuthRepository::increment_failed_attempts(&pool, user_id).await.unwrap();

        let (after,): (i16,) =
            sqlx::query_as("SELECT failed_login_attempts FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(after, 2);

        // Reset clears it
        AuthRepository::reset_failed_attempts(&pool, user_id).await.unwrap();
        let (cleared,): (i16,) =
            sqlx::query_as("SELECT failed_login_attempts FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(cleared, 0);
    }

    #[sqlx::test]
    async fn get_user_permissions_excludes_disabled_wildcard_menu_for_superadmin(pool: PgPool) {
        let (user_id,): (i64,) =
            sqlx::query_as("SELECT id FROM users WHERE username = 'superadmin'")
                .fetch_one(&pool)
                .await
                .unwrap();

        let permissions = AuthRepository::get_user_permissions(&pool, user_id)
            .await
            .expect("permissions query should succeed");

        assert!(permissions.is_empty());
    }

    #[sqlx::test]
    async fn get_user_permissions_returns_assigned_menu_codes_for_normal_user(pool: PgPool) {
        let role_id =
            RoleRepository::create(&pool, "权限角色", "PERMISSION_ROLE", None, 1, 0, &[3, 4])
                .await
                .unwrap();

        let user_id = UserRepository::create_user(
            &pool,
            &CreateUserCommand {
                username: "perm_user".to_string(),
                email: "perm_user@example.com".to_string(),
                password_hash: PasswordUtils::hash_password("Perm@Test1").unwrap(),
                real_name: None,
                status: Some(1),
                role_ids: vec![role_id],
            },
        )
        .await
        .unwrap();

        let permissions = AuthRepository::get_user_permissions(&pool, user_id).await.unwrap();

        assert!(permissions.iter().any(|permission| permission == "system:user:list"));
        assert!(permissions.iter().any(|permission| permission == "system:user:create"));
        assert!(!permissions.iter().any(|permission| permission == "*"));
    }

    #[sqlx::test]
    async fn update_last_login_sets_timestamp(pool: PgPool) {
        let (user_id,): (i64,) =
            sqlx::query_as("SELECT id FROM users WHERE username = 'superadmin'")
                .fetch_one(&pool)
                .await
                .unwrap();

        sqlx::query("UPDATE users SET last_login_at = NULL WHERE id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .unwrap();

        AuthRepository::update_last_login(&pool, user_id).await.expect("update should succeed");

        let (last_login_at,): (Option<chrono::NaiveDateTime>,) =
            sqlx::query_as("SELECT last_login_at FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(last_login_at.is_some());
    }

    #[sqlx::test]
    async fn increment_failed_attempts_locks_user_after_five_attempts(pool: PgPool) {
        let (user_id,): (i64,) =
            sqlx::query_as("SELECT id FROM users WHERE username = 'superadmin'")
                .fetch_one(&pool)
                .await
                .unwrap();

        AuthRepository::reset_failed_attempts(&pool, user_id).await.unwrap();

        for _ in 0..5 {
            AuthRepository::increment_failed_attempts(&pool, user_id)
                .await
                .expect("increment should succeed");
        }

        let (attempts, locked_until): (i16, Option<chrono::DateTime<chrono::Utc>>) =
            sqlx::query_as("SELECT failed_login_attempts, locked_until FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(attempts, 5);
        assert!(locked_until.is_some());
        assert!(locked_until.unwrap() > chrono::Utc::now());
    }

    #[sqlx::test]
    async fn update_avatar_persists_avatar_url(pool: PgPool) {
        let (user_id,): (i64,) =
            sqlx::query_as("SELECT id FROM users WHERE username = 'superadmin'")
                .fetch_one(&pool)
                .await
                .unwrap();

        let avatar_url = "/uploads/avatars/test-avatar.png";
        AuthRepository::update_avatar(&pool, user_id, avatar_url)
            .await
            .expect("avatar update should succeed");

        let (stored_avatar_url,): (Option<String>,) =
            sqlx::query_as("SELECT avatar_url FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(stored_avatar_url.as_deref(), Some(avatar_url));
    }
}
