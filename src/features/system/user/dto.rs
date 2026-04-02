use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use chrono::Utc;

use crate::{common::api::OptionItem, features::auth::model::UserStatus};

use super::model::UserWithRolesEntity;

/// Create user request parameters
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateUserDto {
    pub username: String,
    pub email: String,
    pub password: String,
    pub real_name: Option<String>,
    /// User status. Defaults to Normal.
    pub status: Option<UserStatus>,
    /// A list of role IDs to assign to the user. If empty, will use default role.
    #[serde(default)]
    pub role_ids: Vec<i64>,
}

/// Update user request parameters
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateUserPayload {
    pub email: String,
    pub real_name: Option<String>,
    /// A list of role IDs to assign to the user. If provided, replaces all existing roles.
    pub role_ids: Vec<i64>,
}

/// User list query parameters
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserQuery {
    /// The page number to retrieve. Defaults to 1.
    pub current: Option<i64>,
    /// The number of items per page. Defaults to 10.
    pub page_size: Option<i64>,
    /// Filter by username (case-insensitive search).
    pub username: Option<String>,
    /// Filter by user status. Omit to return all.
    pub status: Option<UserStatus>,
    /// Filter by real name (case-insensitive search).
    pub real_name: Option<String>,
    /// Filter by email (case-insensitive search).
    pub email: Option<String>,
}

/// User options query parameters
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserOptionsQuery {
    /// Search keyword
    pub q: Option<String>,
    /// Maximum number of results to return
    pub limit: Option<i64>,
    /// Filter by user status
    pub status: Option<UserStatus>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateUserPasswordPayload {}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ResetPasswordResp {
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateUserStatusPayload {
    pub status: UserStatus,
}

/// User item for list display
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserItemResp {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub real_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: UserStatus,
    /// ISO-8601 timestamp when an auto-lockout expires.
    /// `None` for users that are not auto-locked.
    /// Frontend uses this to show remaining lock duration.
    #[ts(type = "string | null")]
    pub lock_expires_at: Option<chrono::DateTime<Utc>>,
    pub last_login_at: Option<NaiveDateTime>,
    pub roles: Vec<UserOptionResp>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// User option
pub type UserOptionResp = OptionItem<i64>;

impl From<UserWithRolesEntity> for UserItemResp {
    fn from(user: UserWithRolesEntity) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            real_name: user.real_name,
            avatar_url: user.avatar_url,
            status: UserStatus::try_from(user.effective_status).unwrap_or(UserStatus::Normal),
            lock_expires_at: user.locked_until.filter(|&t| t > Utc::now()),
            last_login_at: user.last_login_at,
            created_at: user.created_at,
            updated_at: user.updated_at,
            roles: serde_json::from_value::<Vec<UserOptionResp>>(user.roles).unwrap_or_default(),
        }
    }
}
