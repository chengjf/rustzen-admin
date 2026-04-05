use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::common::{error::ServiceError, validation::*};

/// Request payload for user authentication.
#[derive(Deserialize, TS)]
#[ts(export)]
pub struct LoginRequest {
    /// Username or email for authentication
    pub username: String,
    /// User's password in plain text
    pub password: String,
}

/// Response payload for successful user login.
#[derive(Debug, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LoginResp {
    /// JWT token for authenticating subsequent requests
    pub token: String,
    /// User information
    pub user_info: UserInfoResp,
}

/// Comprehensive user information for authenticated sessions.
#[derive(Debug, Default, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserInfoResp {
    /// Unique identifier of the user
    pub id: i64,
    /// Username of the user
    pub username: String,
    /// Full/display name of the user (optional)
    pub real_name: Option<String>,
    /// Email of the user
    pub email: Option<String>,
    /// Avatar URL of the user
    pub avatar_url: Option<String>,
    /// Whether the user is a system user
    pub is_system: bool,
    /// List of permission codes the user has access to
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ChangePasswordPayload {
    pub old_password: String,
    pub new_password: String,
}

impl LoginRequest {
    pub fn validate(&self) -> Result<(), ServiceError> {
        validate_non_empty("用户名", &self.username, 100)?;
        if self.password.is_empty() {
            return Err(ServiceError::InvalidOperation("密码不能为空".into()));
        }
        Ok(())
    }
}

impl ChangePasswordPayload {
    pub fn validate(&self) -> Result<(), ServiceError> {
        if self.old_password.is_empty() {
            return Err(ServiceError::InvalidOperation("旧密码不能为空".into()));
        }
        validate_password("新密码", &self.new_password)?;
        if self.old_password == self.new_password {
            return Err(ServiceError::InvalidOperation("新密码不能与旧密码相同".into()));
        }
        Ok(())
    }
}
