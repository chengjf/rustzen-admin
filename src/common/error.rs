use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

// --- Business Service Errors ---

/// A unified error type for the business logic layer.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// User is disabled.
    #[error("用户已被禁用")]
    UserIsDisabled,

    /// User is pending.
    #[error("用户待审核")]
    UserIsPending,

    /// User is manually locked by an administrator.
    #[error("用户已被锁定")]
    UserIsLocked,

    /// User is temporarily locked after too many failed login attempts.
    /// Contains the remaining minutes until auto-unlock.
    #[error("登录失败次数过多，账号已被锁定，请 {0} 分钟后再试")]
    UserIsAutoLocked(i64),

    /// User status is invalid.
    #[error("用户状态非法")]
    InvalidUserStatus,

    /// User is admin.
    #[error("用户是管理员")]
    UserIsAdmin,

    /// Cannot operate on self.
    #[error("不能操作自己的账号")]
    CannotOperateSelf,

    /// Internal server error.
    // #[error("Internal server error")]
    // InternalServerError,

    /// A database query failed.
    #[error("数据库查询失败")]
    DatabaseQueryFailed,

    /// The requested resource was not found.
    #[error("{0} 不存在")]
    NotFound(String),

    /// The user's credentials were invalid.
    #[error("用户名或密码错误")]
    InvalidCredentials,

    /// The provided JWT was invalid or expired.
    #[error("无效或过期的令牌")]
    InvalidToken,

    /// Failed to generate token.
    #[error("令牌创建失败")]
    TokenCreationFailed,

    /// The user does not have permission to perform this action.
    #[error("权限不足")]
    PermissionDenied,

    /// A username that was provided already exists.
    #[error("用户名已存在")]
    UsernameConflict,

    /// An email that was provided already exists.
    #[error("邮箱已存在")]
    EmailConflict,

    /// An operation was attempted that is invalid given the current state.
    #[error("非法操作: {0}")]
    InvalidOperation(String),

    /// Password hashing failed.
    #[error("密码处理失败")]
    PasswordHashingFailed,

    /// Failed to upload file.
    #[error("创建头像文件夹失败")]
    CreateAvatarFolderFailed,

    /// Failed to create avatar file.
    #[error("创建头像文件失败")]
    CreateAvatarFileFailed,
}

// --- Axum Error Handling ---

/// A unified error type for the application layer, which can be converted into an HTTP response.
#[derive(Debug)]
pub struct AppError((StatusCode, i32, String));

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = self.0;
        let body = Json(serde_json::json!({
            "code": code,
            "message": message,
            "data": null,
        }));
        (status, body).into_response()
    }
}

/// Converts a `ServiceError` into an `AppError`.
/// This is the central place to map business logic errors to HTTP-level errors.
impl From<ServiceError> for AppError {
    fn from(err: ServiceError) -> Self {
        let (status, code, message) = match err {
            // 1xxxx: User/Business Errors
            ServiceError::NotFound(resource) => (
                StatusCode::NOT_FOUND,
                10001, // Business-Common-01
                format!("{} 不存在.", resource),
            ),
            ServiceError::InvalidOperation(reason) => (
                StatusCode::BAD_REQUEST,
                10002, // Business-Common-02
                reason,
            ),
            ServiceError::PasswordHashingFailed => {
                (StatusCode::INTERNAL_SERVER_ERROR, 10003, "密码处理失败，请重试。".into())
            }
            ServiceError::UserIsDisabled => (StatusCode::BAD_REQUEST, 10004, "用户已被禁用".into()),
            ServiceError::UserIsPending => (StatusCode::BAD_REQUEST, 10005, "用户待审核".into()),
            ServiceError::UserIsLocked => (StatusCode::BAD_REQUEST, 10006, "用户已被锁定".into()),
            ServiceError::UserIsAutoLocked(mins) => (
                StatusCode::BAD_REQUEST,
                10010,
                format!("登录失败次数过多，账号已被锁定，请 {} 分钟后再试", mins),
            ),
            ServiceError::InvalidUserStatus => {
                (StatusCode::BAD_REQUEST, 10007, "用户状态非法".into())
            }
            ServiceError::UserIsAdmin => (StatusCode::BAD_REQUEST, 10008, "用户是管理员".into()),
            ServiceError::CannotOperateSelf => {
                (StatusCode::BAD_REQUEST, 10009, "不能操作自己的账号".into())
            }
            ServiceError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                10101, // Business-Auth-01
                "用户名或密码错误".into(),
            ),
            ServiceError::TokenCreationFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                10103, // Business-Auth-03
                "令牌创建失败".into(),
            ),
            ServiceError::UsernameConflict => (
                StatusCode::CONFLICT,
                10201, // Business-User-01
                "用户名已存在".into(),
            ),
            ServiceError::EmailConflict => (
                StatusCode::CONFLICT,
                10202, // Business-User-02
                "邮箱已存在".into(),
            ),
            // 2xxxx: System Errors
            ServiceError::DatabaseQueryFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                20001, // System-Common-01
                "数据库查询失败".into(),
            ),
            ServiceError::CreateAvatarFolderFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                20002, // System-Common-02
                "创建头像文件夹失败".into(),
            ),
            ServiceError::CreateAvatarFileFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                20003, // System-Common-03
                "创建头像文件失败".into(),
            ),
            // ServiceError::InternalServerError => (
            //     StatusCode::INTERNAL_SERVER_ERROR,
            //     20002, // System-Common-02
            //     "Internal server error. Please contact the administrator.".to_string(),
            // ),
            // 3xxxx: Permission Errors
            ServiceError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                30000, // System-Auth-01
                "令牌无效或过期，请重新登录".into(),
            ),
            ServiceError::PermissionDenied => (
                StatusCode::FORBIDDEN,
                30001, // System-Auth-02
                "您没有权限执行此操作".into(),
            ),
        };
        AppError((status, code, message))
    }
}

/// Allows `sqlx::Error` to be converted into `AppError` for convenience in route handlers.
/// This should be used sparingly, prefer mapping to `ServiceError` in the service layer.
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        let service_error = ServiceError::DatabaseQueryFailed;
        service_error.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::response::IntoResponse;
    use serde_json::Value;

    fn status_of(err: ServiceError) -> StatusCode {
        let app_err: AppError = err.into();
        app_err.into_response().status()
    }

    async fn response_parts(err: impl Into<AppError>) -> (StatusCode, Value) {
        let response = Into::<AppError>::into(err).into_response();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.expect("body should be readable");
        let body: Value = serde_json::from_slice(&body).expect("body should be valid json");
        (status, body)
    }

    #[test]
    fn not_found_maps_to_404() {
        assert_eq!(status_of(ServiceError::NotFound("User".into())), StatusCode::NOT_FOUND);
    }

    #[test]
    fn invalid_credentials_maps_to_401() {
        assert_eq!(status_of(ServiceError::InvalidCredentials), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn permission_denied_maps_to_403() {
        assert_eq!(status_of(ServiceError::PermissionDenied), StatusCode::FORBIDDEN);
    }

    #[test]
    fn invalid_token_maps_to_401() {
        assert_eq!(status_of(ServiceError::InvalidToken), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn database_error_maps_to_500() {
        assert_eq!(status_of(ServiceError::DatabaseQueryFailed), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn username_conflict_maps_to_409() {
        assert_eq!(status_of(ServiceError::UsernameConflict), StatusCode::CONFLICT);
    }

    #[test]
    fn invalid_operation_maps_to_400() {
        assert_eq!(
            status_of(ServiceError::InvalidOperation("bad".into())),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn user_is_auto_locked_maps_to_400() {
        assert_eq!(status_of(ServiceError::UserIsAutoLocked(5)), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn service_errors_map_to_expected_payloads() {
        let cases = vec![
            (
                ServiceError::PasswordHashingFailed,
                StatusCode::INTERNAL_SERVER_ERROR,
                10003,
                "密码处理失败，请重试。",
            ),
            (ServiceError::UserIsDisabled, StatusCode::BAD_REQUEST, 10004, "用户已被禁用"),
            (ServiceError::UserIsPending, StatusCode::BAD_REQUEST, 10005, "用户待审核"),
            (ServiceError::UserIsLocked, StatusCode::BAD_REQUEST, 10006, "用户已被锁定"),
            (ServiceError::InvalidUserStatus, StatusCode::BAD_REQUEST, 10007, "用户状态非法"),
            (ServiceError::UserIsAdmin, StatusCode::BAD_REQUEST, 10008, "用户是管理员"),
            (ServiceError::CannotOperateSelf, StatusCode::BAD_REQUEST, 10009, "不能操作自己的账号"),
            (
                ServiceError::UserIsAutoLocked(15),
                StatusCode::BAD_REQUEST,
                10010,
                "登录失败次数过多，账号已被锁定，请 15 分钟后再试",
            ),
            (ServiceError::TokenCreationFailed, StatusCode::INTERNAL_SERVER_ERROR, 10103, "令牌创建失败"),
            (ServiceError::EmailConflict, StatusCode::CONFLICT, 10202, "邮箱已存在"),
            (ServiceError::CreateAvatarFolderFailed, StatusCode::INTERNAL_SERVER_ERROR, 20002, "创建头像文件夹失败"),
            (ServiceError::CreateAvatarFileFailed, StatusCode::INTERNAL_SERVER_ERROR, 20003, "创建头像文件失败"),
        ];

        for (err, status, code, message) in cases {
            let (actual_status, body) = response_parts(err).await;
            assert_eq!(actual_status, status);
            assert_eq!(body["code"], code);
            assert_eq!(body["message"], message);
            assert!(body["data"].is_null());
        }
    }

    #[tokio::test]
    async fn sqlx_error_maps_to_database_query_failed_payload() {
        let (status, body) = response_parts(sqlx::Error::RowNotFound).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["code"], 20001);
        assert_eq!(body["message"], "数据库查询失败");
        assert!(body["data"].is_null());
    }
}
