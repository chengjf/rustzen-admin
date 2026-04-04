use super::model::UserWithRolesEntity;
use crate::{common::error::ServiceError, features::auth::model::UserStatus};

use chrono::Utc;
use sqlx::{PgPool, QueryBuilder};

/// User db for database operations
pub struct UserRepository;

#[derive(Debug, Clone)]
pub struct UserListQuery {
    pub username: Option<String>,
    pub status: Option<i16>,
    pub real_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateUserCommand {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub real_name: Option<String>,
    pub status: Option<i16>,
    pub role_ids: Vec<i64>,
}

impl UserRepository {
    fn format_query(query: &UserListQuery, query_builder: &mut QueryBuilder<'_, sqlx::Postgres>) {
        if let Some(username) = &query.username {
            if !username.trim().is_empty() {
                query_builder.push(" AND username ILIKE ").push_bind(format!("%{}%", username));
            }
        }
        if let Some(real_name) = &query.real_name {
            if !real_name.trim().is_empty() {
                query_builder.push(" AND real_name ILIKE ").push_bind(format!("%{}%", real_name));
            }
        }
        if let Some(email) = &query.email {
            if !email.trim().is_empty() {
                query_builder.push(" AND email ILIKE ").push_bind(format!("%{}%", email));
            }
        }
        if let Some(status) = query.status {
            query_builder.push(" AND effective_status = ").push_bind(status);
        }
    }

    /// Count users matching filters
    async fn count_users(pool: &PgPool, query: &UserListQuery) -> Result<i64, ServiceError> {
        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM user_with_roles WHERE 1=1");

        Self::format_query(&query, &mut query_builder);

        let count: (i64,) = query_builder.build_query_as().fetch_one(pool).await.map_err(|e| {
            tracing::error!("Database error counting users: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        tracing::info!("user count: {:?}", count);

        Ok(count.0)
    }

    /// Find users with pagination and filters
    pub async fn find_with_pagination(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        query: UserListQuery,
    ) -> Result<(Vec<UserWithRolesEntity>, i64), ServiceError> {
        tracing::debug!("Finding users with pagination and filters: {:?}", query);
        let total = Self::count_users(pool, &query).await?;
        if total == 0 {
            return Ok((Vec::new(), total));
        }

        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT * FROM user_with_roles WHERE 1=1");

        Self::format_query(&query, &mut query_builder);

        query_builder.push(" ORDER BY created_at DESC");
        query_builder.push(" LIMIT ").push_bind(limit);
        query_builder.push(" OFFSET ").push_bind(offset);

        let users = query_builder.build_query_as().fetch_all(pool).await.map_err(|e| {
            tracing::error!("Database error in user_with_roles pagination: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok((users, total))
    }

    /// Find users for dropdown options
    pub async fn find_options(
        pool: &PgPool,
        status: Option<i16>, // 1, 2, or None (all users)
        q: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(i64, String)>, ServiceError> {
        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
            "SELECT id, COALESCE(real_name, username) as display_name \
             FROM users WHERE deleted_at IS NULL",
        );

        if let Some(status_val) = status {
            query_builder.push(" AND status = ").push_bind(status_val);
        }

        if let Some(search_term) = q {
            if !search_term.trim().is_empty() {
                let pattern = format!("%{}%", search_term);
                query_builder.push(" AND (username ILIKE ").push_bind(pattern.clone());
                query_builder.push(" OR real_name ILIKE ").push_bind(pattern);
                query_builder.push(")");
            }
        }

        query_builder.push(" ORDER BY display_name");

        if let Some(limit_val) = limit {
            query_builder.push(" LIMIT ").push_bind(limit_val);
        }

        let result = query_builder.build_query_as().fetch_all(pool).await.map_err(|e| {
            tracing::error!("Database error finding user options: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(result)
    }

    /// Find user by ID (returns None if not found)
    pub async fn find_by_id(
        pool: &PgPool,
        id: i64,
    ) -> Result<Option<UserWithRolesEntity>, ServiceError> {
        let result =
            sqlx::query_as::<_, UserWithRolesEntity>("SELECT * FROM user_with_roles WHERE id = $1")
                .bind(id)
                .fetch_optional(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database error finding user by ID {}: {:?}", id, e);
                    ServiceError::DatabaseQueryFailed
                })?;

        Ok(result)
    }

    /// Get user by ID (returns NotFound error if not found)
    pub async fn get_by_id(pool: &PgPool, id: i64) -> Result<UserWithRolesEntity, ServiceError> {
        Self::find_by_id(pool, id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("User id: {}", id)))
    }

    /// Create new user with optional roles (unified method)
    pub async fn create_user(pool: &PgPool, cmd: &CreateUserCommand) -> Result<i64, ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for user creation: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        // Create user
        let user_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO users (username, email, password_hash, real_name, status, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id",
        )
        .bind(&cmd.username)
        .bind(&cmd.email)
        .bind(&cmd.password_hash)
        .bind(cmd.real_name.as_deref())
        .bind(cmd.status.unwrap_or(UserStatus::Normal as i16))
        .bind(Utc::now().naive_utc())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error creating user '{}': {:?}", cmd.username, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Self::insert_user_roles(&mut tx, user_id, &cmd.role_ids).await?;

        tx.commit().await.map_err(|e| {
            tracing::error!("Database error committing user creation transaction: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(user_id)
    }

    /// Update existing user
    pub async fn update_user(
        pool: &PgPool,
        id: i64,
        email: &str,
        real_name: Option<&str>,
        role_ids: &[i64],
    ) -> Result<i64, ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for user update: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let user_id = sqlx::query_scalar::<_, i64>(
            "UPDATE users
             SET email = $1, real_name = $2, updated_at = $3
             WHERE id = $4 AND deleted_at IS NULL
             RETURNING id",
        )
        .bind(email)
        .bind(real_name)
        .bind(Utc::now().naive_utc())
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error updating user ID {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        if let Some(id) = user_id {
            Self::insert_user_roles(&mut tx, id, role_ids).await?;
            tx.commit().await.map_err(|e| {
                tracing::error!("Database error committing user update transaction: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;
            Ok(id)
        } else {
            Err(ServiceError::NotFound(format!("User id: {}", id)))
        }
    }

    /// Soft delete user
    pub async fn soft_delete(pool: &PgPool, id: i64) -> Result<bool, ServiceError> {
        let result =
            sqlx::query("UPDATE users SET deleted_at = $1 WHERE id = $2 AND deleted_at IS NULL")
                .bind(Utc::now().naive_utc())
                .bind(id)
                .execute(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database error soft deleting user ID {}: {:?}", id, e);
                    ServiceError::DatabaseQueryFailed
                })?;

        Ok(result.rows_affected() > 0)
    }

    /// Set user roles (replace all existing roles)
    pub async fn insert_user_roles(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: i64,
        role_ids: &[i64],
    ) -> Result<(), ServiceError> {
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                tracing::error!("Database error deleting existing user_roles: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

        if role_ids.is_empty() {
            return Ok(());
        }
        let now = Utc::now().naive_utc();
        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("INSERT INTO user_roles (user_id, role_id, created_at) ");
        query_builder.push_values(role_ids, |mut b, role_id| {
            b.push_bind(user_id).push_bind(role_id).push_bind(now);
        });
        query_builder.build().execute(&mut **tx).await.map_err(|e| {
            tracing::error!("Database error inserting user_roles: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(())
    }

    /// Check if email exists
    pub async fn email_exists(pool: &PgPool, email: &str) -> Result<bool, ServiceError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND deleted_at IS NULL)",
        )
        .bind(email)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking email existence '{}': {:?}", email, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(exists)
    }

    /// Check if email exists for another user
    pub async fn email_exists_exclude_self(
        pool: &PgPool,
        email: &str,
        exclude_id: i64,
    ) -> Result<bool, ServiceError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND id != $2 AND deleted_at IS NULL)",
        )
        .bind(email)
        .bind(exclude_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!(
                "Database error checking email existence (exclude self) '{}': {:?}",
                email,
                e
            );
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(exists)
    }

    /// Check if username exists
    pub async fn username_exists(pool: &PgPool, username: &str) -> Result<bool, ServiceError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1 AND deleted_at IS NULL)",
        )
        .bind(username)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking username existence '{}': {:?}", username, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(exists)
    }

    pub async fn update_user_password(
        pool: &PgPool,
        id: i64,
        password_hash: &str,
    ) -> Result<bool, ServiceError> {
        let result =
            sqlx::query("UPDATE users SET password_hash = $1, updated_at = $2 WHERE id = $3 AND deleted_at IS NULL")
                .bind(password_hash)
                .bind(Utc::now().naive_utc())
                .bind(id)
                .execute(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database error updating user password for ID {}: {:?}", id, e);
                    ServiceError::DatabaseQueryFailed
                })?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn update_user_status(
        pool: &PgPool,
        id: i64,
        status: i16,
    ) -> Result<bool, ServiceError> {
        let result = sqlx::query("UPDATE users SET status = $1, updated_at = $2 WHERE id = $3 AND deleted_at IS NULL")
            .bind(status)
            .bind(Utc::now().naive_utc())
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error updating user status for ID {}: {:?}", id, e);
                ServiceError::DatabaseQueryFailed
            })?;

        Ok(result.rows_affected() > 0)
    }

    /// Find active user IDs assigned to a given role.
    pub async fn find_user_ids_by_role_id(
        pool: &PgPool,
        role_id: i64,
    ) -> Result<Vec<i64>, ServiceError> {
        let ids: Vec<(i64,)> = sqlx::query_as(
            "SELECT ur.user_id FROM user_roles ur
             JOIN users u ON u.id = ur.user_id AND u.deleted_at IS NULL
             WHERE ur.role_id = $1",
        )
        .bind(role_id)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding user IDs by role_id {}: {:?}", role_id, e);
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(ids.into_iter().map(|(id,)| id).collect())
    }

    /// Find active user IDs whose roles include a given menu.
    pub async fn find_user_ids_by_menu_id(
        pool: &PgPool,
        menu_id: i64,
    ) -> Result<Vec<i64>, ServiceError> {
        let ids: Vec<(i64,)> = sqlx::query_as(
            "SELECT DISTINCT ur.user_id FROM role_menus rm
             JOIN user_roles ur ON ur.role_id = rm.role_id
             JOIN users u ON u.id = ur.user_id AND u.deleted_at IS NULL
             WHERE rm.menu_id = $1",
        )
        .bind(menu_id)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding user IDs by menu_id {}: {:?}", menu_id, e);
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(ids.into_iter().map(|(id,)| id).collect())
    }

    /// Clear auto-lockout: reset failed_login_attempts and locked_until.
    pub async fn unlock_user(pool: &PgPool, id: i64) -> Result<bool, ServiceError> {
        let result = sqlx::query(
            "UPDATE users SET failed_login_attempts = 0, locked_until = NULL, updated_at = $1 WHERE id = $2 AND deleted_at IS NULL"
        )
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error unlocking user ID {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::password::PasswordUtils;
    use sqlx::PgPool;

    async fn seed_user(pool: &PgPool, username: &str, email: &str) -> i64 {
        let hash = PasswordUtils::hash_password("Test@12345").unwrap();
        UserRepository::create_user(
            pool,
            &CreateUserCommand {
                username: username.to_string(),
                email: email.to_string(),
                password_hash: hash,
                real_name: Some("Test User".to_string()),
                status: Some(1),
                role_ids: vec![],
            },
        )
        .await
        .expect("create_user should succeed")
    }

    #[sqlx::test]
    async fn create_user_and_find_by_id(pool: PgPool) {
        let id = seed_user(&pool, "testuser_find", "testuser_find@example.com").await;
        assert!(id > 0);

        let found = UserRepository::find_by_id(&pool, id).await.unwrap();
        assert!(found.is_some());
        let user = found.unwrap();
        assert_eq!(user.username, "testuser_find");
    }

    #[sqlx::test]
    async fn find_by_id_returns_none_for_missing(pool: PgPool) {
        let result = UserRepository::find_by_id(&pool, 999_999).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[sqlx::test]
    async fn get_by_id_returns_error_for_missing(pool: PgPool) {
        let result = UserRepository::get_by_id(&pool, 999_999).await;
        assert!(result.is_err());
        matches!(result.unwrap_err(), ServiceError::NotFound(_));
    }

    #[sqlx::test]
    async fn email_exists_is_true_after_create(pool: PgPool) {
        let email = "unique_test@example.com";
        seed_user(&pool, "testuser_email", email).await;
        let exists = UserRepository::email_exists(&pool, email).await.unwrap();
        assert!(exists);
    }

    #[sqlx::test]
    async fn username_exists_is_false_for_new(pool: PgPool) {
        let exists =
            UserRepository::username_exists(&pool, "definitely_not_seeded_xyz").await.unwrap();
        assert!(!exists);
    }

    #[sqlx::test]
    async fn soft_delete_makes_user_unfindable(pool: PgPool) {
        let id = seed_user(&pool, "testuser_del", "testuser_del@example.com").await;

        let deleted = UserRepository::soft_delete(&pool, id).await.unwrap();
        assert!(deleted);

        // user_with_roles view filters out soft-deleted users
        let found = UserRepository::find_by_id(&pool, id).await.unwrap();
        assert!(found.is_none());
    }

    #[sqlx::test]
    async fn find_with_pagination_returns_seeded_users(pool: PgPool) {
        seed_user(&pool, "pagtest1", "pagtest1@example.com").await;
        seed_user(&pool, "pagtest2", "pagtest2@example.com").await;

        let query = UserListQuery {
            username: Some("pagtest".to_string()),
            status: None,
            real_name: None,
            email: None,
        };
        let (users, total) =
            UserRepository::find_with_pagination(&pool, 0, 10, query).await.unwrap();
        assert_eq!(total, 2);
        assert_eq!(users.len(), 2);
    }
}
