use super::model::RoleWithMenuEntity;
use crate::common::{error::ServiceError, status::EnableStatus};

use chrono::Utc;
use sqlx::{PgPool, QueryBuilder};

pub struct RoleRepository;

#[derive(Debug, Clone)]
pub struct RoleListQuery {
    pub name: Option<String>,
    pub code: Option<String>,
    pub status: Option<String>,
}

impl RoleRepository {
    /// Return which role IDs from the given list exist and are enabled.
    pub async fn find_existing_role_ids(
        pool: &PgPool,
        role_ids: &[i64],
    ) -> Result<Vec<i64>, ServiceError> {
        use crate::common::status::EnableStatus;
        let ids: Vec<(i64,)> = sqlx::query_as(
            "SELECT id FROM roles WHERE id = ANY($1) AND status = $2 AND deleted_at IS NULL",
        )
        .bind(role_ids)
        .bind(EnableStatus::Enabled as i16)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding role IDs: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(ids.into_iter().map(|(id,)| id).collect())
    }

    /// Find role by ID (returns None if not found or deleted)
    pub async fn find_by_id(
        pool: &PgPool,
        id: i64,
    ) -> Result<Option<RoleWithMenuEntity>, ServiceError> {
        sqlx::query_as::<_, RoleWithMenuEntity>(
            "SELECT * FROM role_with_menus WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding role by ID {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })
    }

    fn format_query(query: &RoleListQuery, query_builder: &mut QueryBuilder<'_, sqlx::Postgres>) {
        if let Some(name) = &query.name {
            if !name.trim().is_empty() {
                query_builder.push(" AND name ILIKE ").push_bind(format!("%{}%", name));
            }
        }
        if let Some(code) = &query.code {
            if !code.trim().is_empty() {
                query_builder.push(" AND code ILIKE ").push_bind(format!("%{}%", code));
            }
        }
        if let Some(status) = &query.status {
            if let Ok(status_num) = status.parse::<i16>() {
                query_builder.push(" AND status = ").push_bind(status_num);
            }
        }
    }

    /// Count users matching filters
    async fn count_roles(pool: &PgPool, query: &RoleListQuery) -> Result<i64, ServiceError> {
        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM role_with_menus WHERE 1=1");

        Self::format_query(&query, &mut query_builder);

        let count: (i64,) = query_builder.build_query_as().fetch_one(pool).await.map_err(|e| {
            tracing::error!("Database error counting users: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(count.0)
    }

    /// Queries roles with pagination
    pub async fn find_with_pagination(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        query: RoleListQuery,
    ) -> Result<(Vec<RoleWithMenuEntity>, i64), ServiceError> {
        let total = Self::count_roles(pool, &query).await?;
        if total == 0 {
            return Ok((Vec::new(), total));
        }

        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT * FROM role_with_menus WHERE 1=1");

        Self::format_query(&query, &mut query_builder);

        query_builder.push(" ORDER BY sort_order ASC, created_at DESC");
        query_builder.push(" LIMIT ").push_bind(limit);
        query_builder.push(" OFFSET ").push_bind(offset);

        let roles = query_builder.build_query_as().fetch_all(pool).await.map_err(|e| {
            tracing::error!("Database error in user_with_roles pagination: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok((roles, total))
    }

    /// Creates a new role
    pub async fn create(
        pool: &PgPool,
        role_name: &str,
        role_code: &str,
        description: Option<&str>,
        status: i16,
        sort_order: i32,
        menu_ids: &[i64],
    ) -> Result<i64, ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for role creation: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let role_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO roles (name, code, description, status, sort_order, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id",
        )
        .bind(role_name)
        .bind(role_code)
        .bind(description)
        .bind(status)
        .bind(sort_order)
        .bind(Utc::now().naive_utc())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error creating role: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Self::insert_role_menus(&mut tx, role_id, menu_ids).await?;

        tx.commit().await.map_err(|e| {
            tracing::error!("Database error committing role creation transaction: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(role_id)
    }

    /// Updates an existing role
    pub async fn update(
        pool: &PgPool,
        id: i64,
        role_name: &str,
        role_code: &str,
        description: Option<&str>,
        status: i16,
        sort_order: i32,
        menu_ids: &[i64],
    ) -> Result<i64, ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for role update: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        // update role
        let id_opt = sqlx::query_scalar::<_, i64>(
            "UPDATE roles
                 SET name = $1, code = $2, description = $3, status = $4, sort_order = $5
                 WHERE id = $6 AND deleted_at IS NULL
                 RETURNING id",
        )
        .bind(role_name)
        .bind(role_code)
        .bind(description)
        .bind(status)
        .bind(sort_order)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error updating role: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        if let Some(id) = id_opt {
            // update role_menus
            Self::insert_role_menus(&mut tx, id, menu_ids).await?;
            tx.commit().await.map_err(|e| {
                tracing::error!("Database error committing role update transaction: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;
            Ok(id)
        } else {
            Err(ServiceError::NotFound(format!("Role id: {}", id)))
        }
    }

    /// Soft deletes a role
    pub async fn soft_delete(pool: &PgPool, id: i64) -> Result<bool, ServiceError> {
        let result =
            sqlx::query("UPDATE roles SET deleted_at = $1 WHERE id = $2 AND deleted_at IS NULL")
                .bind(Utc::now().naive_utc())
                .bind(id)
                .execute(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database error soft deleting role {}: {:?}", id, e);
                    ServiceError::DatabaseQueryFailed
                })?;

        Ok(result.rows_affected() > 0)
    }

    /// insert role_menus
    async fn insert_role_menus(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        role_id: i64,
        menu_ids: &[i64],
    ) -> Result<(), ServiceError> {
        sqlx::query("DELETE FROM role_menus WHERE role_id = $1")
            .bind(role_id)
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                tracing::error!("Database error deleting existing role_menus: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;
        if menu_ids.is_empty() {
            return Ok(());
        }
        let now = Utc::now().naive_utc();
        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("INSERT INTO role_menus (role_id, menu_id, created_at) ");
        query_builder.push_values(menu_ids, |mut b, menu_id| {
            b.push_bind(role_id).push_bind(menu_id).push_bind(now);
        });
        query_builder.build().execute(&mut **tx).await.map_err(|e| {
            tracing::error!("Database error inserting role_menus: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(())
    }

    /// Retrieves role list for Options API
    pub async fn find_options(
        pool: &PgPool,
        search_query: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(i64, String)>, ServiceError> {
        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT id, name FROM roles WHERE status = ");
        query_builder.push_bind(EnableStatus::Enabled as i16);
        query_builder.push(" AND deleted_at IS NULL");

        if let Some(keyword) = search_query {
            if !keyword.trim().is_empty() {
                query_builder.push(" AND name ILIKE ").push_bind(format!("%{}%", keyword));
            }
        }

        query_builder.push(" ORDER BY name ASC");

        if let Some(l) = limit {
            query_builder.push(" LIMIT ").push_bind(l);
        }

        let results: Vec<(i64, String)> =
            query_builder.build_query_as().fetch_all(pool).await.map_err(|e| {
                tracing::error!("Database error finding role options: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;
        Ok(results)
    }

    pub async fn get_role_user_count(pool: &PgPool, role_id: i64) -> Result<i64, ServiceError> {
        let result =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM user_roles ur JOIN users u ON u.id = ur.user_id AND u.deleted_at IS NULL WHERE ur.role_id = $1")
                .bind(role_id)
                .fetch_one(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database error getting role user count: {:?}", e);
                    ServiceError::DatabaseQueryFailed
                })?;
        Ok(result)
    }

    pub async fn count_by_name(pool: &PgPool, name: &str) -> Result<i64, ServiceError> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM roles WHERE name = $1 AND deleted_at IS NULL",
        )
        .bind(name)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error counting roles by name: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(result)
    }

    pub async fn name_exists_exclude_self(
        pool: &PgPool,
        name: &str,
        exclude_id: i64,
    ) -> Result<bool, ServiceError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM roles WHERE name = $1 AND id != $2 AND deleted_at IS NULL)",
        )
        .bind(name)
        .bind(exclude_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!(
                "Database error checking role name existence (exclude self) '{}': {:?}",
                name,
                e
            );
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(exists)
    }

    pub async fn count_by_code(pool: &PgPool, role_code: &str) -> Result<i64, ServiceError> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM roles WHERE code = $1 AND deleted_at IS NULL",
        )
        .bind(role_code)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error counting roles: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(result)
    }

    pub async fn code_exists_exclude_self(
        pool: &PgPool,
        code: &str,
        exclude_id: i64,
    ) -> Result<bool, ServiceError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM roles WHERE code = $1 AND id != $2 AND deleted_at IS NULL)",
        )
        .bind(code)
        .bind(exclude_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!(
                "Database error checking role code existence (exclude self) '{}': {:?}",
                code,
                e
            );
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    fn empty_query() -> RoleListQuery {
        RoleListQuery { name: None, code: None, status: None }
    }

    async fn seed_role(pool: &PgPool, name: &str, code: &str) -> i64 {
        RoleRepository::create(pool, name, code, None, 1, 0, &[])
            .await
            .expect("create role should succeed")
    }

    #[sqlx::test]
    async fn create_role_and_find_by_id(pool: PgPool) {
        let id = seed_role(&pool, "测试角色", "TEST_ROLE").await;
        assert!(id > 0);

        let found = RoleRepository::find_by_id(&pool, id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "测试角色");
    }

    #[sqlx::test]
    async fn find_by_id_returns_none_for_missing(pool: PgPool) {
        let found = RoleRepository::find_by_id(&pool, 999_999).await.unwrap();
        assert!(found.is_none());
    }

    #[sqlx::test]
    async fn find_with_pagination_filters_by_status(pool: PgPool) {
        RoleRepository::create(&pool, "分页角色A", "ROLE_PAGE_A", None, 1, 2, &[3]).await.unwrap();
        RoleRepository::create(&pool, "分页角色B", "ROLE_PAGE_B", None, 2, 3, &[]).await.unwrap();

        let query = RoleListQuery { name: None, code: None, status: Some("1".to_string()) };

        let (roles, total) =
            RoleRepository::find_with_pagination(&pool, 0, 10, query).await.unwrap();

        assert!(total >= 1);
        assert!(!roles.is_empty());
        assert!(roles.iter().all(|role| role.status == 1));
    }

    #[sqlx::test]
    async fn find_with_pagination_filters_by_name_and_code(pool: PgPool) {
        let matched_id =
            RoleRepository::create(&pool, "筛选角色甲", "ROLE_FILTER_ALPHA", None, 1, 0, &[])
                .await
                .unwrap();
        RoleRepository::create(&pool, "筛选角色乙", "ROLE_OTHER_BETA", None, 1, 0, &[])
            .await
            .unwrap();

        let query = RoleListQuery {
            name: Some("筛选角色甲".to_string()),
            code: Some("FILTER_ALPHA".to_string()),
            status: Some("1".to_string()),
        };

        let (roles, total) =
            RoleRepository::find_with_pagination(&pool, 0, 10, query).await.unwrap();

        assert_eq!(total, 1);
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].id, matched_id);
        assert_eq!(roles[0].name, "筛选角色甲");
        assert_eq!(roles[0].code, "ROLE_FILTER_ALPHA");
    }

    #[sqlx::test]
    async fn find_with_pagination_ignores_blank_and_invalid_filters(pool: PgPool) {
        let role_id = seed_role(&pool, "空筛选角色", "ROLE_BLANK_FILTER").await;

        let query = RoleListQuery {
            name: Some("   ".to_string()),
            code: Some(String::new()),
            status: Some("invalid".to_string()),
        };

        let (roles, total) =
            RoleRepository::find_with_pagination(&pool, 0, 50, query).await.unwrap();

        assert!(total > 0);
        assert!(roles.iter().any(|role| role.id == role_id));
    }

    #[sqlx::test]
    async fn find_with_pagination_returns_empty_when_offset_exceeds_total(pool: PgPool) {
        seed_role(&pool, "偏移分页角色", "ROLE_PAGE_OFFSET").await;

        let (_, total) =
            RoleRepository::find_with_pagination(&pool, 0, 50, empty_query()).await.unwrap();
        let (roles, total_again) =
            RoleRepository::find_with_pagination(&pool, total + 10, 10, empty_query())
                .await
                .unwrap();

        assert!(total_again >= 1);
        assert!(roles.is_empty());
    }

    #[sqlx::test]
    async fn count_by_name_detects_duplicate(pool: PgPool) {
        seed_role(&pool, "重复角色", "DUP_ROLE").await;

        let count = RoleRepository::count_by_name(&pool, "重复角色").await.unwrap();
        assert_eq!(count, 1);

        let zero = RoleRepository::count_by_name(&pool, "不存在角色").await.unwrap();
        assert_eq!(zero, 0);
    }

    #[sqlx::test]
    async fn count_by_code_detects_duplicate(pool: PgPool) {
        seed_role(&pool, "编码测试角色", "CODE_TEST").await;

        let count = RoleRepository::count_by_code(&pool, "CODE_TEST").await.unwrap();
        assert_eq!(count, 1);

        let zero = RoleRepository::count_by_code(&pool, "NO_SUCH_CODE").await.unwrap();
        assert_eq!(zero, 0);
    }

    #[sqlx::test]
    async fn name_exists_exclude_self(pool: PgPool) {
        let id = seed_role(&pool, "排除自身角色", "EXCL_SELF").await;

        // Same name but excluding self → should be false
        let exists =
            RoleRepository::name_exists_exclude_self(&pool, "排除自身角色", id).await.unwrap();
        assert!(!exists, "should not detect self as duplicate");

        // Different ID → should be true
        let another_id = seed_role(&pool, "另一个角色", "ANOTHER_ROLE").await;
        let exists2 = RoleRepository::name_exists_exclude_self(&pool, "排除自身角色", another_id)
            .await
            .unwrap();
        assert!(exists2, "should detect another role with same name");
    }

    #[sqlx::test]
    async fn code_exists_exclude_self(pool: PgPool) {
        let id = seed_role(&pool, "编码排除角色", "CODE_EXCL").await;

        let exists =
            RoleRepository::code_exists_exclude_self(&pool, "CODE_EXCL", id).await.unwrap();
        assert!(!exists, "should not detect self as code duplicate");

        let another_id = seed_role(&pool, "另一角色", "ANOTHER_CODE").await;
        let exists2 =
            RoleRepository::code_exists_exclude_self(&pool, "CODE_EXCL", another_id).await.unwrap();
        assert!(exists2, "should detect another role with same code");
    }

    #[sqlx::test]
    async fn find_existing_role_ids_filters_disabled(pool: PgPool) {
        // Create one enabled role and one disabled role
        let enabled_id =
            RoleRepository::create(&pool, "启用角色", "ENABLED_R", None, 1, 0, &[]).await.unwrap();
        let disabled_id =
            RoleRepository::create(&pool, "禁用角色", "DISABLED_R", None, 2, 0, &[]).await.unwrap();

        let found = RoleRepository::find_existing_role_ids(&pool, &[enabled_id, disabled_id])
            .await
            .unwrap();

        assert!(found.contains(&enabled_id), "enabled role should be found");
        assert!(!found.contains(&disabled_id), "disabled role should be excluded");
    }

    #[sqlx::test]
    async fn find_existing_role_ids_returns_empty_for_empty_input(pool: PgPool) {
        let found = RoleRepository::find_existing_role_ids(&pool, &[]).await.unwrap();
        assert!(found.is_empty());
    }

    #[sqlx::test]
    async fn update_role_replaces_fields_and_menus(pool: PgPool) {
        let role_id = RoleRepository::create(&pool, "原角色", "ROLE_OLD", Some("old"), 1, 1, &[3])
            .await
            .unwrap();

        let updated_id = RoleRepository::update(
            &pool,
            role_id,
            "新角色",
            "ROLE_NEW",
            Some("new"),
            2,
            9,
            &[10, 15],
        )
        .await
        .unwrap();
        assert_eq!(updated_id, role_id);

        let role = RoleRepository::find_by_id(&pool, role_id).await.unwrap().unwrap();
        assert_eq!(role.name, "新角色");
        assert_eq!(role.code, "ROLE_NEW");
        assert_eq!(role.description.as_deref(), Some("new"));
        assert_eq!(role.status, 2);
        assert_eq!(role.sort_order, 9);

        let menu_ids: Vec<i64> = sqlx::query_scalar(
            "SELECT menu_id FROM role_menus WHERE role_id = $1 ORDER BY menu_id",
        )
        .bind(role_id)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(menu_ids, vec![10, 15]);
    }

    #[sqlx::test]
    async fn update_role_can_clear_description_and_menus(pool: PgPool) {
        let role_id =
            RoleRepository::create(&pool, "可清空角色", "ROLE_CLEAR", Some("has-desc"), 1, 1, &[3])
                .await
                .unwrap();

        RoleRepository::update(&pool, role_id, "可清空角色", "ROLE_CLEAR", None, 1, 1, &[])
            .await
            .unwrap();

        let role = RoleRepository::find_by_id(&pool, role_id).await.unwrap().unwrap();
        assert!(role.description.is_none());

        let menu_ids: Vec<i64> = sqlx::query_scalar(
            "SELECT menu_id FROM role_menus WHERE role_id = $1 ORDER BY menu_id",
        )
        .bind(role_id)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert!(menu_ids.is_empty());
        assert_eq!(role.menus, serde_json::json!([]));
    }

    #[sqlx::test]
    async fn update_missing_role_returns_not_found(pool: PgPool) {
        let result =
            RoleRepository::update(&pool, 999_999, "missing", "MISSING", None, 1, 1, &[]).await;
        assert!(matches!(result, Err(ServiceError::NotFound(_))));
    }

    #[sqlx::test]
    async fn find_options_returns_enabled_roles_only_and_honors_limit(pool: PgPool) {
        RoleRepository::create(&pool, "选项角色A", "ROLE_OPT_A", None, 1, 1, &[]).await.unwrap();
        RoleRepository::create(&pool, "选项角色B", "ROLE_OPT_B", None, 2, 1, &[]).await.unwrap();

        let options =
            RoleRepository::find_options(&pool, Some("选项角色"), Some(10)).await.unwrap();

        assert!(!options.is_empty());
        assert!(options.iter().any(|(_, name)| name == "选项角色A"));
        assert!(!options.iter().any(|(_, name)| name == "选项角色B"));
    }

    #[sqlx::test]
    async fn find_options_ignores_blank_search_query(pool: PgPool) {
        let role_id = seed_role(&pool, "空查询角色", "ROLE_EMPTY_QUERY").await;
        let options = RoleRepository::find_options(&pool, Some("   "), None).await.unwrap();
        assert!(options.iter().any(|(id, _)| *id == role_id));
    }

    #[sqlx::test]
    async fn find_options_orders_by_name_and_honors_limit(pool: PgPool) {
        RoleRepository::create(&pool, "B排序角色", "ROLE_OPT_ORDER_B", None, 1, 1, &[])
            .await
            .unwrap();
        let first_id =
            RoleRepository::create(&pool, "A排序角色", "ROLE_OPT_ORDER_A", None, 1, 1, &[])
                .await
                .unwrap();

        let options = RoleRepository::find_options(&pool, Some("排序角色"), Some(1)).await.unwrap();

        assert_eq!(options.len(), 1);
        assert_eq!(options[0].0, first_id);
        assert_eq!(options[0].1, "A排序角色");
    }

    #[sqlx::test]
    async fn get_role_user_count(pool: PgPool) {
        let role_id = seed_role(&pool, "统计角色", "COUNT_ROLE").await;

        // No users assigned initially
        let count = RoleRepository::get_role_user_count(&pool, role_id).await.unwrap();
        assert_eq!(count, 0);

        // Seed system admin role - it has 1 user (superadmin)
        let (sys_role_id,): (i64,) =
            sqlx::query_as("SELECT id FROM roles WHERE code = 'SYSTEM_ADMIN'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let sys_count = RoleRepository::get_role_user_count(&pool, sys_role_id).await.unwrap();
        assert_eq!(sys_count, 1);
    }

    #[sqlx::test]
    async fn soft_delete_makes_role_unfindable(pool: PgPool) {
        let id = seed_role(&pool, "待删角色", "TO_DELETE").await;

        let deleted = RoleRepository::soft_delete(&pool, id).await.unwrap();
        assert!(deleted);

        let found = RoleRepository::find_by_id(&pool, id).await.unwrap();
        assert!(found.is_none(), "soft-deleted role should not be found");
    }

    #[sqlx::test]
    async fn soft_delete_returns_false_when_role_is_already_deleted(pool: PgPool) {
        let id = seed_role(&pool, "重复删除角色", "TO_DELETE_TWICE").await;

        assert!(RoleRepository::soft_delete(&pool, id).await.unwrap());
        assert!(!RoleRepository::soft_delete(&pool, id).await.unwrap());
    }
}
