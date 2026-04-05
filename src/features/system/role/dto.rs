use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::common::{api::OptionItem, error::ServiceError, validation::*};

use super::model::RoleWithMenuEntity;

/// Create and update role request parameters
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateRoleDto {
    pub name: String,
    pub code: String,
    pub status: i16,
    pub sort_order: Option<i32>,
    pub menu_ids: Vec<i64>,
    pub description: Option<String>,
}

/// Update role request parameters
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateRolePayload {
    pub name: String,
    pub code: String,
    pub status: i16,
    pub sort_order: Option<i32>,
    pub menu_ids: Vec<i64>,
    pub description: Option<String>,
}

/// Role list query parameters
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RoleQuery {
    /// The page number to retrieve. Defaults to 1.
    pub current: Option<i64>,
    /// The number of items per page. Defaults to 10.
    pub page_size: Option<i64>,
    /// Filter by role name (case-insensitive search).
    pub name: Option<String>,
    /// Filter by role code (case-insensitive search).
    pub code: Option<String>,
    /// Filter by role status.
    pub status: Option<String>,
}

/// Role item for list display
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RoleItemResp {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub status: i16,
    pub sort_order: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub menus: Vec<OptionItem<i64>>,
}

impl CreateRoleDto {
    pub fn validate(&self) -> Result<(), ServiceError> {
        validate_non_empty("角色名称", &self.name, 50)?;
        validate_non_empty("角色编码", &self.code, 50)?;
        validate_optional_text("角色描述", &self.description, 500)?;
        validate_i16_in("角色状态", self.status, &[1, 2])?;
        Ok(())
    }
}

impl UpdateRolePayload {
    pub fn validate(&self) -> Result<(), ServiceError> {
        validate_non_empty("角色名称", &self.name, 50)?;
        validate_non_empty("角色编码", &self.code, 50)?;
        validate_optional_text("角色描述", &self.description, 500)?;
        validate_i16_in("角色状态", self.status, &[1, 2])?;
        Ok(())
    }
}

impl RoleQuery {
    pub fn validate(&self) -> Result<(), ServiceError> {
        validate_pagination(self.current, self.page_size)?;
        if let Some(status) = &self.status {
            match status.as_str() {
                "1" | "2" => {}
                _ => return Err(ServiceError::InvalidOperation("角色状态取值非法".into())),
            }
        }
        Ok(())
    }
}

impl From<RoleWithMenuEntity> for RoleItemResp {
    fn from(role: RoleWithMenuEntity) -> Self {
        Self {
            id: role.id,
            name: role.name,
            code: role.code,
            description: role.description,
            status: role.status,
            sort_order: role.sort_order,
            created_at: role.created_at,
            updated_at: role.updated_at,
            menus: serde_json::from_value::<Vec<OptionItem<i64>>>(role.menus).unwrap_or_default(),
        }
    }
}
