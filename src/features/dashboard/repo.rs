use super::dto::{StatsResp, SystemMetricsDataResp, TrendResp, UserTrendsResp};
use crate::{common::error::ServiceError, features::auth::model::UserStatus};
use sqlx::PgPool;

pub struct DashboardRepository;

impl DashboardRepository {
    pub async fn get_stats(pool: &PgPool) -> Result<StatsResp, ServiceError> {
        // 并行执行所有查询
        let (
            total_users,
            active_users,
            today_logins,
            system_uptime,
            pending_users,
        ) = tokio::join!(
            // 获取总用户数
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
                .fetch_one(pool),

            // 获取活跃用户数（7天内登录）
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE last_login_at > NOW() - INTERVAL '7 days' AND deleted_at IS NULL"
            )
            .fetch_one(pool),

            // 获取今日登录数
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE last_login_at > NOW() - INTERVAL '1 day' AND deleted_at IS NULL"
            )
            .fetch_one(pool),

            // 计算系统运行时间
            // EXTRACT(MINUTES FROM (NOW() - pg_postmaster_start_time()))::text || '分钟'
            sqlx::query_scalar::<_, String>(
                r#"
                SELECT
                    EXTRACT(DAYS FROM (NOW() - pg_postmaster_start_time()))::text || '天 ' ||
                    EXTRACT(HOURS FROM (NOW() - pg_postmaster_start_time()))::text || '小时 '
                "#
            )
            .fetch_one(pool),

            // 获取待审核用户数
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE status = $1 AND deleted_at IS NULL"
            )
            .bind(UserStatus::Pending as i16)
            .fetch_one(pool)
        );

        // 处理查询结果
        let total_users = total_users.map_err(|e| {
            tracing::error!("Database error getting total users: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let active_users = active_users.map_err(|e| {
            tracing::error!("Database error getting active users: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let today_logins = today_logins.map_err(|e| {
            tracing::error!("Database error getting today logins: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let system_uptime = system_uptime.map_err(|e| {
            tracing::error!("Database error getting system uptime: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let pending_users = pending_users.map_err(|e| {
            tracing::error!("Database error getting pending users: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let stats =
            StatsResp { total_users, active_users, today_logins, system_uptime, pending_users };
        Ok(stats)
    }

    pub async fn get_metrics(pool: &PgPool) -> Result<SystemMetricsDataResp, ServiceError> {
        // 并行获取系统指标
        let (
            total_requests,
            error_requests,
            avg_response_time,
        ) = tokio::join!(
            // 获取总请求数（从日志表统计）
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM operation_logs WHERE created_at > NOW() - INTERVAL '7 days'"
            )
            .fetch_one(pool),

            // 获取错误请求数（状态为 FAILED 或 ERROR）
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM operation_logs WHERE status IN ('FAILED', 'ERROR') AND created_at > NOW() - INTERVAL '7 days'"
            )
            .fetch_one(pool),

            // 获取平均响应时间（毫秒）
            sqlx::query_scalar::<_, Option<f64>>(
                "SELECT AVG(duration_ms::FLOAT8) FROM operation_logs WHERE created_at > NOW() - INTERVAL '7 days' AND duration_ms IS NOT NULL"
            )
            .fetch_one(pool)
        );

        // 处理查询结果
        let total_requests = total_requests.map_err(|e| {
            tracing::error!("Database error getting total requests: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let error_requests = error_requests.map_err(|e| {
            tracing::error!("Database error getting error requests: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let avg_response_time = avg_response_time.map_err(|e| {
            tracing::error!("Database error getting avg response time: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        // 计算错误率
        let error_rate = if total_requests > 0 {
            (error_requests as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        // 计算平均响应时间（毫秒）
        let avg_response_time_ms = avg_response_time.unwrap_or(0.0) as i64;

        let metrics = SystemMetricsDataResp {
            avg_response_time: avg_response_time_ms,
            error_rate: error_rate,
            total_requests,
        };

        Ok(metrics)
    }

    pub async fn get_trends(pool: &PgPool) -> Result<UserTrendsResp, ServiceError> {
        // 并行获取趋势数据
        let (daily_logins, hourly_active) = tokio::join!(
            // 获取最近30天的登录趋势
            Self::get_daily_login_trends(pool),
            // 获取24小时活跃用户分布
            Self::get_hourly_active_users(pool)
        );

        let daily_logins = daily_logins?;
        let hourly_active = hourly_active?;

        Ok(UserTrendsResp { daily_logins, hourly_active })
    }

    /// 获取每日登录趋势（最近30天）
    async fn get_daily_login_trends(pool: &PgPool) -> Result<Vec<TrendResp>, ServiceError> {
        let daily_logins = sqlx::query_as(
            r#"
            SELECT
                DATE(created_at)::TEXT as date,
                COUNT(*) as count
            FROM operation_logs
            WHERE action = 'AUTH_LOGIN'
                AND status = 'SUCCESS'
                AND created_at > NOW() - INTERVAL '30 days'
            GROUP BY DATE(created_at)
            ORDER BY date
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error getting daily login trends: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(daily_logins)
    }

    /// 获取24小时活跃用户分布
    async fn get_hourly_active_users(pool: &PgPool) -> Result<Vec<TrendResp>, ServiceError> {
        let hourly_active: Vec<TrendResp> = sqlx::query_as(
            r#"
            WITH hour_series AS (
                SELECT generate_series(0, 23) as hour
            )
            SELECT
                hs.hour::TEXT as date,
                COALESCE(COUNT(DISTINCT ol.user_id), 0) as count
            FROM hour_series hs
            LEFT JOIN operation_logs ol ON EXTRACT(HOUR FROM ol.created_at) = hs.hour
                AND ol.created_at > NOW() - INTERVAL '24 hours'
                AND ol.user_id IS NOT NULL
            GROUP BY hs.hour
            ORDER BY hs.hour
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error getting hourly active users: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(hourly_active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::password::PasswordUtils;
    use crate::features::system::user::repo::{CreateUserCommand, UserRepository};
    use chrono::{Duration, Utc};
    use sqlx::PgPool;

    async fn seed_user(pool: &PgPool, username: &str, status: i16) -> i64 {
        UserRepository::create_user(
            pool,
            &CreateUserCommand {
                username: username.to_string(),
                email: format!("{username}@example.com"),
                password_hash: PasswordUtils::hash_password("Dash@Test1").unwrap(),
                real_name: Some(username.to_string()),
                status: Some(status),
                role_ids: vec![],
            },
        )
        .await
        .expect("seed user should succeed")
    }

    async fn insert_log(
        pool: &PgPool,
        user_id: Option<i64>,
        username: &str,
        action: &str,
        status: &str,
        duration_ms: Option<i32>,
        created_at: chrono::DateTime<Utc>,
    ) {
        sqlx::query(
            "INSERT INTO operation_logs
             (user_id, username, action, description, data, status, duration_ms, ip_address, user_agent, created_at)
             VALUES ($1, $2, $3, 'test', '{}'::jsonb, $4, $5, '127.0.0.1', 'test-agent', $6)",
        )
        .bind(user_id)
        .bind(username)
        .bind(action)
        .bind(status)
        .bind(duration_ms)
        .bind(created_at.naive_utc())
        .execute(pool)
        .await
        .unwrap();
    }

    #[sqlx::test]
    async fn get_stats_counts_recent_and_pending_users(pool: PgPool) {
        let active_id = seed_user(&pool, "dash_active", 1).await;
        let stale_id = seed_user(&pool, "dash_stale", 1).await;
        let _pending_id = seed_user(&pool, "dash_pending", UserStatus::Pending as i16).await;

        sqlx::query("UPDATE users SET last_login_at = NOW() - INTERVAL '2 hours' WHERE id = $1")
            .bind(active_id)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("UPDATE users SET last_login_at = NOW() - INTERVAL '10 days' WHERE id = $1")
            .bind(stale_id)
            .execute(&pool)
            .await
            .unwrap();

        let stats = DashboardRepository::get_stats(&pool).await.unwrap();

        assert!(stats.total_users >= 4);
        assert!(stats.active_users >= 1);
        assert!(stats.today_logins >= 1);
        assert!(stats.pending_users >= 1);
        assert!(stats.system_uptime.contains('天') || stats.system_uptime.contains("小时"));
    }

    #[sqlx::test]
    async fn get_metrics_returns_zeroes_without_recent_logs(pool: PgPool) {
        let metrics = DashboardRepository::get_metrics(&pool).await.unwrap();

        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.avg_response_time, 0);
        assert_eq!(metrics.error_rate, 0.0);
    }

    #[sqlx::test]
    async fn get_metrics_calculates_error_rate_and_average_duration(pool: PgPool) {
        let user_id = seed_user(&pool, "dash_metric_user", 1).await;
        let now = Utc::now();

        insert_log(
            &pool,
            Some(user_id),
            "dash_metric_user",
            "AUTH_LOGIN",
            "SUCCESS",
            Some(100),
            now,
        )
        .await;
        insert_log(
            &pool,
            Some(user_id),
            "dash_metric_user",
            "AUTH_LOGIN",
            "FAILED",
            Some(200),
            now,
        )
        .await;
        insert_log(&pool, Some(user_id), "dash_metric_user", "AUTH_LOGIN", "ERROR", Some(300), now)
            .await;

        let metrics = DashboardRepository::get_metrics(&pool).await.unwrap();

        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.avg_response_time, 200);
        assert!((metrics.error_rate - 66.6666666667).abs() < 0.01);
    }

    #[sqlx::test]
    async fn get_trends_returns_daily_login_and_hourly_activity(pool: PgPool) {
        let user_id = seed_user(&pool, "dash_trend_user", 1).await;
        let now = Utc::now();

        insert_log(
            &pool,
            Some(user_id),
            "dash_trend_user",
            "AUTH_LOGIN",
            "SUCCESS",
            Some(120),
            now - Duration::days(1),
        )
        .await;
        insert_log(&pool, Some(user_id), "dash_trend_user", "AUTH_LOGIN", "SUCCESS", Some(80), now)
            .await;
        insert_log(
            &pool,
            Some(user_id),
            "dash_trend_user",
            "SYSTEM_VIEW",
            "SUCCESS",
            Some(30),
            now,
        )
        .await;

        let trends = DashboardRepository::get_trends(&pool).await.unwrap();

        assert_eq!(trends.hourly_active.len(), 24);
        assert!(trends.daily_logins.len() >= 2);
        assert!(trends.daily_logins.iter().all(|item| item.count.unwrap_or(0) >= 1));
        assert!(trends.hourly_active.iter().any(|item| item.count.unwrap_or(0) >= 1));
    }
}
