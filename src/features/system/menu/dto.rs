use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::common::{error::ServiceError, validation::*};

use super::model::MenuEntity;

/// Menu type enum constants
#[derive(Debug, Clone, Copy, PartialEq, Eq, TS)]
#[repr(i16)]
#[ts(export)]
pub enum MenuType {
    Directory = 1,
    Menu = 2,
    Button = 3,
}

impl From<MenuType> for i16 {
    fn from(t: MenuType) -> Self {
        t as i16
    }
}

/// Create menu request parameters
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateMenuDto {
    pub parent_id: i64,
    pub name: String,
    pub code: String,
    pub menu_type: i16,
    pub sort_order: i16,
    pub status: i16,
}

/// Update menu request parameters
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateMenuPayload {
    pub parent_id: i64,
    pub name: String,
    pub code: String,
    pub menu_type: i16,
    pub sort_order: i16,
    pub status: i16,
}

/// Menu query parameters
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MenuQuery {
    /// The name of the menu.
    pub name: Option<String>,
    /// The code of the menu.
    pub code: Option<String>,
    /// The status of the menu.
    pub status: Option<String>,
    /// The type of the menu.
    pub menu_type: Option<i16>,
}

/// Menu item for tree list display
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MenuItemResp {
    pub id: i64,
    pub parent_id: i64,
    pub name: String,
    pub code: String,
    pub menu_type: i16,
    pub status: i16,
    pub is_system: bool,
    pub sort_order: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub children: Option<Vec<MenuItemResp>>,
}

impl CreateMenuDto {
    pub fn validate(&self) -> Result<(), ServiceError> {
        validate_non_empty("菜单名称", &self.name, 100)?;
        validate_non_empty("菜单编码", &self.code, 100)?;
        validate_i16_in("菜单类型", self.menu_type, &[1, 2, 3])?;
        validate_i16_in("菜单状态", self.status, &[1, 2])?;
        Ok(())
    }
}

impl UpdateMenuPayload {
    pub fn validate(&self) -> Result<(), ServiceError> {
        validate_non_empty("菜单名称", &self.name, 100)?;
        validate_non_empty("菜单编码", &self.code, 100)?;
        validate_i16_in("菜单类型", self.menu_type, &[1, 2, 3])?;
        validate_i16_in("菜单状态", self.status, &[1, 2])?;
        Ok(())
    }
}

impl MenuQuery {
    pub fn validate(&self) -> Result<(), ServiceError> {
        if let Some(status) = &self.status {
            match status.as_str() {
                "1" | "2" => {}
                _ => return Err(ServiceError::InvalidOperation("菜单状态取值非法".into())),
            }
        }
        if let Some(menu_type) = self.menu_type {
            validate_i16_in("菜单类型", menu_type, &[1, 2, 3])?;
        }
        Ok(())
    }
}

impl From<MenuEntity> for MenuItemResp {
    fn from(entity: MenuEntity) -> Self {
        Self {
            id: entity.id,
            parent_id: entity.parent_id,
            name: entity.name,
            code: entity.code,
            menu_type: entity.menu_type,
            is_system: entity.is_system,
            sort_order: entity.sort_order,
            status: entity.status,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            children: None,
        }
    }
}
