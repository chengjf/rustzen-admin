use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

/// User with roles (for view-based queries)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserWithRolesEntity {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub real_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: i16,
    pub locked_until: Option<DateTime<Utc>>,
    /// Computed by the view: merges `status` and `locked_until` into one value.
    /// Auto-locked users (status=1 but locked_until > NOW()) map to 4 (Locked).
    /// Use this field for display and filtering; never write it to the DB.
    pub effective_status: i16,
    pub last_login_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub roles: serde_json::Value,
}

/// Type alias: repository/API use this name for the user-with-roles model.
pub type User = UserWithRolesEntity;
