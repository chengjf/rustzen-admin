use super::model::MenuEntity;
use crate::common::{error::ServiceError, status::EnableStatus};

use chrono::Utc;
use sqlx::{PgPool, QueryBuilder, pool};

/// Menu data access layer
pub struct MenuRepository;

#[derive(Debug, Clone)]
pub struct MenuListQuery {
    pub name: Option<String>,
    pub code: Option<String>,
    pub status: Option<String>,
    pub menu_type: Option<i16>,
}

impl MenuRepository {
    fn format_query(query: &MenuListQuery, query_builder: &mut QueryBuilder<'_, sqlx::Postgres>) {
        if let Some(name) = &query.name {
            if !name.trim().is_empty() {
                query_builder.push(" AND name ILIKE  ").push_bind(format!("%{}%", name));
            }
        }
        if let Some(code) = &query.code {
            if !code.trim().is_empty() {
                query_builder.push(" AND code ILIKE  ").push_bind(format!("%{}%", code));
            }
        }
        if let Some(status) = &query.status {
            if let Ok(status_num) = status.parse::<i16>() {
                query_builder.push(" AND status = ").push_bind(status_num);
            }
        }
        if let Some(menu_type) = query.menu_type {
            query_builder.push(" AND menu_type = ").push_bind(menu_type);
        }
    }

    /// Queries a menu by ID
    /// Returns None if the menu does not exist or is deleted
    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<MenuEntity>, ServiceError> {
        sqlx::query_as::<_, MenuEntity>(
            "SELECT id, parent_id, name, code, menu_type, status, is_system, sort_order, created_at, updated_at FROM menus WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding menu by ID {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })
    }

    /// Queries menus by parent ID
    /// Returns None if the parent menu is deleted
    pub async fn find_by_parent_id(
        pool: &PgPool,
        parent_id: i64,
    ) -> Result<Vec<MenuEntity>, ServiceError> {
        let menus = sqlx::query_as::<_, MenuEntity>(
            "SELECT id, parent_id, name, code, menu_type, status, is_system, sort_order, created_at, updated_at FROM menus WHERE parent_id = $1 AND deleted_at IS NULL",
        )
        .bind(parent_id)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding menus by parent ID: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(menus)
    }

    /// Queries menus based on conditions
    pub async fn find_all(
        pool: &PgPool,
        query: MenuListQuery,
    ) -> Result<Vec<MenuEntity>, ServiceError> {
        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
            "SELECT id, parent_id, name, code, menu_type, status, is_system, sort_order, created_at, updated_at FROM menus WHERE 1=1",
        );

        Self::format_query(&query, &mut query_builder);

        query_builder.push(" AND deleted_at IS NULL");
        query_builder.push(" ORDER BY sort_order ASC, id ASC");

        let menus = query_builder.build_query_as().fetch_all(pool).await.map_err(|e| {
            tracing::error!("Database error finding menus with conditions: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(menus)
    }

    /// Creates a new menu
    pub async fn create(
        pool: &PgPool,
        parent_id: i64,
        name: &str,
        code: &str,
        menu_type: i16,
        sort_order: i16,
        status: i16,
    ) -> Result<i64, ServiceError> {
        let menu_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO menus (parent_id, name, code, menu_type, sort_order, status, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id",
        )
        .bind(parent_id)
        .bind(name)
        .bind(code)
        .bind(menu_type)
        .bind(sort_order)
        .bind(status)
        .bind(Utc::now().naive_utc())
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error creating menu: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(menu_id)
    }

    /// Updates an existing menu
    pub async fn update(
        pool: &PgPool,
        id: i64,
        parent_id: i64,
        name: &str,
        code: &str,
        menu_type: i16,
        sort_order: i16,
        status: i16,
    ) -> Result<i64, ServiceError> {
        let menu_id = sqlx::query_scalar::<_, i64>(
                "UPDATE menus
                 SET parent_id = $2, name = $3, code = $4, menu_type = $5, sort_order = $6, status = $7, updated_at = $8
                 WHERE id = $1 AND deleted_at IS NULL
                 RETURNING id",
            )
            .bind(id)
            .bind(parent_id)
            .bind(name)
            .bind(code)
            .bind(menu_type)
            .bind(sort_order)
            .bind(status)
            .bind(Utc::now().naive_utc())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error updating menu: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

        if let Some(menu_id) = menu_id {
            Ok(menu_id)
        } else {
            Err(ServiceError::NotFound("Menu".to_string()))
        }
    }

    /// Soft deletes a menu
    pub async fn soft_delete(pool: &PgPool, id: i64) -> Result<bool, ServiceError> {
        let result = sqlx::query(
            "UPDATE menus SET deleted_at = $1, updated_at = $1 WHERE id = $2 AND is_system = false AND deleted_at IS NULL"
        )
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error soft deleting menu {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(result.rows_affected() > 0)
    }

    /// Retrieves menu list for Options API
    pub async fn find_options(
        pool: &PgPool,
        search_query: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(i64, String)>, ServiceError> {
        let mut query = format!(
            "SELECT id, name FROM menus WHERE status = {} AND deleted_at IS NULL",
            EnableStatus::Enabled as i16,
        );

        if let Some(keyword) = search_query {
            query.push_str(&format!(" AND name ILIKE '%{}%'", keyword.replace("'", "''")));
        }

        query.push_str(" ORDER BY sort_order ASC, name ASC");

        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        let menus = sqlx::query_as(&query).fetch_all(pool).await.map_err(|e| {
            tracing::error!("Database error finding menu options: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(menus)
    }

    /// Retrieves menu options with code for permission grouping
    pub async fn find_options_with_code(
        pool: &PgPool,
        search_query: Option<&str>,
        limit: Option<i64>,
        btn_filter: Option<bool>,
    ) -> Result<Vec<(i64, String, Option<String>, i64, i16)>, ServiceError> {
        let mut query = format!(
            "SELECT id, name, code, parent_id, menu_type FROM menus WHERE status = {} AND deleted_at IS NULL",
            EnableStatus::Enabled as i16,
        );

        if let Some(keyword) = search_query {
            query.push_str(&format!(" AND name ILIKE '%{}%'", keyword.replace("'", "''")));
        }

        if let Some(btn_filter) = btn_filter {
            if btn_filter {
                query.push_str(" AND menu_type <> 3");
            }
        }

        query.push_str(" ORDER BY parent_id ASC, sort_order ASC, name ASC");

        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        let menus = sqlx::query_as(&query).fetch_all(pool).await.map_err(|e| {
            tracing::error!("Database error finding menu options with code: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(menus)
    }

    /// Retrieves menu IDs for a role
    pub async fn find_by_ids(
        pool: &PgPool,
        menu_ids: Vec<i64>,
    ) -> Result<Vec<MenuEntity>, ServiceError> {
        // 1. 使用 = ANY($1) 语法，这在 Postgres 中等价于 IN，但支持数组绑定
        // 2. 注意 query_scalar 直接返回单列数据，不需要定义临时结构体
        let ids = sqlx::query_as::<_, MenuEntity>(
            "SELECT id, parent_id, name, code, menu_type, status, is_system, sort_order, created_at, updated_at FROM menus WHERE id = ANY($1) AND deleted_at IS NULL",
        )
        .bind(&menu_ids) // 绑定 Vec<i64>
        .fetch_all(pool)
        .await
        .map_err(|e| {
            // 这里的日志参数位置修正一下，先打印错误信息
            tracing::error!(
                "Database error finding menu IDs. Input: {:?}, Error: {:?}",
                menu_ids,
                e
            );
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(ids)
    }

    pub async fn name_exists(pool: &PgPool, name: &str) -> Result<bool, ServiceError> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM menus WHERE name = $1 AND deleted_at IS NULL)",
        )
        .bind(name)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking menu name existence '{}': {:?}", name, e);
            ServiceError::DatabaseQueryFailed
        })
    }

    pub async fn code_exists(pool: &PgPool, code: &str) -> Result<bool, ServiceError> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM menus WHERE code = $1 AND deleted_at IS NULL)",
        )
        .bind(code)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking menu code existence '{}': {:?}", code, e);
            ServiceError::DatabaseQueryFailed
        })
    }

    pub async fn name_exists_exclude_self(
        pool: &PgPool,
        name: &str,
        exclude_id: i64,
    ) -> Result<bool, ServiceError> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM menus WHERE name = $1 AND id != $2 AND deleted_at IS NULL)",
        )
        .bind(name)
        .bind(exclude_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking menu name existence (exclude self) '{}': {:?}", name, e);
            ServiceError::DatabaseQueryFailed
        })
    }

    pub async fn code_exists_exclude_self(
        pool: &PgPool,
        code: &str,
        exclude_id: i64,
    ) -> Result<bool, ServiceError> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM menus WHERE code = $1 AND id != $2 AND deleted_at IS NULL)",
        )
        .bind(code)
        .bind(exclude_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking menu code existence (exclude self) '{}': {:?}", code, e);
            ServiceError::DatabaseQueryFailed
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    /// Seed data (0105_seed.sql) provides:
    /// id=2 (Directory: 系统管理, parent_id=0)
    /// id=3 (Menu: 用户管理, parent_id=2)
    /// id=4 (Button: 用户创建, parent_id=3)

    #[sqlx::test]
    async fn find_by_id_returns_seeded_menu(pool: PgPool) {
        let menu = MenuRepository::find_by_id(&pool, 2).await.unwrap();
        assert!(menu.is_some());
        let m = menu.unwrap();
        assert_eq!(m.code, "system");
        assert_eq!(m.parent_id, 0);
    }

    #[sqlx::test]
    async fn find_by_id_returns_none_for_missing(pool: PgPool) {
        let menu = MenuRepository::find_by_id(&pool, 999_999).await.unwrap();
        assert!(menu.is_none());
    }

    #[sqlx::test]
    async fn find_by_parent_id_returns_children(pool: PgPool) {
        // id=2 (系统管理) has children: id=3 (用户管理), id=10 (角色管理), id=15 (菜单管理), id=25 (操作日志)
        let children = MenuRepository::find_by_parent_id(&pool, 2).await.unwrap();
        assert!(!children.is_empty());
        let codes: Vec<&str> = children.iter().map(|m| m.code.as_str()).collect();
        assert!(codes.contains(&"system:user:list"));
    }

    #[sqlx::test]
    async fn find_by_ids_returns_batch(pool: PgPool) {
        let menus = MenuRepository::find_by_ids(&pool, vec![2, 3, 4]).await.unwrap();
        assert_eq!(menus.len(), 3);
    }

    #[sqlx::test]
    async fn name_exists_and_code_exists(pool: PgPool) {
        // Seeded menu: name="系统管理", code="system"
        let name_exists = MenuRepository::name_exists(&pool, "系统管理").await.unwrap();
        assert!(name_exists);

        let code_exists = MenuRepository::code_exists(&pool, "system").await.unwrap();
        assert!(code_exists);

        let no_name = MenuRepository::name_exists(&pool, "不存在菜单").await.unwrap();
        assert!(!no_name);
    }

    #[sqlx::test]
    async fn create_menu_and_find(pool: PgPool) {
        let id = MenuRepository::create(&pool, 0, "测试目录", "test:dir", 1, 1, 1).await.unwrap();
        assert!(id > 0);

        let found = MenuRepository::find_by_id(&pool, id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "测试目录");
    }

    #[sqlx::test]
    async fn soft_delete_non_system_menu(pool: PgPool) {
        let id = MenuRepository::create(&pool, 0, "待删目录", "test:del", 1, 1, 1).await.unwrap();

        let deleted = MenuRepository::soft_delete(&pool, id).await.unwrap();
        assert!(deleted);

        let found = MenuRepository::find_by_id(&pool, id).await.unwrap();
        assert!(found.is_none(), "soft-deleted menu should not be found");
    }

    #[sqlx::test]
    async fn soft_delete_system_menu_returns_false(pool: PgPool) {
        // id=2 is a system menu (is_system = true)
        let deleted = MenuRepository::soft_delete(&pool, 2).await.unwrap();
        assert!(!deleted, "system menus cannot be soft-deleted");

        // Verify it still exists
        let found = MenuRepository::find_by_id(&pool, 2).await.unwrap();
        assert!(found.is_some());
    }
}
