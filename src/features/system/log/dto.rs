use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use crate::common::{error::ServiceError, validation::validate_pagination};

use super::model::LogEntity;

/// Log query parameters
#[derive(Debug, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LogQuery {
    pub current: Option<i64>,
    pub page_size: Option<i64>,
    pub username: Option<String>,
    pub action: Option<String>,
    pub description: Option<String>,
    pub ip_address: Option<String>,
}

/// Log item for list display
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LogItemResp {
    pub id: i64,
    pub user_id: i64,
    pub username: String,
    pub action: String,
    pub description: Option<String>,
    pub data: Option<Value>,
    pub status: String,
    pub duration_ms: i32,
    pub ip_address: String,
    pub user_agent: String,
    pub created_at: NaiveDateTime,
}

impl LogQuery {
    pub fn validate(&self) -> Result<(), ServiceError> {
        validate_pagination(self.current, self.page_size)
    }
}

impl From<LogEntity> for LogItemResp {
    fn from(entity: LogEntity) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            username: entity.username,
            action: entity.action,
            description: entity.description,
            data: entity.data,
            ip_address: entity.ip_address.to_string(),
            user_agent: entity.user_agent,
            status: entity.status,
            duration_ms: entity.duration_ms,
            created_at: entity.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn log_entity_converts_to_response() {
        let now = chrono::Utc::now().naive_utc();
        let entity = LogEntity {
            id: 1,
            user_id: 2,
            username: "tester".to_string(),
            action: "LOGIN".to_string(),
            description: Some("user login".to_string()),
            data: Some(serde_json::json!({"ok": true})),
            status: "SUCCESS".to_string(),
            duration_ms: 42,
            ip_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            user_agent: "test-agent".to_string(),
            created_at: now,
        };

        let response = LogItemResp::from(entity);
        assert_eq!(response.id, 1);
        assert_eq!(response.user_id, 2);
        assert_eq!(response.username, "tester");
        assert_eq!(response.ip_address, "127.0.0.1");
        assert_eq!(response.duration_ms, 42);
        assert_eq!(response.status, "SUCCESS");
        assert_eq!(response.created_at, now);
    }
}
