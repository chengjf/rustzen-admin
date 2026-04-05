use super::{
    dto::{CreateMenuDto, MenuItemResp, MenuQuery, UpdateMenuPayload},
    service::{MenuService, MenuTreeOption},
};
use crate::{
    common::{
        api::{ApiResponse, AppResult, OptionsQuery, OptionsWithCodeQuery},
        router_ext::RouterExt,
    },
    core::permission::PermissionsCheck,
};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
};
use sqlx::PgPool;

/// Menu management routes with permission examples
pub fn menu_routes() -> Router<PgPool> {
    Router::new()
        .route_with_permission(
            "/",
            get(get_menu_list),
            PermissionsCheck::Single("system:menu:list"),
        )
        .route_with_permission(
            "/",
            post(create_menu),
            PermissionsCheck::Single("system:menu:create"),
        )
        .route_with_permission(
            "/{id}",
            put(update_menu),
            PermissionsCheck::Single("system:menu:update"),
        )
        .route_with_permission(
            "/{id}",
            delete(delete_menu),
            PermissionsCheck::Single("system:menu:delete"),
        )
        .route_with_permission(
            "/options",
            get(get_menu_options),
            PermissionsCheck::Single("system:menu:options"),
        )
        .route_with_permission(
            "/options-with-code",
            get(get_menu_options_with_code),
            PermissionsCheck::Any(vec![
                "system:menu:create",
                "system:menu:update",
                "system:role:create",
                "system:role:update",
            ]),
        )
}

/// Get menu list with optional filtering
/// Query params: title, status
/// Need show all menu, not pagination
async fn get_menu_list(
    State(pool): State<PgPool>,
    Query(params): Query<MenuQuery>,
) -> AppResult<Vec<MenuItemResp>> {
    params.validate()?;
    tracing::info!("Menu list request: {:?}", params);

    let menu_tree = MenuService::get_menu_list(&pool, params).await?;

    tracing::info!("Menu list retrieved: items={}", menu_tree.len());

    Ok(ApiResponse::success(menu_tree))
}

/// Create new menu
/// Body: name, path, parent_id, icon, sort_order, status
async fn create_menu(
    State(pool): State<PgPool>,
    Json(request): Json<CreateMenuDto>,
) -> AppResult<i64> {
    request.validate()?;
    let menu_id = MenuService::create_menu(&pool, request).await?;
    Ok(ApiResponse::success(menu_id))
}

/// Update menu
/// Body: name, path, parent_id, icon, sort_order, status (all optional)
async fn update_menu(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateMenuPayload>,
) -> AppResult<i64> {
    request.validate()?;
    let menu_id = MenuService::update_menu(&pool, id, request).await?;
    Ok(ApiResponse::success(menu_id))
}

/// Delete menu (handles child cleanup)
async fn delete_menu(State(pool): State<PgPool>, Path(id): Path<i64>) -> AppResult<()> {
    MenuService::delete_menu(&pool, id).await?;
    Ok(ApiResponse::success(()))
}

/// Get menu options for dropdowns
async fn get_menu_options(
    State(pool): State<PgPool>,
    Query(query): Query<OptionsQuery>,
) -> AppResult<Vec<crate::common::api::OptionItem<i64>>> {
    let options = MenuService::get_menu_options(&pool, query).await?;
    Ok(ApiResponse::success(options))
}

/// Get menu options with code for permission tree
async fn get_menu_options_with_code(
    State(pool): State<PgPool>,
    Query(query): Query<OptionsWithCodeQuery>,
) -> AppResult<Vec<MenuTreeOption>> {
    let options = MenuService::get_menu_options_with_code(&pool, query).await?;
    Ok(ApiResponse::success(options))
}
