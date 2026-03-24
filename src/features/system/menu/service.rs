use std::collections::HashMap;

use super::{
    dto::{CreateMenuDto, MenuItemResp, MenuQuery, UpdateMenuPayload},
    repo::{MenuListQuery, MenuRepository},
};
use crate::common::{
    api::{OptionItem, OptionsQuery, OptionsWithCodeQuery},
    error::ServiceError,
};

use serde::Serialize;
use sqlx::PgPool;

pub struct MenuService;

impl MenuService {
    /// Get menu list as tree structure with optional filtering
    pub async fn get_menu_list(
        pool: &PgPool,
        query: MenuQuery,
    ) -> Result<(Vec<MenuItemResp>, i64), ServiceError> {
        tracing::info!("Fetching menu list with query: {:?}", query);

        let repo_query = MenuListQuery { name: query.name, code: query.code, status: query.status };

        let menus = MenuRepository::find_all(pool, repo_query).await?;

        let menu_responses: Vec<MenuItemResp> = menus.into_iter().map(MenuItemResp::from).collect();
        let count = menu_responses.len() as i64;

        Ok((menu_responses, count))
    }

    /// Create new menu with validation
    pub async fn create_menu(pool: &PgPool, request: CreateMenuDto) -> Result<i64, ServiceError> {
        tracing::info!("Attempting to create menu with name: {}", request.name);
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

        Self::validate_menu_type_update(pool, id, &request).await?;

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

        let success = MenuRepository::soft_delete(pool, id).await?;

        if success {
            tracing::info!("Successfully deleted menu: {}", id);
            Ok(())
        } else {
            Err(ServiceError::NotFound("Menu".to_string()))
        }
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
            if request.menu_type != 1 && request.menu_type != 2 {
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
    ) -> Result<(), ServiceError> {
        // 我的父节点不能是我
        if request.parent_id == id {
            return Err(ServiceError::InvalidOperation(format!("父级菜单不能是自己",)));
        }
        // 检查当前id对应的菜单是否存在
        let menu = MenuRepository::find_by_id(pool, id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("菜单不存在: {}", id)))?;
        // 检查父节点和子节点的类型约束
        if request.parent_id != 0 {
            let parent =
                MenuRepository::find_by_id(pool, request.parent_id).await?.ok_or_else(|| {
                    ServiceError::NotFound(format!("父级菜单不存在: {}", request.parent_id))
                })?;
            // 如果父级是目录，那么当前菜单只能是目录或菜单
            // 1: 目录
            // 2: 菜单
            // 3: 按钮
            Self::check_type_constraint(parent.menu_type, request.menu_type)?;
        } else {
            // 如果父级菜单是0，那么当前菜单只能是目录或菜单
            if request.menu_type != 1 && request.menu_type != 2 {
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
        match parent_type {
            1 => {
                // 目录
                if child_type != 1 && child_type != 2 {
                    return Err(ServiceError::InvalidOperation(
                        "父级是目录，子级必须是目录或菜单".into(),
                    ));
                }
            }
            2 => {
                // 菜单
                if child_type != 3 {
                    return Err(ServiceError::InvalidOperation(
                        "父级是菜单，子级必须是按钮".into(),
                    ));
                }
            }
            3 => {
                // 按钮
                return Err(ServiceError::InvalidOperation("按钮不能作为父级".into()));
            }
            _ => return Err(ServiceError::InvalidOperation("未知的父级类型".into())),
        }
        Ok(())
    }
}

/// Menu tree option item for permission selection
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuTreeOption {
    pub label: String,
    pub value: i64,
    pub code: String,
    pub parent_id: i64,
    pub menu_type: i16,
    pub children: Option<Vec<MenuTreeOption>>,
}
