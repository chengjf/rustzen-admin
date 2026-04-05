use std::collections::HashMap;

use super::{
    dto::{CreateMenuDto, MenuItemResp, MenuQuery, MenuType, UpdateMenuPayload},
    repo::{MenuListQuery, MenuRepository},
};
use crate::{
    common::{
        api::{OptionItem, OptionsQuery, OptionsWithCodeQuery},
        error::ServiceError,
    },
    core::session::SessionStore,
    features::system::user::repo::UserRepository,
};

use chrono::Utc;
use serde::Serialize;
use sqlx::PgPool;
use ts_rs::TS;

pub struct MenuService;

impl MenuService {
    /// Get menu list as tree structure with optional filtering
    pub async fn get_menu_list(
        pool: &PgPool,
        query: MenuQuery,
    ) -> Result<Vec<MenuItemResp>, ServiceError> {
        tracing::info!("Fetching menu list with query: {:?}", query);

        let repo_query = MenuListQuery {
            name: query.name,
            code: query.code,
            status: query.status,
            menu_type: query.menu_type,
        };

        let menus = MenuRepository::find_all(pool, repo_query).await?;

        let menu_responses: Vec<MenuItemResp> = menus.into_iter().map(MenuItemResp::from).collect();

        let tree = Self::build_response_tree(menu_responses);

        Ok(tree)
    }

    /// Build menu response tree from flat list
    fn build_response_tree(items: Vec<MenuItemResp>) -> Vec<MenuItemResp> {
        let mut grouping: HashMap<i64, Vec<MenuItemResp>> = HashMap::new();

        for item in items {
            grouping.entry(item.parent_id).or_default().push(item);
        }

        fn recursive_build(
            parent_id: i64,
            grouping: &mut HashMap<i64, Vec<MenuItemResp>>,
        ) -> Vec<MenuItemResp> {
            if let Some(children_list) = grouping.remove(&parent_id) {
                children_list
                    .into_iter()
                    .map(|mut item| {
                        let sub_children = recursive_build(item.id, grouping);
                        item.children =
                            if sub_children.is_empty() { None } else { Some(sub_children) };
                        item
                    })
                    .collect()
            } else {
                vec![]
            }
        }

        recursive_build(0, &mut grouping)
    }

    /// Create new menu with validation
    pub async fn create_menu(pool: &PgPool, request: CreateMenuDto) -> Result<i64, ServiceError> {
        tracing::info!("Attempting to create menu with name: {}", request.name);

        if MenuRepository::name_exists(pool, &request.name).await? {
            return Err(ServiceError::InvalidOperation(format!(
                "菜单名称 {} 已存在",
                request.name
            )));
        }
        if MenuRepository::code_exists(pool, &request.code).await? {
            return Err(ServiceError::InvalidOperation(format!(
                "菜单编码 {} 已存在",
                request.code
            )));
        }

        Self::validate_menu_type_create(pool, &request).await?;

        let menu_id = MenuRepository::create(
            pool,
            request.parent_id,
            &request.name,
            &request.code,
            request.menu_type,
            request.sort_order,
            request.status,
        )
        .await?;

        tracing::info!("Successfully created menu: {}", menu_id);
        Ok(menu_id)
    }

    /// Update existing menu with validation
    pub async fn update_menu(
        pool: &PgPool,
        id: i64,
        request: UpdateMenuPayload,
    ) -> Result<i64, ServiceError> {
        tracing::info!("Attempting to update menu: {}", id);

        // 检查当前id对应的菜单是否存在
        let menu = MenuRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("菜单不存在: {}", id)))?;

        // 系统菜单保护
        if menu.is_system {
            return Err(ServiceError::InvalidOperation("系统内置菜单不能修改".into()));
        }

        if MenuRepository::name_exists_exclude_self(pool, &request.name, id).await? {
            return Err(ServiceError::InvalidOperation(format!(
                "菜单名称 {} 已存在",
                request.name
            )));
        }
        if MenuRepository::code_exists_exclude_self(pool, &request.code, id).await? {
            return Err(ServiceError::InvalidOperation(format!(
                "菜单编码 {} 已存在",
                request.code
            )));
        }

        Self::validate_menu_type_update(pool, id, &request, &menu).await?;

        let menu_id = MenuRepository::update(
            pool,
            id,
            request.parent_id,
            &request.name,
            &request.code,
            request.menu_type,
            request.sort_order,
            request.status,
        )
        .await?;

        // Invalidate permission caches for all users whose roles include this menu
        match UserRepository::find_user_ids_by_menu_id(pool, id).await {
            Ok(user_ids) => {
                for uid in user_ids {
                    if let Err(e) = SessionStore::delete_by_user_id(pool, uid).await {
                        tracing::error!("Failed to delete session for user_id={} after menu update: {:?}", uid, e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to fetch users for menu_id={} during cache invalidation: {:?}", id, e);
            }
        }

        tracing::info!("Successfully updated menu: {}", menu_id);
        Ok(menu_id)
    }

    /// Delete menu with child validation
    pub async fn delete_menu(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        tracing::info!("Attempting to delete menu: {}", id);
        // 检查是否有子节点
        let children = MenuRepository::find_by_parent_id(pool, id).await?;
        if !children.is_empty() {
            return Err(ServiceError::InvalidOperation(format!("当前菜单有子菜单，不能删除")));
        }

        // Collect affected users before role_menus are deleted inside the transaction
        let affected_users = UserRepository::find_user_ids_by_menu_id(pool, id).await.unwrap_or_else(|e| {
            tracing::error!("Failed to fetch users for menu_id={} during cache invalidation: {:?}", id, e);
            vec![]
        });

        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for menu deletion: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        // Clean up role-menu associations
        sqlx::query("DELETE FROM role_menus WHERE menu_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
            tracing::error!("Database error deleting role_menus for menu {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        // Soft delete menu (only the specific menu, not children)
        let result = sqlx::query(
            "UPDATE menus SET deleted_at = $1, updated_at = $1 WHERE id = $2 AND is_system = false AND deleted_at IS NULL"
        )
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error soft deleting menu {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        if result.rows_affected() == 0 {
            return Err(ServiceError::NotFound("Menu".to_string()));
        }

        tx.commit().await.map_err(|e| {
            tracing::error!("Database error committing menu deletion transaction: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        // Invalidate permission caches for affected users
        for uid in affected_users {
            if let Err(e) = SessionStore::delete_by_user_id(pool, uid).await {
                tracing::error!("Failed to delete session for user_id={} after menu deletion: {:?}", uid, e);
            }
        }

        tracing::info!("Successfully deleted menu: {}", id);
        Ok(())
    }

    /// Get menu options for dropdowns
    pub async fn get_menu_options(
        pool: &PgPool,
        query: OptionsQuery,
    ) -> Result<Vec<OptionItem<i64>>, ServiceError> {
        tracing::info!("Fetching menu options: {:?}", query);

        let menus = MenuRepository::find_options(pool, query.q.as_deref(), query.limit).await?;

        let options: Vec<OptionItem<i64>> =
            menus.into_iter().map(|(id, name)| OptionItem { label: name, value: id }).collect();

        tracing::info!("Successfully retrieved {} menu options", options.len());
        Ok(options)
    }

    /// Get menu options with code for permission tree
    pub async fn get_menu_options_with_code(
        pool: &PgPool,
        query: OptionsWithCodeQuery,
    ) -> Result<Vec<MenuTreeOption>, ServiceError> {
        tracing::info!("Fetching menu options with code for tree: {:?}", query);

        let menus = MenuRepository::find_options_with_code(
            pool,
            query.q.as_deref(),
            query.limit,
            query.btn_filter,
        )
        .await?;

        // Build tree structure
        let menu_items: Vec<MenuTreeOption> = menus
            .into_iter()
            .map(|(id, name, code, parent_id, menu_type)| MenuTreeOption {
                label: name,
                value: id,
                code: code.unwrap_or_default(),
                parent_id,
                menu_type,
                children: None,
            })
            .collect();

        let tree = Self::build_menu_tree(menu_items);

        tracing::info!("Successfully retrieved {} menu tree options", tree.len());
        Ok(tree)
    }

    /// Build menu tree from flat list
    fn build_menu_tree(items: Vec<MenuTreeOption>) -> Vec<MenuTreeOption> {
        // 1. 使用 HashMap 按 parent_id 进行分组
        // Key: parent_id, Value: 该父级下的所有子节点列表
        let mut grouping: HashMap<i64, Vec<MenuTreeOption>> = HashMap::new();

        for item in items {
            grouping.entry(item.parent_id).or_default().push(item);
        }

        // 2. 定义递归闭包（内联函数）来递归组装
        // 使用递归函数从指定父 ID 开始构建
        fn recursive_build(
            parent_id: i64,
            grouping: &mut HashMap<i64, Vec<MenuTreeOption>>,
        ) -> Vec<MenuTreeOption> {
            // 从 Map 中移除并获取当前层级的子节点，避免多次借用
            if let Some(children_list) = grouping.remove(&parent_id) {
                children_list
                    .into_iter()
                    .map(|mut item| {
                        // 递归获取下一层级
                        let sub_children = recursive_build(item.value, grouping);
                        item.children =
                            if sub_children.is_empty() { None } else { Some(sub_children) };
                        item
                    })
                    .collect()
            } else {
                vec![]
            }
        }

        // 3. 从根节点 (parent_id = 0) 开始构建
        recursive_build(0, &mut grouping)
    }

    /// 新增菜单校验
    async fn validate_menu_type_create(
        pool: &PgPool,
        request: &CreateMenuDto,
    ) -> Result<(), ServiceError> {
        if request.parent_id != 0 {
            let parent =
                MenuRepository::find_by_id(pool, request.parent_id).await?.ok_or_else(|| {
                    ServiceError::NotFound(format!("父级菜单不存在: {}", request.parent_id))
                })?;
            Self::check_type_constraint(parent.menu_type, request.menu_type)?;
        } else {
            // 如果父级菜单是0，那么当前菜单只能是目录或菜单
            if request.menu_type != MenuType::Directory as i16
                && request.menu_type != MenuType::Menu as i16
            {
                return Err(ServiceError::InvalidOperation(format!(
                    "父级菜单是根目录，当前菜单类型必须是目录或菜单"
                )));
            }
        }
        Ok(())
    }

    /// 更新菜单校验
    async fn validate_menu_type_update(
        pool: &PgPool,
        id: i64,
        request: &UpdateMenuPayload,
        menu: &super::model::MenuEntity,
    ) -> Result<(), ServiceError> {
        // 我的父节点不能是我
        if request.parent_id == id {
            return Err(ServiceError::InvalidOperation(format!("父级菜单不能是自己")));
        }

        // 父节点不能是当前节点的后代（防止循环树）
        if request.parent_id != 0 {
            let mut current_id = request.parent_id;
            loop {
                if current_id == id {
                    return Err(ServiceError::InvalidOperation(
                        "父级菜单不能是当前菜单的后代".into(),
                    ));
                }
                match MenuRepository::find_by_id(pool, current_id).await? {
                    None => break,
                    Some(m) if m.parent_id == 0 => break,
                    Some(m) => current_id = m.parent_id,
                }
            }
        }

        // 检查父节点和子节点的类型约束
        if request.parent_id != 0 {
            let parent =
                MenuRepository::find_by_id(pool, request.parent_id).await?.ok_or_else(|| {
                    ServiceError::NotFound(format!("父级菜单不存在: {}", request.parent_id))
                })?;
            Self::check_type_constraint(parent.menu_type, request.menu_type)?;
        } else {
            // 如果父级菜单是0，那么当前菜单只能是目录或菜单
            if request.menu_type != MenuType::Directory as i16
                && request.menu_type != MenuType::Menu as i16
            {
                return Err(ServiceError::InvalidOperation(format!(
                    "父级菜单是根目录，当前菜单类型必须是目录或菜单"
                )));
            }
        }
        // 如果类型修改，判断约束
        if menu.menu_type != request.menu_type {
            // 先获取当前菜单的所有子菜单
            let children = MenuRepository::find_by_parent_id(pool, id).await?;
            if !children.is_empty() {
                return Err(ServiceError::InvalidOperation(format!(
                    "当前菜单有子菜单，不能修改类型"
                )));
            }
        }
        Ok(())
    }

    /// 抽离公共的类型匹配逻辑
    fn check_type_constraint(parent_type: i16, child_type: i16) -> Result<(), ServiceError> {
        if parent_type == MenuType::Directory as i16 {
            if child_type != MenuType::Directory as i16 && child_type != MenuType::Menu as i16 {
                return Err(ServiceError::InvalidOperation(
                    "父级是目录，子级必须是目录或菜单".into(),
                ));
            }
        } else if parent_type == MenuType::Menu as i16 {
            if child_type != MenuType::Button as i16 {
                return Err(ServiceError::InvalidOperation("父级是菜单，子级必须是按钮".into()));
            }
        } else if parent_type == MenuType::Button as i16 {
            return Err(ServiceError::InvalidOperation("按钮不能作为父级".into()));
        } else {
            return Err(ServiceError::InvalidOperation("未知的父级类型".into()));
        }
        Ok(())
    }
}

/// Menu tree option item for permission selection
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MenuTreeOption {
    pub label: String,
    pub value: i64,
    pub code: String,
    pub parent_id: i64,
    pub menu_type: i16,
    pub children: Option<Vec<MenuTreeOption>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        common::error::ServiceError,
        features::system::menu::dto::{CreateMenuDto, MenuType, UpdateMenuPayload},
    };
    use sqlx::PgPool;

    fn dir() -> i16 {
        MenuType::Directory as i16
    }
    fn menu() -> i16 {
        MenuType::Menu as i16
    }
    fn btn() -> i16 {
        MenuType::Button as i16
    }

    fn make_create_dto(parent_id: i64, name: &str, code: &str, menu_type: i16) -> CreateMenuDto {
        CreateMenuDto { parent_id, name: name.to_string(), code: code.to_string(), menu_type, sort_order: 1, status: 1 }
    }

    // ── create_menu ──────────────────────────────────────────────────────

    #[sqlx::test]
    async fn create_directory_at_root_succeeds(pool: PgPool) {
        let id = MenuService::create_menu(&pool, make_create_dto(0, "测试目录", "test:root", dir()))
            .await
            .expect("should succeed");
        assert!(id > 0);
    }

    #[sqlx::test]
    async fn create_button_at_root_returns_error(pool: PgPool) {
        let result =
            MenuService::create_menu(&pool, make_create_dto(0, "根按钮", "root:btn", btn())).await;
        assert!(matches!(result, Err(ServiceError::InvalidOperation(_))));
    }

    #[sqlx::test]
    async fn create_menu_duplicate_name_returns_error(pool: PgPool) {
        // id=2 name="系统管理" already exists in seed data
        let result =
            MenuService::create_menu(&pool, make_create_dto(0, "系统管理", "new:code", dir())).await;
        assert!(matches!(result, Err(ServiceError::InvalidOperation(_))));
    }

    #[sqlx::test]
    async fn create_menu_duplicate_code_returns_error(pool: PgPool) {
        // id=2 code="system" already exists in seed data
        let result =
            MenuService::create_menu(&pool, make_create_dto(0, "新目录名", "system", dir())).await;
        assert!(matches!(result, Err(ServiceError::InvalidOperation(_))));
    }

    #[sqlx::test]
    async fn create_button_under_menu_succeeds(pool: PgPool) {
        // id=3 is a Menu (system:user:list), buttons are valid children
        let id =
            MenuService::create_menu(&pool, make_create_dto(3, "新按钮", "test:new:btn", btn()))
                .await
                .expect("button under menu should succeed");
        assert!(id > 0);
    }

    #[sqlx::test]
    async fn create_button_under_directory_returns_error(pool: PgPool) {
        // id=2 is a Directory; buttons can't be direct children of a directory
        let result =
            MenuService::create_menu(&pool, make_create_dto(2, "错误按钮", "dir:btn", btn())).await;
        assert!(matches!(result, Err(ServiceError::InvalidOperation(_))));
    }

    // ── update_menu ──────────────────────────────────────────────────────

    #[sqlx::test]
    async fn update_menu_changes_name(pool: PgPool) {
        let id = MenuService::create_menu(&pool, make_create_dto(0, "原名称", "upd:code", dir()))
            .await
            .unwrap();

        MenuService::update_menu(
            &pool,
            id,
            UpdateMenuPayload {
                parent_id: 0,
                name: "新名称".to_string(),
                code: "upd:code".to_string(),
                menu_type: dir(),
                sort_order: 1,
                status: 1,
            },
        )
        .await
        .expect("update should succeed");

        let updated = MenuRepository::find_by_id(&pool, id).await.unwrap().unwrap();
        assert_eq!(updated.name, "新名称");
    }

    #[sqlx::test]
    async fn update_system_menu_returns_error(pool: PgPool) {
        // id=2 (系统管理) is a system menu
        let result = MenuService::update_menu(
            &pool,
            2,
            UpdateMenuPayload {
                parent_id: 0,
                name: "黑客改名".to_string(),
                code: "hack".to_string(),
                menu_type: dir(),
                sort_order: 1,
                status: 1,
            },
        )
        .await;
        assert!(matches!(result, Err(ServiceError::InvalidOperation(_))));
    }

    #[sqlx::test]
    async fn update_menu_self_reference_returns_error(pool: PgPool) {
        let id = MenuService::create_menu(&pool, make_create_dto(0, "自引用", "self:ref", dir()))
            .await
            .unwrap();

        let result = MenuService::update_menu(
            &pool,
            id,
            UpdateMenuPayload {
                parent_id: id, // parent = self
                name: "自引用".to_string(),
                code: "self:ref".to_string(),
                menu_type: dir(),
                sort_order: 1,
                status: 1,
            },
        )
        .await;
        assert!(matches!(result, Err(ServiceError::InvalidOperation(_))));
    }

    #[sqlx::test]
    async fn update_menu_cycle_detection_returns_error(pool: PgPool) {
        // Create: root → child1 → child2
        let root =
            MenuService::create_menu(&pool, make_create_dto(0, "根", "cyc:root", dir())).await.unwrap();
        let child1 =
            MenuService::create_menu(&pool, make_create_dto(root, "子1", "cyc:c1", dir())).await.unwrap();
        let child2 =
            MenuService::create_menu(&pool, make_create_dto(child1, "子2", "cyc:c2", dir()))
                .await
                .unwrap();

        // Trying to set root's parent to child2 would create a cycle
        let result = MenuService::update_menu(
            &pool,
            root,
            UpdateMenuPayload {
                parent_id: child2, // cycle: root → ... → child2 → root
                name: "根".to_string(),
                code: "cyc:root".to_string(),
                menu_type: dir(),
                sort_order: 1,
                status: 1,
            },
        )
        .await;
        assert!(
            matches!(result, Err(ServiceError::InvalidOperation(_))),
            "cycle detection should block this update"
        );
    }

    // ── delete_menu ──────────────────────────────────────────────────────

    #[sqlx::test]
    async fn delete_menu_succeeds(pool: PgPool) {
        let id = MenuService::create_menu(&pool, make_create_dto(0, "待删目录", "del:dir", dir()))
            .await
            .unwrap();

        MenuService::delete_menu(&pool, id).await.expect("delete should succeed");

        let found = MenuRepository::find_by_id(&pool, id).await.unwrap();
        assert!(found.is_none());
    }

    #[sqlx::test]
    async fn delete_menu_with_children_returns_error(pool: PgPool) {
        // id=2 (系统管理) has children — this is non-system so we create our own tree
        let parent =
            MenuService::create_menu(&pool, make_create_dto(0, "父目录", "par:dir", dir())).await.unwrap();
        MenuService::create_menu(&pool, make_create_dto(parent, "子菜单", "child:menu", menu()))
            .await
            .unwrap();

        let result = MenuService::delete_menu(&pool, parent).await;
        assert!(
            matches!(result, Err(ServiceError::InvalidOperation(_))),
            "should block deletion when menu has children"
        );
    }

    #[sqlx::test]
    async fn delete_system_menu_returns_not_found(pool: PgPool) {
        // id=4 (用户创建) is a leaf system button (no children, is_system=true).
        // The SQL filter `is_system = false` means rows_affected = 0 → NotFound.
        let result = MenuService::delete_menu(&pool, 4).await;
        assert!(matches!(result, Err(ServiceError::NotFound(_))));
    }

    // Directory parent
    #[test]
    fn directory_can_contain_directory() {
        assert!(MenuService::check_type_constraint(dir(), dir()).is_ok());
    }

    #[test]
    fn directory_can_contain_menu() {
        assert!(MenuService::check_type_constraint(dir(), menu()).is_ok());
    }

    #[test]
    fn directory_cannot_contain_button() {
        let err = MenuService::check_type_constraint(dir(), btn());
        assert!(err.is_err());
    }

    // Menu parent
    #[test]
    fn menu_can_contain_button() {
        assert!(MenuService::check_type_constraint(menu(), btn()).is_ok());
    }

    #[test]
    fn menu_cannot_contain_directory() {
        assert!(MenuService::check_type_constraint(menu(), dir()).is_err());
    }

    #[test]
    fn menu_cannot_contain_menu() {
        assert!(MenuService::check_type_constraint(menu(), menu()).is_err());
    }

    // Button parent
    #[test]
    fn button_cannot_be_parent_of_anything() {
        assert!(MenuService::check_type_constraint(btn(), dir()).is_err());
        assert!(MenuService::check_type_constraint(btn(), menu()).is_err());
        assert!(MenuService::check_type_constraint(btn(), btn()).is_err());
    }

    // Tree building
    #[test]
    fn build_response_tree_nests_children() {
        use crate::features::system::menu::dto::MenuItemResp;
        let now = chrono::Utc::now().naive_utc();
        let items = vec![
            MenuItemResp {
                id: 1,
                parent_id: 0,
                name: "Root".into(),
                code: "root".into(),
                menu_type: dir(),
                status: 1,
                is_system: false,
                sort_order: 1,
                created_at: now,
                updated_at: now,
                children: None,
            },
            MenuItemResp {
                id: 2,
                parent_id: 1,
                name: "Child".into(),
                code: "child".into(),
                menu_type: menu(),
                status: 1,
                is_system: false,
                sort_order: 1,
                created_at: now,
                updated_at: now,
                children: None,
            },
        ];

        let tree = MenuService::build_response_tree(items);
        assert_eq!(tree.len(), 1);
        let root = &tree[0];
        assert_eq!(root.id, 1);
        let children = root.children.as_ref().expect("root should have children");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, 2);
    }
}
