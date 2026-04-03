use super::{
    dto::{LoginRequest, LoginResp, UserInfoResp},
    model::{LoginCredentialsEntity, UserStatus},
    repo::AuthRepository,
};
use crate::{
    common::error::ServiceError,
    core::{
        config::CONFIG,
        password::PasswordUtils,
        session::SessionStore,
    },
    features::{auth::dto::ChangePasswordPayload, system::user::repo::UserRepository},
};

use chrono::{Duration, Utc};
use sqlx::PgPool;
use tracing;

pub struct AuthService;

impl AuthService {
    /// Login with username/password, returning an opaque session token.
    pub async fn login(
        pool: &PgPool,
        request: LoginRequest,
        client_ip: &str,
        user_agent: &str,
    ) -> Result<LoginResp, ServiceError> {
        let start = std::time::Instant::now();
        tracing::info!("Login attempt for username: {}", request.username);

        let user = Self::verify_login(pool, &request.username, &request.password).await?;

        // Create a server-side session and get the opaque token.
        let expires_at = Utc::now() + Duration::seconds(CONFIG.session_expiration_secs);
        let token = SessionStore::create(pool, user.id, expires_at, client_ip, user_agent)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create session for user_id={}: {:?}", user.id, e);
                e
            })?;

        // Update last login time in the background.
        let pool_clone = pool.clone();
        let user_id_clone = user.id;
        tokio::spawn(async move {
            let _ = AuthRepository::update_last_login(&pool_clone, user_id_clone).await;
        });

        let user_info = Self::get_login_info(pool, user.id).await?;

        tracing::info!(
            "Login successful for username={}, user_id={}, elapsed={:?}",
            request.username,
            user.id,
            start.elapsed()
        );

        Ok(LoginResp { token, user_info })
    }

    /// Get detailed user info (permissions fetched fresh from DB).
    pub async fn get_login_info(pool: &PgPool, user_id: i64) -> Result<UserInfoResp, ServiceError> {
        let user = AuthRepository::get_user_by_id(pool, user_id)
            .await?
            .ok_or(ServiceError::NotFound("User".to_string()))?;

        let permissions = if user.is_system {
            vec!["*".to_string()]
        } else {
            AuthRepository::get_user_permissions(pool, user_id).await?
        };

        Ok(UserInfoResp {
            id: user.id,
            username: user.username,
            real_name: user.real_name,
            email: user.email,
            avatar_url: user.avatar_url,
            is_system: user.is_system,
            permissions,
        })
    }

    /// Verify credentials and return the user entity on success.
    pub async fn verify_login(
        pool: &PgPool,
        username: &str,
        password: &str,
    ) -> Result<LoginCredentialsEntity, ServiceError> {
        let user = AuthRepository::get_login_credentials(pool, username)
            .await?
            .ok_or(ServiceError::InvalidCredentials)?;

        let status = UserStatus::try_from(user.status)?;
        status.check_status()?;

        if let Some(locked_until) = user.locked_until {
            if locked_until > Utc::now() {
                let remaining_mins = (locked_until - Utc::now()).num_minutes() + 1;
                tracing::warn!(
                    "Account locked for username={}, ~{}m remaining",
                    username,
                    remaining_mins
                );
                return Err(ServiceError::UserIsAutoLocked(remaining_mins));
            }
            AuthRepository::reset_failed_attempts(pool, user.id).await?;
        }

        if !PasswordUtils::verify_password(&password.to_string(), &user.password_hash) {
            let _ = AuthRepository::increment_failed_attempts(pool, user.id).await;
            let new_count = user.failed_login_attempts + 1;
            tracing::warn!(
                "Invalid password ({}/5) for username={}, user_id={}",
                new_count,
                username,
                user.id
            );
            return Err(ServiceError::InvalidCredentials);
        }

        let _ = AuthRepository::reset_failed_attempts(pool, user.id).await;

        tracing::info!("Credentials verified for username={}, user_id={}", username, user.id);
        Ok(user)
    }

    pub async fn update_avatar(
        pool: &PgPool,
        user_id: i64,
        avatar_url: &str,
    ) -> Result<(), ServiceError> {
        AuthRepository::update_avatar(pool, user_id, avatar_url).await
    }

    pub async fn change_password(
        pool: &PgPool,
        user_id: i64,
        dto: ChangePasswordPayload,
    ) -> Result<(), ServiceError> {
        let user = UserRepository::get_by_id(pool, user_id).await?;

        if !PasswordUtils::verify_password(&dto.old_password, &user.password_hash) {
            return Err(ServiceError::InvalidOperation("旧密码错误".to_string()));
        }

        let new_hash = PasswordUtils::hash_password(&dto.new_password)?;
        UserRepository::update_user_password(pool, user_id, &new_hash).await?;

        // Revoke session so the user must re-login with the new password.
        if let Err(e) = SessionStore::delete_by_user_id(pool, user_id).await {
            tracing::error!(
                "Failed to delete session after password change for user_id={}: {:?}",
                user_id,
                e
            );
        }

        Ok(())
    }
}
