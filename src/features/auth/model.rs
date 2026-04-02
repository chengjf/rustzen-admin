use crate::common::error::ServiceError;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Minimal user info for authentication (login)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LoginCredentialsEntity {
    pub id: i64,
    pub password_hash: String,
    pub status: i16,
    pub is_system: bool,
    /// Number of consecutive failed login attempts since last success or unlock
    pub failed_login_attempts: i16,
    /// Account locked until this time; None means not auto-locked
    pub locked_until: Option<chrono::DateTime<chrono::Utc>>,
}

/// Basic user info for session/profile
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuthUserEntity {
    pub id: i64,
    pub username: String,
    pub real_name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub is_system: bool,
}

/// User status enum for authentication and account control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum UserStatus {
    /// Account is active and can log in
    Normal = 1,
    /// Manually disabled by an administrator
    Disabled = 2,
    /// Awaiting administrator approval
    Pending = 3,
    /// Manually locked by an administrator
    Locked = 4,
}

impl From<UserStatus> for i16 {
    fn from(s: UserStatus) -> i16 {
        s as i16
    }
}

impl TryFrom<i16> for UserStatus {
    type Error = ServiceError;

    /// Convert i16 to UserStatus, returns error if value is invalid.
    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(UserStatus::Normal),
            2 => Ok(UserStatus::Disabled),
            3 => Ok(UserStatus::Pending),
            4 => Ok(UserStatus::Locked),
            _ => Err(ServiceError::InvalidUserStatus),
        }
    }
}

impl UserStatus {
    /// Checks if the user status allows login.
    /// Returns Ok(()) if allowed, or an appropriate ServiceError otherwise.
    pub fn check_status(&self) -> Result<(), ServiceError> {
        match self {
            UserStatus::Normal => Ok(()),
            UserStatus::Disabled => Err(ServiceError::UserIsDisabled),
            UserStatus::Pending => Err(ServiceError::UserIsPending),
            UserStatus::Locked => Err(ServiceError::UserIsLocked),
        }
    }
}

/// Type alias for login credentials model.
pub type LoginCredentials = LoginCredentialsEntity;
/// Type alias for auth user model.
pub type AuthUser = AuthUserEntity;
