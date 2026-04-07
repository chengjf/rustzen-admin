use std::collections::HashSet;

use super::{
    dto::{CreateRoleDto, RoleItemResp, RoleQuery, UpdateRolePayload},
    repo::{RoleListQuery, RoleRepository},
};
use crate::{
    common::{
        api::{OptionItem, OptionsQuery},
        error::ServiceError,
        pagination::Pagination,
    },
    core::session::SessionStore,
    features::system::{menu::repo::MenuRepository, user::repo::UserRepository},
};

use chrono::Utc;
use sqlx::PgPool;

pub struct RoleService;

impl RoleService {
    /// Get paginated role list with filtering
    pub async fn get_role_list(
        pool: &PgPool,
        query: RoleQuery,
    ) -> Result<(Vec<RoleItemResp>, i64), ServiceError> {
        tracing::info!("Fetching role list with query: {:?}", query);

        let (limit, offset, _) = Pagination::normalize(query.current, query.page_size);
        let repo_query = RoleListQuery { name: query.name, code: query.code, status: query.status };

        let (roles, total) =
            RoleRepository::find_with_pagination(pool, offset, limit, repo_query).await?;

        let list = roles.into_iter().map(RoleItemResp::from).collect();

        Ok((list, total))
    }

    /// Create new role with validation
    pub async fn create_role(pool: &PgPool, request: CreateRoleDto) -> Result<(), ServiceError> {
        tracing::info!("Creating role: {}", request.name);
        // 检查角色名称是否已被占用
        if RoleRepository::count_by_name(pool, &request.name).await? > 0 {
            return Err(ServiceError::InvalidOperation(format!(
                "角色名称 {} 已存在",
                request.name
            )));
        }
        // 检查角色编码是否已被占用
        let count = RoleRepository::count_by_code(pool, &request.code).await?;
        if count > 0 {
            tracing::warn!("Role code {} already exists", request.code);
            return Err(ServiceError::InvalidOperation(format!(
                "角色编码 {} 已存在",
                request.code
            )));
        }

        Self::validate_menu_ids(pool, request.menu_ids.clone()).await?;
        let id: i64 = RoleRepository::create(
            pool,
            &request.name,
            &request.code,
            request.description.as_deref(),
            request.status,
            request.sort_order.unwrap_or(0),
            &request.menu_ids,
        )
        .await?;

        tracing::info!("Created role: {}", id);
        Ok(())
    }

    /// Update existing role with validation
    pub async fn update_role(
        pool: &PgPool,
        id: i64,
        request: UpdateRolePayload,
    ) -> Result<(), ServiceError> {
        tracing::info!("Updating role: {}", id);

        // 系统内置角色不允许修改
        let role = RoleRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("角色 ID: {}", id)))?;
        if role.is_system == Some(true) {
            return Err(ServiceError::InvalidOperation("系统内置角色不能修改".into()));
        }

        // 检查角色名称是否已被其他角色占用
        if RoleRepository::name_exists_exclude_self(pool, &request.name, id).await? {
            return Err(ServiceError::InvalidOperation(format!(
                "角色名称 {} 已存在",
                request.name
            )));
        }
        // 检查角色编码是否已被其他角色占用
        if RoleRepository::code_exists_exclude_self(pool, &request.code, id).await? {
            tracing::warn!("Role code {} already exists", request.code);
            return Err(ServiceError::InvalidOperation(format!(
                "角色编码 {} 已存在",
                request.code
            )));
        }

        Self::validate_menu_ids(pool, request.menu_ids.clone()).await?;

        let new_id: i64 = RoleRepository::update(
            pool,
            id,
            &request.name,
            &request.code,
            request.description.as_deref(),
            request.status,
            request.sort_order.unwrap_or(0),
            &request.menu_ids,
        )
        .await?;

        // Invalidate permission caches for all users assigned to this role
        match UserRepository::find_user_ids_by_role_id(pool, id).await {
            Ok(user_ids) => {
                for uid in user_ids {
                    if let Err(e) = SessionStore::delete_by_user_id(pool, uid).await {
                        tracing::error!(
                            "Failed to delete session for user_id={} after role update: {:?}",
                            uid,
                            e
                        );
                    }
                }
            }
            Err(e) => {
                tracing::error!(
                    "Failed to fetch users for role_id={} during cache invalidation: {:?}",
                    id,
                    e
                );
            }
        }

        tracing::info!("Updated role: {}", new_id);
        Ok(())
    }

    /// Delete role with user assignment validation
    pub async fn delete_role(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        tracing::info!("Attempting to delete role: {}", id);

        // 系统内置角色不允许删除
        let role = RoleRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("角色 ID: {}", id)))?;
        if role.is_system == Some(true) {
            return Err(ServiceError::InvalidOperation("系统内置角色不能删除".into()));
        }

        // Check if role is still assigned to users
        let user_count = RoleRepository::get_role_user_count(pool, id).await?;
        if user_count > 0 {
            tracing::warn!("Cannot delete role {} - still assigned to {} users", id, user_count);
            return Err(ServiceError::InvalidOperation(format!(
                "角色ID {} 仍被 {} 个用户分配，无法删除",
                id, user_count
            )));
        }

        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for role deletion: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        // Clean up role-menu associations
        sqlx::query("DELETE FROM role_menus WHERE role_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
            tracing::error!("Database error deleting role_menus for role {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        // Soft delete role
        let result = sqlx::query(
            "UPDATE roles SET deleted_at = $1, updated_at = $1 WHERE id = $2 AND deleted_at IS NULL"
        )
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error soft deleting role {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        if result.rows_affected() == 0 {
            return Err(ServiceError::NotFound("角色".to_string()));
        }

        tx.commit().await.map_err(|e| {
            tracing::error!("Database error committing role deletion transaction: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        tracing::info!("Successfully deleted role: {}", id);
        Ok(())
    }

    /// Get role options for dropdowns
    pub async fn get_role_options(
        pool: &PgPool,
        query: OptionsQuery,
    ) -> Result<Vec<OptionItem<i64>>, ServiceError> {
        tracing::info!("Retrieving role options: {:?}", query);

        let roles = RoleRepository::find_options(pool, query.q.as_deref(), query.limit).await?;

        let options: Vec<OptionItem<i64>> =
            roles.into_iter().map(|(id, name)| OptionItem { label: name, value: id }).collect();

        tracing::info!("Retrieved {} role options", options.len());
        Ok(options)
    }

    /// 检查新建和修改角色时，菜单ID是否合理
    /// 1、是否有重复菜单ID
    /// 2、是否有不存在的菜单ID
    /// 3、路径是否完整，即是否有缺失的父菜单ID
    async fn validate_menu_ids(pool: &PgPool, menu_ids: Vec<i64>) -> Result<(), ServiceError> {
        if menu_ids.is_empty() {
            return Ok(());
        }

        // 1. 检查重复 (利用 HashSet)
        let unique_ids: HashSet<i64> = menu_ids.iter().cloned().collect();
        if unique_ids.len() != menu_ids.len() {
            return Err(ServiceError::InvalidOperation("菜单ID重复".to_string()));
        }

        // 2. 检查 ID 是否存在，并同时获取它们的 parent_id
        // 建议修改 Repository 接口，返回包含 id 和 parent_id 的结构体
        let menus = MenuRepository::find_by_ids(pool, menu_ids.clone()).await?;

        if menus.len() != menu_ids.len() {
            let found_ids: HashSet<i64> = menus.iter().map(|m| m.id).collect();
            let missing: Vec<String> = menu_ids
                .iter()
                .filter(|id| !found_ids.contains(id))
                .map(|id| id.to_string())
                .collect();
            return Err(ServiceError::InvalidOperation(format!(
                "菜单ID {} 不存在",
                missing.join(",")
            )));
        }

        // 3. 检查路径完整性（核心改进）
        // 逻辑：遍历每个选中的菜单，如果它的 parent_id 不是 0，那么这个 parent_id 必须也在 unique_ids 中
        for menu in menus {
            if menu.parent_id != 0 && !unique_ids.contains(&menu.parent_id) {
                return Err(ServiceError::InvalidOperation(format!(
                    "菜单路径不完整：缺少 ID 为 {} 的父级菜单",
                    menu.parent_id
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        common::error::ServiceError,
        core::session::SessionStore,
        features::system::{
            role::dto::{CreateRoleDto, UpdateRolePayload},
            user::{dto::CreateUserDto, service::UserService},
        },
    };
    use chrono::{Duration, Utc};
    use sqlx::PgPool;

    fn make_role_dto(name: &str, code: &str, menu_ids: Vec<i64>) -> CreateRoleDto {
        CreateRoleDto {
            name: name.to_string(),
            code: code.to_string(),
            status: 1,
            sort_order: None,
            menu_ids,
            description: None,
        }
    }

    #[sqlx::test]
    async fn create_role_succeeds(pool: PgPool) {
        // Use seed menus: 目录(2) -> 菜单(3) -> 按钮(4) — complete path
        let result =
            RoleService::create_role(&pool, make_role_dto("新角色", "NEW_ROLE", vec![2, 3, 4]))
                .await;
        assert!(result.is_ok());
    }

    #[sqlx::test]
    async fn create_role_duplicate_name_returns_error(pool: PgPool) {
        RoleService::create_role(&pool, make_role_dto("重复名称", "UNIQ_CODE1", vec![]))
            .await
            .unwrap();

        let result =
            RoleService::create_role(&pool, make_role_dto("重复名称", "UNIQ_CODE2", vec![])).await;
        assert!(matches!(result, Err(ServiceError::InvalidOperation(_))));
    }

    #[sqlx::test]
    async fn create_role_duplicate_code_returns_error(pool: PgPool) {
        RoleService::create_role(&pool, make_role_dto("角色A", "SAME_CODE", vec![])).await.unwrap();

        let result =
            RoleService::create_role(&pool, make_role_dto("角色B", "SAME_CODE", vec![])).await;
        assert!(matches!(result, Err(ServiceError::InvalidOperation(_))));
    }

    #[sqlx::test]
    async fn create_role_missing_parent_menu_returns_error(pool: PgPool) {
        // id=3 (菜单) has parent id=2 (目录). Providing only [3] is incomplete.
        let result =
            RoleService::create_role(&pool, make_role_dto("路径不全角色", "BROKEN_PATH", vec![3]))
                .await;
        assert!(
            matches!(result, Err(ServiceError::InvalidOperation(_))),
            "incomplete menu path should fail"
        );
    }

    #[sqlx::test]
    async fn create_role_with_nonexistent_menu_returns_error(pool: PgPool) {
        let result = RoleService::create_role(
            &pool,
            make_role_dto("无效菜单角色", "BAD_MENU", vec![999_999]),
        )
        .await;
        assert!(matches!(result, Err(ServiceError::InvalidOperation(_))));
    }

    #[sqlx::test]
    async fn create_role_with_duplicate_menu_ids_returns_error(pool: PgPool) {
        // id=2 duplicated
        let result =
            RoleService::create_role(&pool, make_role_dto("重复菜单角色", "DUP_MENU", vec![2, 2]))
                .await;
        assert!(matches!(result, Err(ServiceError::InvalidOperation(_))));
    }

    #[sqlx::test]
    async fn get_role_list_returns_paginated_filtered_roles(pool: PgPool) {
        let first_id =
            RoleRepository::create(&pool, "分页角色一", "PAGE_ROLE_1", None, 1, 0, &[2, 3])
                .await
                .unwrap();
        let second_id =
            RoleRepository::create(&pool, "分页角色二", "PAGE_ROLE_2", None, 1, 0, &[2, 10])
                .await
                .unwrap();
        RoleRepository::create(&pool, "分页角色禁用", "PAGE_ROLE_DISABLED", None, 2, 0, &[2, 3])
            .await
            .unwrap();

        let (roles, total) = RoleService::get_role_list(
            &pool,
            RoleQuery {
                current: Some(1),
                page_size: Some(10),
                name: None,
                code: None,
                status: Some("1".to_string()),
            },
        )
        .await
        .unwrap();

        assert!(total >= 2);
        assert!(!roles.is_empty());
        assert!(roles.iter().all(|role| role.status == 1));
        assert!(roles.iter().any(|role| role.id == first_id));
        assert!(roles.iter().any(|role| role.id == second_id));
    }

    #[sqlx::test]
    async fn get_role_options_returns_enabled_matches_only(pool: PgPool) {
        let enabled_id =
            RoleRepository::create(&pool, "角色选项启用", "ROLE_OPT_ENABLED", None, 1, 0, &[2, 3])
                .await
                .unwrap();
        let disabled_id =
            RoleRepository::create(&pool, "角色选项禁用", "ROLE_OPT_DISABLED", None, 2, 0, &[2, 3])
                .await
                .unwrap();

        let options = RoleService::get_role_options(
            &pool,
            OptionsQuery { q: Some("角色选项".to_string()), limit: Some(10) },
        )
        .await
        .unwrap();

        assert!(
            options.iter().any(|item| item.value == enabled_id && item.label == "角色选项启用")
        );
        assert!(!options.iter().any(|item| item.value == disabled_id));
    }

    #[sqlx::test]
    async fn delete_role_blocked_when_assigned_to_users(pool: PgPool) {
        // Create role, assign to user, then try to delete
        let role_id = RoleRepository::create(&pool, "有用户角色", "ASSIGNED_R", None, 1, 0, &[])
            .await
            .unwrap();

        UserService::create_user(
            &pool,
            CreateUserDto {
                username: "roleuser".to_string(),
                email: "roleuser@test.com".to_string(),
                password: "Test@Pass1".to_string(),
                real_name: None,
                status: None,
                role_ids: vec![role_id],
            },
        )
        .await
        .unwrap();

        let result = RoleService::delete_role(&pool, role_id).await;
        assert!(
            matches!(result, Err(ServiceError::InvalidOperation(_))),
            "should block deletion when role has users"
        );
    }

    #[sqlx::test]
    async fn delete_system_role_returns_error(pool: PgPool) {
        let sys_role_id: i64 = sqlx::query_scalar(
            "INSERT INTO roles (name, code, status, is_system, sort_order, created_at)
             VALUES ('系统角色', 'SYS_TEST_DEL', 1, TRUE, 99, NOW()) RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let result = RoleService::delete_role(&pool, sys_role_id).await;
        assert!(matches!(result, Err(ServiceError::InvalidOperation(_))));
    }

    #[sqlx::test]
    async fn update_system_role_returns_error(pool: PgPool) {
        let sys_role_id: i64 = sqlx::query_scalar(
            "INSERT INTO roles (name, code, status, is_system, sort_order, created_at)
             VALUES ('系统角色', 'SYS_TEST_UPD', 1, TRUE, 99, NOW()) RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let result = RoleService::update_role(
            &pool,
            sys_role_id,
            UpdateRolePayload {
                name: "新名称".to_string(),
                code: "NEW_CODE".to_string(),
                status: 1,
                sort_order: None,
                menu_ids: vec![],
                description: None,
            },
        )
        .await;

        assert!(matches!(result, Err(ServiceError::InvalidOperation(_))));
    }

    #[sqlx::test]
    async fn update_role_returns_not_found_for_missing(pool: PgPool) {
        let result = RoleService::update_role(
            &pool,
            999_999,
            UpdateRolePayload {
                name: "缺失角色".to_string(),
                code: "MISSING_ROLE".to_string(),
                status: 1,
                sort_order: None,
                menu_ids: vec![],
                description: None,
            },
        )
        .await;

        assert!(matches!(result, Err(ServiceError::NotFound(_))));
    }

    #[sqlx::test]
    async fn delete_role_returns_not_found_for_missing(pool: PgPool) {
        let result = RoleService::delete_role(&pool, 999_999).await;
        assert!(matches!(result, Err(ServiceError::NotFound(_))));
    }

    #[sqlx::test]
    async fn update_role_invalidates_sessions_for_assigned_users(pool: PgPool) {
        let role_id = RoleRepository::create(
            &pool,
            "会话失效角色",
            "ROLE_INVALIDATE_SESSION",
            None,
            1,
            0,
            &[2, 3, 4],
        )
        .await
        .unwrap();

        let user_id = UserService::create_user(
            &pool,
            CreateUserDto {
                username: "role_session_user".to_string(),
                email: "role_session_user@example.com".to_string(),
                password: "Role@Test1".to_string(),
                real_name: None,
                status: None,
                role_ids: vec![role_id],
            },
        )
        .await
        .unwrap();

        let token = SessionStore::create(
            &pool,
            user_id,
            Utc::now() + Duration::hours(1),
            "127.0.0.1",
            "test-agent",
        )
        .await
        .unwrap();

        assert!(SessionStore::get_by_token(&pool, &token).await.unwrap().is_some());

        RoleService::update_role(
            &pool,
            role_id,
            UpdateRolePayload {
                name: "会话失效角色-更新".to_string(),
                code: "ROLE_INVALIDATE_SESSION".to_string(),
                status: 1,
                sort_order: None,
                menu_ids: vec![2, 10, 11],
                description: None,
            },
        )
        .await
        .unwrap();

        assert!(SessionStore::get_by_token(&pool, &token).await.unwrap().is_none());
    }
}
