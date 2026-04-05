use super::{
    dto::{LogItemResp, LogQuery},
    model::LogEntity,
    repo::{LogListQuery, LogRepository},
};
use crate::common::{error::ServiceError, pagination::Pagination};

use sqlx::PgPool;
/// A service for log-related operations
pub struct LogService;

impl LogService {
    /// Retrieves a paginated list of system logs
    pub async fn get_log_list(
        pool: &PgPool,
        query: LogQuery,
    ) -> Result<(Vec<LogItemResp>, i64), ServiceError> {
        let (limit, offset, _) = Pagination::normalize(query.current, query.page_size);
        let repo_query = LogListQuery {
            username: query.username,
            action: query.action,
            description: query.description,
            ip_address: query.ip_address,
        };

        let (logs, total) =
            LogRepository::find_with_pagination(pool, offset, limit, repo_query).await?;
        let list: Vec<LogItemResp> = logs.into_iter().map(LogItemResp::from).collect();

        Ok((list, total))
    }

    /// Logs an HTTP request (for middleware use only)
    /// This should be called by the logging middleware, not by business logic.
    pub async fn log_http_request(
        pool: &PgPool,
        method: &str,
        uri: &str,
        user_id: Option<i64>,
        username: Option<&str>,
        ip_address: &str,
        user_agent: &str,
        status_code: u16,
        duration_ms: i32,
    ) -> Result<(), ServiceError> {
        let action = format!("HTTP_{}", method);
        let status = if status_code < 400 { "SUCCESS" } else { "ERROR" };
        let description = format!("{} {} - {}", method, uri, status_code);

        tracing::info!("Logging HTTP request: {} {} - {}", method, uri, status_code);

        // Use the detailed method for HTTP requests
        let _ = LogRepository::create_with_details(
            pool,
            user_id.unwrap_or(0), // Use 0 for anonymous users
            username.unwrap_or("anonymous"),
            Some(&action),
            Some(&description),
            None,
            Some(status),
            Some(duration_ms),
            Some(ip_address),
            Some(user_agent),
        )
        .await?;

        Ok(())
    }

    /// Logs a business operation (for explicit CRUD, not for HTTP middleware)
    pub async fn log_business_operation(
        pool: &PgPool,
        user_id: i64,
        username: &str,
        action: &str,
        description: &str,
        data: serde_json::Value,
        status: &str,
        duration_ms: i32,
        ip_address: &str,
        user_agent: &str,
    ) -> Result<(), ServiceError> {
        let _ = LogRepository::create_with_details(
            pool,
            user_id,
            username,
            Some(action),
            Some(description),
            Some(data),
            Some(status),
            Some(duration_ms),
            Some(ip_address),
            Some(user_agent),
        )
        .await?;

        Ok(())
    }

    pub async fn get_all_log_csv(pool: &PgPool, query: LogQuery) -> Result<String, ServiceError> {
        let repo_query = LogListQuery {
            username: query.username,
            action: query.action,
            description: query.description,
            ip_address: query.ip_address,
        };
        let logs = LogRepository::find_all(pool, repo_query).await?;
        Self::create_csv_chunk(logs, true).await
    }

    /// Create CSV chunk for a batch of logs
    async fn create_csv_chunk(
        logs: Vec<LogEntity>,
        include_header: bool,
    ) -> Result<String, ServiceError> {
        let mut csv_content = String::new();

        // Add CSV header if this is the first batch
        if include_header {
            csv_content
                .push_str("ID,user_id,username,action,description,status,duration_ms,ip_address,user_agent,created_at\n");
        }

        // Add data rows
        for log in logs {
            let row = format!(
                "{},{},{},{},{},{},{},{},{},{}\n",
                log.id,
                log.user_id,
                Self::escape_csv_field(&log.username),
                Self::escape_csv_field(&log.action),
                Self::escape_csv_field(log.description.as_deref().unwrap_or("")),
                Self::escape_csv_field(&log.status),
                log.duration_ms,
                Self::escape_csv_field(&log.ip_address.to_string()),
                Self::escape_csv_field(&log.user_agent),
                log.created_at.format("%Y-%m-%d %H:%M:%S")
            );
            csv_content.push_str(&row);
        }

        Ok(csv_content)
    }

    /// Escape CSV field to handle commas, quotes, and newlines
    fn escape_csv_field(field: &str) -> String {
        if field.contains(',')
            || field.contains('"')
            || field.contains('\n')
            || field.contains('\r')
        {
            // Escape quotes by doubling them and wrap in quotes
            let escaped = field.replace('"', "\"\"");
            format!("\"{}\"", escaped)
        } else {
            field.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::system::log::{dto::LogQuery, repo::LogRepository};
    use sqlx::PgPool;

    fn empty_query() -> LogQuery {
        LogQuery {
            current: None,
            page_size: None,
            username: None,
            action: None,
            description: None,
            ip_address: None,
        }
    }

    // ── DB-level service tests ────────────────────────────────────────────

    #[sqlx::test]
    async fn log_http_request_writes_success_log(pool: PgPool) {
        LogService::log_http_request(
            &pool,
            "GET",
            "/api/users",
            Some(1),
            Some("admin"),
            "127.0.0.1",
            "test-agent",
            200,
            15,
        )
        .await
        .expect("should succeed");

        let (logs, total) = LogRepository::find_with_pagination(
            &pool,
            0,
            10,
            crate::features::system::log::repo::LogListQuery {
                username: Some("admin".to_string()),
                action: None,
                description: None,
                ip_address: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(total, 1);
        assert_eq!(logs[0].action, "HTTP_GET");
        assert_eq!(logs[0].status, "SUCCESS");
    }

    #[sqlx::test]
    async fn log_http_request_marks_error_for_4xx(pool: PgPool) {
        LogService::log_http_request(
            &pool,
            "POST",
            "/api/auth/login",
            None,
            None,
            "10.0.0.1",
            "curl/7.0",
            401,
            5,
        )
        .await
        .unwrap();

        let (logs, _) = LogRepository::find_with_pagination(
            &pool,
            0,
            10,
            crate::features::system::log::repo::LogListQuery {
                username: Some("anonymous".to_string()),
                action: None,
                description: None,
                ip_address: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(logs[0].status, "ERROR");
        assert_eq!(logs[0].action, "HTTP_POST");
    }

    #[sqlx::test]
    async fn log_business_operation_writes_to_db(pool: PgPool) {
        LogService::log_business_operation(
            &pool,
            1,
            "admin",
            "CREATE_USER",
            "创建用户 testuser",
            serde_json::json!({"user_id": 42}),
            "SUCCESS",
            30,
            "192.168.1.1",
            "AdminUI/1.0",
        )
        .await
        .expect("should succeed");

        let (logs, total) = LogRepository::find_with_pagination(
            &pool,
            0,
            10,
            crate::features::system::log::repo::LogListQuery {
                username: None,
                action: Some("CREATE_USER".to_string()),
                description: None,
                ip_address: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(total, 1);
        assert_eq!(logs[0].username, "admin");
    }

    #[sqlx::test]
    async fn get_all_log_csv_contains_header_and_rows(pool: PgPool) {
        LogService::log_business_operation(
            &pool,
            1,
            "csvuser",
            "EXPORT",
            "导出日志",
            serde_json::json!({}),
            "SUCCESS",
            10,
            "127.0.0.1",
            "agent",
        )
        .await
        .unwrap();

        let csv = LogService::get_all_log_csv(&pool, empty_query()).await.unwrap();

        assert!(csv.starts_with("ID,user_id,username"), "should have CSV header");
        assert!(csv.contains("csvuser"), "should contain data row");
        assert!(csv.contains("EXPORT"), "should contain action");
    }

    // ── pure unit tests ───────────────────────────────────────────────────

    #[test]
    fn plain_field_unchanged() {
        assert_eq!(LogService::escape_csv_field("hello"), "hello");
    }

    #[test]
    fn field_with_comma_is_quoted() {
        assert_eq!(LogService::escape_csv_field("a,b"), "\"a,b\"");
    }

    #[test]
    fn field_with_newline_is_quoted() {
        assert_eq!(LogService::escape_csv_field("line1\nline2"), "\"line1\nline2\"");
    }

    #[test]
    fn field_with_carriage_return_is_quoted() {
        assert_eq!(LogService::escape_csv_field("line1\rline2"), "\"line1\rline2\"");
    }

    #[test]
    fn field_with_quote_is_doubled_and_wrapped() {
        assert_eq!(LogService::escape_csv_field("say \"hello\""), "\"say \"\"hello\"\"\"");
    }

    #[test]
    fn empty_field_unchanged() {
        assert_eq!(LogService::escape_csv_field(""), "");
    }
}
