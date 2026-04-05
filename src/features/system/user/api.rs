use super::{
    dto::{
        CreateUserDto, ResetPasswordResp, UpdateUserPasswordPayload, UpdateUserPayload,
        UpdateUserStatusPayload, UserItemResp, UserOptionResp, UserOptionsQuery, UserQuery,
    },
    service::UserService,
};
use crate::{
    common::{
        api::{ApiResponse, AppResult},
        error::ServiceError,
        router_ext::RouterExt,
    },
    core::extractor::CurrentUser,
    core::permission::PermissionsCheck,
};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
};
use sqlx::PgPool;
use tracing::instrument;

/// User management routes
pub fn user_routes() -> Router<sqlx::PgPool> {
    Router::new()
        .route_with_permission(
            "/",
            get(get_user_list),
            PermissionsCheck::Single("system:user:list"),
        )
        .route_with_permission(
            "/",
            post(create_user),
            PermissionsCheck::Single("system:user:create"),
        )
        .route_with_permission(
            "/{id}",
            put(update_user),
            PermissionsCheck::Single("system:user:update"),
        )
        .route_with_permission(
            "/{id}",
            delete(delete_user),
            PermissionsCheck::Single("system:user:delete"),
        )
        .route_with_permission(
            "/options",
            get(get_user_options),
            PermissionsCheck::Single("system:user:list"),
        )
        .route_with_permission(
            "/status-options",
            get(get_user_status_options),
            PermissionsCheck::Single("system:user:list"),
        )
        .route_with_permission(
            "/{id}/password",
            put(update_user_password),
            PermissionsCheck::Single("system:user:password"),
        )
        .route_with_permission(
            "/{id}/status",
            put(update_user_status),
            PermissionsCheck::Single("system:user:status"),
        )
        .route_with_permission(
            "/{id}/unlock",
            put(unlock_user),
            PermissionsCheck::Single("system:user:unlock"),
        )
}

/// Get user list
#[instrument(skip(pool, query))]
pub async fn get_user_list(
    State(pool): State<PgPool>,
    Query(query): Query<UserQuery>,
) -> AppResult<Vec<UserItemResp>> {
    query.validate()?;
    tracing::info!("Getting user list");

    let (users, total) = UserService::get_user_list(&pool, query).await?;

    tracing::info!("Successfully retrieved {} users", users.len());
    Ok(ApiResponse::page(users, total))
}

/// Create user
#[instrument(skip(pool, dto))]
pub async fn create_user(
    State(pool): State<PgPool>,
    Json(dto): Json<CreateUserDto>,
) -> AppResult<i64> {
    dto.validate()?;
    tracing::info!("Creating user: {}", dto.username);

    let user_id = UserService::create_user(&pool, dto).await?;

    tracing::info!("Successfully created user");
    Ok(ApiResponse::success(user_id))
}

/// Update user
#[instrument(skip(pool, id, dto, current_user))]
pub async fn update_user(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    current_user: CurrentUser,
    Json(dto): Json<UpdateUserPayload>,
) -> AppResult<i64> {
    dto.validate()?;
    tracing::info!("Updating user ID: {}", id);

    if id == current_user.user_id {
        return Err(ServiceError::CannotOperateSelf.into());
    }

    UserService::ensure_not_system(&pool, id).await?;

    let user_id = UserService::update_user(&pool, id, dto).await?;

    tracing::info!("Successfully updated user");
    Ok(ApiResponse::success(user_id))
}

/// Delete user
#[instrument(skip(pool, id, current_user))]
pub async fn delete_user(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    current_user: CurrentUser,
) -> AppResult<()> {
    tracing::info!("Deleting user ID: {}", id);

    if id == current_user.user_id {
        return Err(ServiceError::CannotOperateSelf.into());
    }

    UserService::ensure_not_system(&pool, id).await?;

    UserService::delete_user(&pool, id).await?;

    tracing::info!("Successfully deleted user ID: {}", id);
    Ok(ApiResponse::success(()))
}

/// Get user status options
#[instrument]
pub async fn get_user_status_options() -> AppResult<Vec<UserOptionResp>> {
    tracing::info!("Getting user status options");

    let result = UserService::get_user_status_options();

    tracing::info!("Successfully retrieved {} status options", result.len());
    Ok(ApiResponse::success(result))
}

/// Get user options
#[instrument(skip(pool, query))]
pub async fn get_user_options(
    State(pool): State<PgPool>,
    Query(query): Query<UserOptionsQuery>,
) -> AppResult<Vec<UserOptionResp>> {
    query.validate()?;
    tracing::info!("Getting user options");

    let result = UserService::get_user_options(&pool, query).await?;

    tracing::info!("Successfully retrieved {} user options", result.len());
    Ok(ApiResponse::success(result))
}

#[instrument(skip(pool, id, dto, current_user))]
pub async fn update_user_password(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    current_user: CurrentUser,
    Json(dto): Json<UpdateUserPasswordPayload>,
) -> AppResult<ResetPasswordResp> {
    tracing::info!("Resetting password for user: {}", id);

    if id == current_user.user_id {
        return Err(ServiceError::CannotOperateSelf.into());
    }

    UserService::ensure_not_system(&pool, id).await?;

    let result = UserService::update_user_password(&pool, id, dto).await?;

    tracing::info!("Successfully reset user password");
    Ok(ApiResponse::success(result))
}

#[instrument(skip(pool, id, current_user))]
pub async fn unlock_user(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    current_user: CurrentUser,
) -> AppResult<()> {
    tracing::info!("Unlocking user ID: {} by {}", id, current_user.username);

    if id == current_user.user_id {
        return Err(ServiceError::CannotOperateSelf.into());
    }

    UserService::unlock_user(&pool, id).await?;

    tracing::info!("User ID {} successfully unlocked", id);
    Ok(ApiResponse::success(()))
}

#[instrument(skip(pool, id, dto, current_user))]
pub async fn update_user_status(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    current_user: CurrentUser,
    Json(dto): Json<UpdateUserStatusPayload>,
) -> AppResult<()> {
    tracing::info!("Updating user status for user: {}", id);

    if id == current_user.user_id {
        return Err(ServiceError::CannotOperateSelf.into());
    }

    UserService::ensure_not_system(&pool, id).await?;

    UserService::update_user_status(&pool, id, dto).await?;

    tracing::info!("Successfully updated user status");
    Ok(ApiResponse::success(()))
}
