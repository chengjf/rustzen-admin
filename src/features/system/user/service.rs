use super::{
    dto::{
        CreateUserDto, ResetPasswordResp, UpdateUserPasswordPayload, UpdateUserPayload,
        UpdateUserStatusPayload, UserItemResp, UserOptionResp, UserOptionsQuery, UserQuery,
    },
    repo::{CreateUserCommand, UserListQuery, UserRepository},
};
use crate::{
    common::{error::ServiceError, pagination::Pagination},
    core::{password::PasswordUtils, permission::PermissionService, session::SessionStore},
    features::auth::model::UserStatus,
};

use chrono::Utc;
use sqlx::PgPool;

/// User service for business operations
pub struct UserService;

impl UserService {
    /// Get user list with pagination
    pub async fn get_user_list(
        pool: &PgPool,
        query: UserQuery,
    ) -> Result<(Vec<UserItemResp>, i64), ServiceError> {
        tracing::info!("Fetching user list with query: {:?}", query);

        let (limit, offset, _) = Pagination::normalize(query.current, query.page_size);
        let repo_query = UserListQuery {
            username: query.username.clone(),
            status: query.status.map(i16::from),
            real_name: query.real_name.clone(),
            email: query.email.clone(),
        };

        let (users, total) =
            UserRepository::find_with_pagination(pool, offset, limit, repo_query).await?;

        tracing::info!("Users: {:?}", users);
        let list = users.into_iter().map(UserItemResp::from).collect();

        Ok((list, total))
    }

    /// Create user
    pub async fn create_user(pool: &PgPool, dto: CreateUserDto) -> Result<i64, ServiceError> {
        tracing::debug!("Creating user: {}", dto.username);

        // Check if username already exists
        if UserRepository::username_exists(pool, &dto.username).await? {
            return Err(ServiceError::UsernameConflict);
        }

        // Check if email already exists
        if UserRepository::email_exists(pool, &dto.email).await? {
            return Err(ServiceError::EmailConflict);
        }

        // Hash password
        let password_hash = PasswordUtils::hash_password(&dto.password)?;

        // Create user DTO with hashed password
        let create_cmd = CreateUserCommand {
            username: dto.username,
            email: dto.email,
            password_hash,
            real_name: dto.real_name,
            status: dto.status.map(i16::from),
            role_ids: dto.role_ids,
        };

        let user_id = UserRepository::create_user(pool, &create_cmd).await?;

        Ok(user_id)
    }

    /// Update user
    pub async fn update_user(
        pool: &PgPool,
        id: i64,
        request: UpdateUserPayload,
    ) -> Result<i64, ServiceError> {
        tracing::debug!("Updating user ID: {}", id);

        // Check if email already exists for another user
        if UserRepository::email_exists_exclude_self(pool, &request.email, id).await? {
            return Err(ServiceError::EmailConflict);
        }

        // Update user
        let user_id = UserRepository::update_user(
            pool,
            id,
            &request.email,
            request.real_name.as_deref(),
            &request.role_ids,
        )
        .await?;

        Ok(user_id)
    }

    /// Delete user
    pub async fn delete_user(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        tracing::debug!("Deleting user ID: {}", id);

        // Ensure user exists (get_by_id returns NotFound if missing)
        let _ = UserRepository::get_by_id(pool, id).await?;

        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for user deletion: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        // Clean up user-role associations
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
            tracing::error!("Database error deleting user_roles for user {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        // Soft delete user
        let result = sqlx::query(
            "UPDATE users SET deleted_at = $1, updated_at = $1 WHERE id = $2 AND deleted_at IS NULL"
        )
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error soft deleting user ID {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        if result.rows_affected() == 0 {
            return Err(ServiceError::NotFound(format!("User id: {}", id)));
        }

        tx.commit().await.map_err(|e| {
            tracing::error!("Database error committing user deletion transaction: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        // Revoke active session immediately after deletion
        PermissionService::clear_user_cache(id);
        if let Err(e) = SessionStore::delete(pool, id).await {
            tracing::error!("Failed to delete session for user_id={} after deletion: {:?}", id, e);
        }

        Ok(())
    }

    /// Get user status options
    pub fn get_user_status_options() -> Vec<UserOptionResp> {
        vec![
            UserOptionResp { label: "Normal".to_string(), value: UserStatus::Normal as i64 },
            UserOptionResp { label: "Disabled".to_string(), value: UserStatus::Disabled as i64 },
        ]
    }

    /// Get user options for dropdowns
    pub async fn get_user_options(
        pool: &PgPool,
        query: UserOptionsQuery,
    ) -> Result<Vec<UserOptionResp>, ServiceError> {
        tracing::debug!("Getting user options with query: {:?}", query);

        let options =
            UserRepository::find_options(pool, query.status.map(i16::from), query.q.as_deref(), query.limit)
                .await?;

        let user_options =
            options.into_iter().map(|(value, label)| UserOptionResp { label, value }).collect();

        Ok(user_options)
    }

    pub async fn update_user_password(
        pool: &PgPool,
        id: i64,
        _dto: UpdateUserPasswordPayload,
    ) -> Result<ResetPasswordResp, ServiceError> {
        tracing::debug!("Resetting password for user ID: {}", id);

        let password = PasswordUtils::generate_password(6);
        let password_hash = PasswordUtils::hash_password(&password)?;

        UserRepository::update_user_password(pool, id, &password_hash).await?;

        // Revoke active session so the user must re-login with the new password
        PermissionService::clear_user_cache(id);
        if let Err(e) = SessionStore::delete(pool, id).await {
            tracing::error!("Failed to delete session after password reset for user_id={}: {:?}", id, e);
        }
        tracing::info!("Session revoked for user_id={} after admin password reset", id);

        Ok(ResetPasswordResp { password })
    }

    /// Unlock a user that was auto-locked due to repeated failed login attempts.
    /// Clears `locked_until` and resets `failed_login_attempts`.
    pub async fn unlock_user(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        tracing::debug!("Unlocking user ID: {}", id);

        UserRepository::unlock_user(pool, id).await?;

        tracing::info!("User ID {} unlocked by admin", id);
        Ok(())
    }

    pub async fn update_user_status(
        pool: &PgPool,
        id: i64,
        dto: UpdateUserStatusPayload,
    ) -> Result<bool, ServiceError> {
        tracing::debug!("Updating user status for user ID: {}", id);

        let result = UserRepository::update_user_status(pool, id, dto.status.into()).await?;

        // Revoke active session immediately when disabling or locking an account
        if dto.status == UserStatus::Disabled || dto.status == UserStatus::Locked {
            PermissionService::clear_user_cache(id);
            if let Err(e) = SessionStore::delete(pool, id).await {
                tracing::error!("Failed to delete session for user_id={} on status change to {:?}: {:?}", id, dto.status, e);
            }
            tracing::info!("Session revoked for user_id={} due to status change to {:?}", id, dto.status);
        }

        Ok(result)
    }
}
