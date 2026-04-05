use super::{
    dto::{
        CreateUserDto, ResetPasswordResp, UpdateUserPasswordPayload, UpdateUserPayload,
        UpdateUserStatusPayload, UserItemResp, UserOptionResp, UserOptionsQuery, UserQuery,
    },
    repo::{CreateUserCommand, UserListQuery, UserRepository},
};
use crate::{
    common::{error::ServiceError, pagination::Pagination},
    core::{password::PasswordUtils, session::SessionStore},
    features::{auth::model::UserStatus, system::role::repo::RoleRepository},
};
use std::collections::HashSet;

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

        // Validate role_ids
        Self::validate_role_ids(pool, &dto.role_ids).await?;

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

    /// Reject operation if the target user is a system built-in user.
    pub async fn ensure_not_system(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        let user = UserRepository::get_by_id(pool, id).await?;
        if user.is_system {
            return Err(ServiceError::UserIsAdmin);
        }
        Ok(())
    }

    /// Validate role_ids: no duplicates, all IDs must exist and be enabled.
    async fn validate_role_ids(pool: &PgPool, role_ids: &[i64]) -> Result<(), ServiceError> {
        if role_ids.is_empty() {
            return Ok(());
        }
        let unique: HashSet<i64> = role_ids.iter().cloned().collect();
        if unique.len() != role_ids.len() {
            return Err(ServiceError::InvalidOperation("角色ID重复".into()));
        }
        let found = RoleRepository::find_existing_role_ids(pool, role_ids).await?;
        if found.len() != role_ids.len() {
            let missing: Vec<String> =
                role_ids.iter().filter(|id| !found.contains(id)).map(|id| id.to_string()).collect();
            return Err(ServiceError::InvalidOperation(format!(
                "角色ID {} 不存在或已禁用",
                missing.join(",")
            )));
        }
        Ok(())
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

        // Validate role_ids
        Self::validate_role_ids(pool, &request.role_ids).await?;

        // Update user
        let user_id = UserRepository::update_user(
            pool,
            id,
            &request.email,
            request.real_name.as_deref(),
            &request.role_ids,
        )
        .await?;

        // Delete session so the next login picks up updated role assignments.
        if let Err(e) = SessionStore::delete_by_user_id(pool, id).await {
            tracing::error!(
                "Failed to delete session for user_id={} after role update: {:?}",
                id,
                e
            );
        }

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

        // Revoke active session immediately after deletion.
        if let Err(e) = SessionStore::delete_by_user_id(pool, id).await {
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

        let options = UserRepository::find_options(
            pool,
            query.status.map(i16::from),
            query.q.as_deref(),
            query.limit,
        )
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

        let updated = UserRepository::update_user_password(pool, id, &password_hash).await?;
        if !updated {
            return Err(ServiceError::NotFound(format!("用户 ID: {}", id)));
        }

        // Revoke session so the user must re-login with the new password.
        if let Err(e) = SessionStore::delete_by_user_id(pool, id).await {
            tracing::error!(
                "Failed to delete session after password reset for user_id={}: {:?}",
                id,
                e
            );
        }
        tracing::info!("Session revoked for user_id={} after admin password reset", id);

        Ok(ResetPasswordResp { password })
    }

    /// Unlock a user that was auto-locked due to repeated failed login attempts.
    /// Clears `locked_until` and resets `failed_login_attempts`.
    pub async fn unlock_user(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        tracing::debug!("Unlocking user ID: {}", id);

        let updated = UserRepository::unlock_user(pool, id).await?;
        if !updated {
            return Err(ServiceError::NotFound(format!("用户 ID: {}", id)));
        }

        tracing::info!("User ID {} unlocked by admin", id);
        Ok(())
    }

    pub async fn update_user_status(
        pool: &PgPool,
        id: i64,
        dto: UpdateUserStatusPayload,
    ) -> Result<bool, ServiceError> {
        tracing::debug!("Updating user status for user ID: {}", id);

        let updated = UserRepository::update_user_status(pool, id, dto.status.into()).await?;
        if !updated {
            return Err(ServiceError::NotFound(format!("用户 ID: {}", id)));
        }

        // Revoke active session immediately when disabling, pending, or locking an account
        if dto.status == UserStatus::Disabled
            || dto.status == UserStatus::Locked
            || dto.status == UserStatus::Pending
        {
            if let Err(e) = SessionStore::delete_by_user_id(pool, id).await {
                tracing::error!(
                    "Failed to delete session for user_id={} on status change to {:?}: {:?}",
                    id,
                    dto.status,
                    e
                );
            }
            tracing::info!(
                "Session revoked for user_id={} due to status change to {:?}",
                id,
                dto.status
            );
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        common::error::ServiceError,
        core::session::SessionStore,
        features::{
            auth::model::UserStatus,
            system::{
                role::repo::RoleRepository,
                user::dto::{CreateUserDto, UpdateUserPayload, UpdateUserStatusPayload},
            },
        },
    };
    use chrono::{Duration, Utc};
    use sqlx::PgPool;

    fn make_create_dto(username: &str, email: &str, role_ids: Vec<i64>) -> CreateUserDto {
        CreateUserDto {
            username: username.to_string(),
            email: email.to_string(),
            password: "TestPass@123".to_string(),
            real_name: None,
            status: None,
            role_ids,
        }
    }

    #[sqlx::test]
    async fn create_user_with_no_roles_succeeds(pool: PgPool) {
        let id = UserService::create_user(
            &pool,
            make_create_dto("newuser1", "newuser1@test.com", vec![]),
        )
        .await;
        assert!(id.is_ok());
        assert!(id.unwrap() > 0);
    }

    #[sqlx::test]
    async fn create_user_duplicate_username_returns_conflict(pool: PgPool) {
        UserService::create_user(&pool, make_create_dto("dupuser", "dup1@test.com", vec![]))
            .await
            .unwrap();

        let result =
            UserService::create_user(&pool, make_create_dto("dupuser", "dup2@test.com", vec![]))
                .await;
        assert!(matches!(result, Err(ServiceError::UsernameConflict)));
    }

    #[sqlx::test]
    async fn create_user_duplicate_email_returns_conflict(pool: PgPool) {
        UserService::create_user(&pool, make_create_dto("emailuser1", "same@test.com", vec![]))
            .await
            .unwrap();

        let result =
            UserService::create_user(&pool, make_create_dto("emailuser2", "same@test.com", vec![]))
                .await;
        assert!(matches!(result, Err(ServiceError::EmailConflict)));
    }

    #[sqlx::test]
    async fn create_user_with_nonexistent_role_returns_error(pool: PgPool) {
        let result = UserService::create_user(
            &pool,
            make_create_dto("roletest", "roletest@test.com", vec![999_999]),
        )
        .await;
        assert!(matches!(result, Err(ServiceError::InvalidOperation(_))));
    }

    #[sqlx::test]
    async fn create_user_with_duplicate_role_ids_returns_error(pool: PgPool) {
        let role_id = RoleRepository::create(&pool, "测试角色", "TEST_USER_R", None, 1, 0, &[])
            .await
            .unwrap();

        let result = UserService::create_user(
            &pool,
            make_create_dto("duproleid", "duproleid@test.com", vec![role_id, role_id]),
        )
        .await;
        assert!(matches!(result, Err(ServiceError::InvalidOperation(_))));
    }

    #[sqlx::test]
    async fn create_user_with_disabled_role_returns_error(pool: PgPool) {
        let disabled_role_id =
            RoleRepository::create(&pool, "禁用角色", "DISABLED_R2", None, 2, 0, &[])
                .await
                .unwrap();

        let result = UserService::create_user(
            &pool,
            make_create_dto("disroleuser", "disroleuser@test.com", vec![disabled_role_id]),
        )
        .await;
        assert!(matches!(result, Err(ServiceError::InvalidOperation(_))));
    }

    #[sqlx::test]
    async fn ensure_not_system_blocks_system_user(pool: PgPool) {
        let (superadmin_id,): (i64,) =
            sqlx::query_as("SELECT id FROM users WHERE username = 'superadmin'")
                .fetch_one(&pool)
                .await
                .unwrap();

        let result = UserService::ensure_not_system(&pool, superadmin_id).await;
        assert!(matches!(result, Err(ServiceError::UserIsAdmin)));
    }

    #[sqlx::test]
    async fn ensure_not_system_allows_regular_user(pool: PgPool) {
        let id = UserService::create_user(
            &pool,
            make_create_dto("regular_user", "regular@test.com", vec![]),
        )
        .await
        .unwrap();

        let result = UserService::ensure_not_system(&pool, id).await;
        assert!(result.is_ok());
    }

    // ── update_user ──────────────────────────────────────────────────────

    #[sqlx::test]
    async fn update_user_changes_email_and_real_name(pool: PgPool) {
        let id =
            UserService::create_user(&pool, make_create_dto("upd_user", "upd@test.com", vec![]))
                .await
                .unwrap();

        let result = UserService::update_user(
            &pool,
            id,
            UpdateUserPayload {
                email: "upd_new@test.com".to_string(),
                real_name: Some("新名字".to_string()),
                role_ids: vec![],
            },
        )
        .await;

        assert!(result.is_ok());

        // Verify the change persisted
        let user = UserRepository::get_by_id(&pool, id).await.unwrap();
        assert_eq!(user.email, "upd_new@test.com");
    }

    #[sqlx::test]
    async fn update_user_duplicate_email_returns_conflict(pool: PgPool) {
        let _id1 =
            UserService::create_user(&pool, make_create_dto("upd_u1", "taken@test.com", vec![]))
                .await
                .unwrap();
        let id2 =
            UserService::create_user(&pool, make_create_dto("upd_u2", "own@test.com", vec![]))
                .await
                .unwrap();

        let result = UserService::update_user(
            &pool,
            id2,
            UpdateUserPayload {
                email: "taken@test.com".to_string(), // already owned by id1
                real_name: None,
                role_ids: vec![],
            },
        )
        .await;

        assert!(matches!(result, Err(ServiceError::EmailConflict)));
    }

    #[sqlx::test]
    async fn update_user_not_found_returns_error(pool: PgPool) {
        let result = UserService::update_user(
            &pool,
            999_999,
            UpdateUserPayload {
                email: "nobody@test.com".to_string(),
                real_name: None,
                role_ids: vec![],
            },
        )
        .await;

        assert!(matches!(result, Err(ServiceError::NotFound(_))));
    }

    // ── delete_user ──────────────────────────────────────────────────────

    #[sqlx::test]
    async fn delete_user_removes_user(pool: PgPool) {
        let id =
            UserService::create_user(&pool, make_create_dto("del_user", "del@test.com", vec![]))
                .await
                .unwrap();

        UserService::delete_user(&pool, id).await.expect("delete should succeed");

        let found = UserRepository::find_by_id(&pool, id).await.unwrap();
        assert!(found.is_none(), "deleted user should not be findable");
    }

    #[sqlx::test]
    async fn delete_user_not_found_returns_error(pool: PgPool) {
        let result = UserService::delete_user(&pool, 999_999).await;
        assert!(matches!(result, Err(ServiceError::NotFound(_))));
    }

    #[sqlx::test]
    async fn get_user_list_returns_paginated_filtered_users(pool: PgPool) {
        let matched_id = UserService::create_user(
            &pool,
            CreateUserDto {
                username: "list_user_alpha".to_string(),
                email: "list_user_alpha@test.com".to_string(),
                password: "TestPass@123".to_string(),
                real_name: Some("列表用户甲".to_string()),
                status: Some(UserStatus::Normal),
                role_ids: vec![],
            },
        )
        .await
        .unwrap();

        UserService::create_user(
            &pool,
            CreateUserDto {
                username: "list_user_beta".to_string(),
                email: "list_user_beta@test.com".to_string(),
                password: "TestPass@123".to_string(),
                real_name: Some("其他用户".to_string()),
                status: Some(UserStatus::Disabled),
                role_ids: vec![],
            },
        )
        .await
        .unwrap();

        let (users, total) = UserService::get_user_list(
            &pool,
            UserQuery {
                current: Some(1),
                page_size: Some(10),
                username: Some("list_user_alpha".to_string()),
                status: Some(UserStatus::Normal),
                real_name: Some("列表用户".to_string()),
                email: Some("alpha@".to_string()),
            },
        )
        .await
        .unwrap();

        assert_eq!(total, 1);
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].id, matched_id);
    }

    #[sqlx::test]
    async fn get_user_options_returns_filtered_values(pool: PgPool) {
        let enabled_id = UserService::create_user(
            &pool,
            CreateUserDto {
                username: "option_user_enabled".to_string(),
                email: "option_user_enabled@test.com".to_string(),
                password: "TestPass@123".to_string(),
                real_name: Some("用户选项启用".to_string()),
                status: Some(UserStatus::Normal),
                role_ids: vec![],
            },
        )
        .await
        .unwrap();
        let disabled_id = UserService::create_user(
            &pool,
            CreateUserDto {
                username: "option_user_disabled".to_string(),
                email: "option_user_disabled@test.com".to_string(),
                password: "TestPass@123".to_string(),
                real_name: Some("用户选项禁用".to_string()),
                status: Some(UserStatus::Disabled),
                role_ids: vec![],
            },
        )
        .await
        .unwrap();

        let options = UserService::get_user_options(
            &pool,
            UserOptionsQuery {
                status: Some(UserStatus::Normal),
                q: Some("用户选项".to_string()),
                limit: Some(10),
            },
        )
        .await
        .unwrap();

        assert!(options.iter().any(|item| item.value == enabled_id && item.label == "用户选项启用"));
        assert!(!options.iter().any(|item| item.value == disabled_id));
    }

    #[test]
    fn get_user_status_options_returns_normal_and_disabled() {
        let options = UserService::get_user_status_options();
        assert_eq!(options.len(), 2);
        assert_eq!(options[0].label, "Normal");
        assert_eq!(options[0].value, UserStatus::Normal as i64);
        assert_eq!(options[1].label, "Disabled");
        assert_eq!(options[1].value, UserStatus::Disabled as i64);
    }

    // ── update_user_status ───────────────────────────────────────────────

    #[sqlx::test]
    async fn update_user_password_resets_password_and_invalidates_session(pool: PgPool) {
        let id = UserService::create_user(
            &pool,
            make_create_dto("password_reset_user", "password_reset_user@test.com", vec![]),
        )
        .await
        .unwrap();
        let token = SessionStore::create(
            &pool,
            id,
            Utc::now() + Duration::hours(1),
            "127.0.0.1",
            "test-agent",
        )
        .await
        .unwrap();

        let response = UserService::update_user_password(
            &pool,
            id,
            UpdateUserPasswordPayload {},
        )
        .await
        .unwrap();

        assert_eq!(response.password.len(), 6);
        assert!(SessionStore::get_by_token(&pool, &token).await.unwrap().is_none());
    }

    #[sqlx::test]
    async fn update_user_password_returns_not_found_for_missing(pool: PgPool) {
        let result =
            UserService::update_user_password(&pool, 999_999, UpdateUserPasswordPayload {}).await;
        assert!(matches!(result, Err(ServiceError::NotFound(_))));
    }

    #[sqlx::test]
    async fn unlock_user_clears_auto_lock(pool: PgPool) {
        let id = UserService::create_user(
            &pool,
            make_create_dto("unlock_service_user", "unlock_service_user@test.com", vec![]),
        )
        .await
        .unwrap();

        sqlx::query(
            "UPDATE users SET failed_login_attempts = 5, locked_until = NOW() + INTERVAL '30 minutes' WHERE id = $1",
        )
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

        UserService::unlock_user(&pool, id).await.unwrap();

        let (attempts, locked_until): (i16, Option<chrono::DateTime<chrono::Utc>>) =
            sqlx::query_as("SELECT failed_login_attempts, locked_until FROM users WHERE id = $1")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(attempts, 0);
        assert!(locked_until.is_none());
    }

    #[sqlx::test]
    async fn unlock_user_returns_not_found_for_missing(pool: PgPool) {
        let result = UserService::unlock_user(&pool, 999_999).await;
        assert!(matches!(result, Err(ServiceError::NotFound(_))));
    }

    #[sqlx::test]
    async fn update_user_status_to_disabled(pool: PgPool) {
        let id = UserService::create_user(
            &pool,
            make_create_dto("status_user", "status@test.com", vec![]),
        )
        .await
        .unwrap();

        let result = UserService::update_user_status(
            &pool,
            id,
            UpdateUserStatusPayload { status: UserStatus::Disabled },
        )
        .await;

        assert!(result.is_ok());

        let (status,): (i16,) = sqlx::query_as("SELECT status FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
        .unwrap();
        assert_eq!(status, UserStatus::Disabled as i16);
    }

    #[sqlx::test]
    async fn update_user_status_to_normal_keeps_session(pool: PgPool) {
        let id = UserService::create_user(
            &pool,
            make_create_dto("status_normal_user", "status_normal_user@test.com", vec![]),
        )
        .await
        .unwrap();
        let token = SessionStore::create(
            &pool,
            id,
            Utc::now() + Duration::hours(1),
            "127.0.0.1",
            "test-agent",
        )
        .await
        .unwrap();

        let result = UserService::update_user_status(
            &pool,
            id,
            UpdateUserStatusPayload { status: UserStatus::Normal },
        )
        .await;

        assert_eq!(result.unwrap(), true);
        assert!(SessionStore::get_by_token(&pool, &token).await.unwrap().is_some());
    }

    #[sqlx::test]
    async fn update_user_status_not_found_returns_error(pool: PgPool) {
        let result = UserService::update_user_status(
            &pool,
            999_999,
            UpdateUserStatusPayload { status: UserStatus::Disabled },
        )
        .await;
        assert!(matches!(result, Err(ServiceError::NotFound(_))));
    }
}
