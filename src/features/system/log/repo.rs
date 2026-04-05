use super::model::LogEntity;
use crate::common::error::ServiceError;

use sqlx::{PgPool, QueryBuilder};

/// Log data access layer
pub struct LogRepository;

#[derive(Debug, Clone)]
pub struct LogListQuery {
    pub username: Option<String>,
    pub action: Option<String>,
    pub description: Option<String>,
    pub ip_address: Option<String>,
}

impl LogRepository {
    fn format_query(query: &LogListQuery, query_builder: &mut QueryBuilder<'_, sqlx::Postgres>) {
        if let Some(username) = &query.username {
            if !username.trim().is_empty() {
                query_builder.push(" AND username ILIKE ").push_bind(format!("%{}%", username));
            }
        }
        if let Some(action) = &query.action {
            if !action.trim().is_empty() {
                query_builder.push(" AND action ILIKE ").push_bind(format!("%{}%", action));
            }
        }
        if let Some(description) = &query.description {
            if !description.trim().is_empty() {
                query_builder
                    .push(" AND description ILIKE ")
                    .push_bind(format!("%{}%", description));
            }
        }
        if let Some(ip_address) = &query.ip_address {
            if !ip_address.trim().is_empty() {
                query_builder
                    .push(" AND ip_address::text ILIKE ")
                    .push_bind(format!("%{}%", ip_address));
            }
        }
    }

    /// Count logs matching filters
    async fn count_logs(pool: &PgPool, query: &LogListQuery) -> Result<i64, ServiceError> {
        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM operation_logs WHERE 1=1");

        Self::format_query(&query, &mut query_builder);

        let count: (i64,) = query_builder.build_query_as().fetch_one(pool).await.map_err(|e| {
            tracing::error!("Database error counting users: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        tracing::info!("user count: {:?}", count);

        Ok(count.0)
    }

    /// Find logs with pagination and filters
    pub async fn find_with_pagination(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        query: LogListQuery,
    ) -> Result<(Vec<LogEntity>, i64), ServiceError> {
        tracing::debug!("Finding users with pagination and filters: {:?}", query);
        let total = Self::count_logs(pool, &query).await?;
        if total == 0 {
            return Ok((Vec::new(), total));
        }

        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT * FROM operation_logs WHERE 1=1");

        Self::format_query(&query, &mut query_builder);

        query_builder.push(" ORDER BY created_at DESC");
        query_builder.push(" LIMIT ").push_bind(limit);
        query_builder.push(" OFFSET ").push_bind(offset);

        let logs = query_builder.build_query_as().fetch_all(pool).await.map_err(|e| {
            tracing::error!("Database error in operation_logs pagination: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok((logs, total))
    }

    /// Creates a new log entry with full details (for business operations)
    pub async fn create_with_details(
        pool: &PgPool,
        user_id: i64,
        username: &str,
        action: Option<&str>,
        description: Option<&str>,
        data: Option<serde_json::Value>,
        status: Option<&str>,
        duration_ms: Option<i32>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<i64, ServiceError> {
        tracing::debug!("Creating detailed log entry with action: {:?}", action);

        let log_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO operation_logs \
             (user_id, username, action, description, data, status, duration_ms, ip_address, user_agent) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8::inet, $9) \
             RETURNING id",
        )
        .bind(user_id)
        .bind(username)
        .bind(action)
        .bind(description)
        .bind(data)
        .bind(status)
        .bind(duration_ms)
        .bind(ip_address)
        .bind(user_agent)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error creating detailed log: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(log_id)
    }

    pub async fn find_all(
        pool: &PgPool,
        query: LogListQuery,
    ) -> Result<Vec<LogEntity>, ServiceError> {
        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT * FROM operation_logs WHERE 1=1");

        Self::format_query(&query, &mut query_builder);

        let logs = query_builder.build_query_as().fetch_all(pool).await.map_err(|e| {
            tracing::error!("Database error in operation_logs pagination: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(logs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    async fn seed_log(pool: &PgPool, username: &str, action: &str) -> i64 {
        LogRepository::create_with_details(
            pool,
            1,
            username,
            Some(action),
            Some("test description"),
            None,
            Some("SUCCESS"),
            Some(10),
            Some("127.0.0.1"),
            Some("test-agent"),
        )
        .await
        .expect("create log should succeed")
    }

    #[sqlx::test]
    async fn insert_and_query_log(pool: PgPool) {
        let id = seed_log(&pool, "testuser", "LOGIN").await;
        assert!(id > 0);

        let (logs, total) = LogRepository::find_with_pagination(
            &pool,
            0,
            10,
            LogListQuery { username: None, action: None, description: None, ip_address: None },
        )
        .await
        .unwrap();

        assert_eq!(total, 1);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].username, "testuser");
        assert_eq!(logs[0].action, "LOGIN");
    }

    #[sqlx::test]
    async fn query_with_filters(pool: PgPool) {
        seed_log(&pool, "alice", "CREATE_USER").await;
        seed_log(&pool, "bob", "DELETE_USER").await;

        let (logs, total) = LogRepository::find_with_pagination(
            &pool,
            0,
            10,
            LogListQuery {
                username: Some("alice".to_string()),
                action: None,
                description: None,
                ip_address: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(total, 1);
        assert_eq!(logs[0].username, "alice");

        let (_, all_total) = LogRepository::find_with_pagination(
            &pool,
            0,
            10,
            LogListQuery {
                username: Some("_USER".to_string()),
                action: None,
                description: None,
                ip_address: None,
            },
        )
        .await
        .unwrap();
        // Neither alice nor bob has "_USER" in username, but action filter is not username
        // The logs count remains by username filter
        assert_eq!(all_total, 0);

        // Filter by action
        let (action_logs, action_total) = LogRepository::find_with_pagination(
            &pool,
            0,
            10,
            LogListQuery {
                username: None,
                action: Some("DELETE".to_string()),
                description: None,
                ip_address: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(action_total, 1);
        assert_eq!(action_logs[0].username, "bob");
    }
}
