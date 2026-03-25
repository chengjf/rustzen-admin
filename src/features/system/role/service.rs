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
    features::system::menu::repo::MenuRepository,
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
        let repo_query = RoleListQuery {
            role_name: query.role_name,
            role_code: query.role_code,
            status: query.status,
        };

        let (roles, total) =
            RoleRepository::find_with_pagination(pool, offset, limit, repo_query).await?;

        let list = roles.into_iter().map(RoleItemResp::from).collect();

        Ok((list, total))
    }

    /// Create new role with validation
    pub async fn create_role(pool: &PgPool, request: CreateRoleDto) -> Result<(), ServiceError> {
        tracing::info!("Creating role: {}", request.name);
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

        tracing::info!("Updated role: {}", new_id);
        Ok(())
    }

    /// Delete role with user assignment validation
    pub async fn delete_role(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        tracing::info!("Attempting to delete role: {}", id);

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
