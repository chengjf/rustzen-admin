use super::{
    dto::{LoginRequest, LoginResp, UserInfoResp},
    service::AuthService,
};
use crate::{
    common::{
        api::{ApiResponse, AppResult},
        files::save_avatar,
    },
    core::{extractor::CurrentUser, session::SessionStore},
    features::{auth::dto::ChangePasswordPayload, system::log::service::LogService},
};

use axum::{
    Json, Router,
    extract::{ConnectInfo, Multipart, State},
    http::HeaderMap,
    routing::{get, post, put},
};
use sqlx::PgPool;
use std::{net::SocketAddr, time::Instant};

/// Public auth routes (no session required).
pub fn public_auth_routes() -> Router<PgPool> {
    Router::new().route("/login", post(login_handler))
}

/// Protected auth routes (session required).
pub fn protected_auth_routes() -> Router<PgPool> {
    Router::new()
        .route("/me", get(get_login_info_handler))
        .route("/logout", get(logout_handler))
        .route("/avatar", post(update_avatar))
        .route("/self/password", put(change_password_self))
}

#[tracing::instrument(name = "login", skip(pool, addr, headers, request))]
async fn login_handler(
    State(pool): State<PgPool>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> AppResult<LoginResp> {
    request.validate()?;
    let start_time = Instant::now();
    tracing::info!("Login attempt from {}", addr.ip());

    let username = request.username.clone();
    let ip_address = addr.ip().to_string();
    let user_agent = headers.get("user-agent").and_then(|h| h.to_str().ok()).unwrap_or("Unknown");

    match AuthService::login(&pool, request, &ip_address, user_agent).await {
        Ok(response) => {
            if let Err(e) = LogService::log_business_operation(
                &pool,
                response.user_info.id,
                &username,
                "AUTH_LOGIN",
                "User login successful",
                serde_json::json!({}),
                "SUCCESS",
                start_time.elapsed().as_millis() as i32,
                &ip_address,
                &user_agent,
            )
            .await
            {
                tracing::error!("Failed to log login operation: {:?}", e);
            }
            Ok(ApiResponse::success(response))
        }
        Err(err) => {
            if let Err(e) = LogService::log_business_operation(
                &pool,
                0_i64,
                &username,
                "AUTH_LOGIN",
                &err.to_string(),
                serde_json::json!({}),
                "FAIL",
                start_time.elapsed().as_millis() as i32,
                &ip_address,
                &user_agent,
            )
            .await
            {
                tracing::error!("Failed to log failed login operation: {:?}", e);
            }
            tracing::error!("Login failed for user: {}", username);
            Err(err.into())
        }
    }
}

#[tracing::instrument(name = "get_login_info", skip(current_user, pool))]
async fn get_login_info_handler(
    current_user: CurrentUser,
    State(pool): State<PgPool>,
) -> AppResult<UserInfoResp> {
    let user_info = AuthService::get_login_info(&pool, current_user.user_id).await?;
    Ok(ApiResponse::success(user_info))
}

#[tracing::instrument(name = "logout", skip(pool, current_user))]
async fn logout_handler(State(pool): State<PgPool>, current_user: CurrentUser) -> AppResult<()> {
    tracing::info!("Logout for user_id={}", current_user.user_id);

    if let Err(e) = SessionStore::delete_by_user_id(&pool, current_user.user_id).await {
        tracing::error!("Failed to delete session for user_id={}: {:?}", current_user.user_id, e);
    }

    Ok(ApiResponse::success(()))
}

#[tracing::instrument(name = "update_avatar", skip(current_user, pool))]
async fn update_avatar(
    current_user: CurrentUser,
    State(pool): State<PgPool>,
    mut multipart: Multipart,
) -> AppResult<String> {
    let avatar_url = save_avatar(&mut multipart).await?;
    AuthService::update_avatar(&pool, current_user.user_id, &avatar_url).await?;
    Ok(ApiResponse::success(avatar_url))
}

#[tracing::instrument(name = "change_password_self", skip(pool, dto, current_user))]
pub async fn change_password_self(
    State(pool): State<PgPool>,
    current_user: CurrentUser,
    Json(dto): Json<ChangePasswordPayload>,
) -> AppResult<()> {
    dto.validate()?;
    AuthService::change_password(&pool, current_user.user_id, dto).await?;
    Ok(ApiResponse::success(()))
}
