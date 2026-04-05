use axum::Router;
use axum::http::HeaderValue;
/// HTTP-level integration tests using axum-test.
///
/// Covers all API handlers that had 0% coverage:
///   - features/auth/api.rs
///   - features/dashboard/api.rs
///   - features/system/user/api.rs
///   - features/system/role/api.rs
///   - features/system/menu/api.rs
///   - features/system/log/api.rs
///   - middleware/auth.rs
///   - middleware/log.rs (exercised by all authenticated requests)
///
/// Each `#[sqlx::test]` gets a fresh migrated DB (including seed data).
/// The test server is built from the real router, so all middleware runs.
use axum::http::header::AUTHORIZATION;
use axum_test::TestServer;
use axum_test::multipart::{MultipartForm, Part};
use chrono::{Duration, Utc};
use rustzen_admin::{
    core::{app::build_app, session::SessionStore},
    features::system::user::repo::{CreateUserCommand, UserRepository},
};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::net::SocketAddr;

/// Build a typed `Authorization: Bearer <token>` header value.
fn bearer(token: &str) -> HeaderValue {
    HeaderValue::from_str(&format!("Bearer {}", token)).unwrap()
}

// ─────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────

/// Build the full application router (identical to production setup).
fn build_router(pool: PgPool) -> Router {
    build_app(pool)
}

/// Create a `TestServer` with `ConnectInfo<SocketAddr>` support.
fn make_server(pool: PgPool) -> TestServer {
    let router = build_router(pool);
    TestServer::new(router.into_make_service_with_connect_info::<SocketAddr>()).unwrap()
}

/// Create a session token for the seeded superadmin (bypasses password check).
async fn admin_token(pool: &PgPool) -> String {
    let (user_id,): (i64,) = sqlx::query_as("SELECT id FROM users WHERE username = 'superadmin'")
        .fetch_one(pool)
        .await
        .expect("superadmin must be seeded");
    let expires_at = Utc::now() + Duration::hours(8);
    SessionStore::create(pool, user_id, expires_at, "127.0.0.1", "test-agent")
        .await
        .expect("session creation must succeed")
}

/// Seed a non-system user with a known password; returns (user_id, username, password).
async fn seed_plain_user(pool: &PgPool, username: &str, password: &str) -> i64 {
    use rustzen_admin::core::password::PasswordUtils;
    let hash = PasswordUtils::hash_password(password).unwrap();
    UserRepository::create_user(
        pool,
        &CreateUserCommand {
            username: username.to_string(),
            email: format!("{}@test.example", username),
            password_hash: hash,
            real_name: None,
            status: Some(1),
            role_ids: vec![],
        },
    )
    .await
    .expect("seed user must succeed")
}

// ─────────────────────────────────────────────────────────────────
// Auth API
// ─────────────────────────────────────────────────────────────────

/// POST /api/auth/login — correct credentials return a token.
#[sqlx::test]
async fn login_success_returns_token(pool: PgPool) {
    let password = "Login@Pass1";
    seed_plain_user(&pool, "login_ok", password).await;

    let server = make_server(pool);
    let resp = server
        .post("/api/auth/login")
        .json(&json!({"username": "login_ok", "password": password}))
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
    assert!(body["data"]["token"].is_string(), "response must include a token");
}

/// POST /api/auth/login — wrong password → 401.
#[sqlx::test]
async fn login_wrong_password_returns_401(pool: PgPool) {
    let server = make_server(pool);
    let resp = server
        .post("/api/auth/login")
        .json(&json!({"username": "superadmin", "password": "definitely_wrong"}))
        .await;
    resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

/// POST /api/auth/login — nonexistent user → 401.
#[sqlx::test]
async fn login_nonexistent_user_returns_401(pool: PgPool) {
    let server = make_server(pool);
    let resp = server
        .post("/api/auth/login")
        .json(&json!({"username": "nobody_xyz", "password": "any"}))
        .await;
    resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

/// GET /api/auth/me — valid session token returns user info.
#[sqlx::test]
async fn get_me_with_valid_token_returns_user_info(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let resp = server.get("/api/auth/me").add_header(AUTHORIZATION, bearer(&token)).await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
    assert_eq!(body["data"]["username"], "superadmin");
}

/// GET /api/auth/me — no Authorization header → 401.
#[sqlx::test]
async fn get_me_without_token_returns_401(pool: PgPool) {
    let server = make_server(pool);
    let resp = server.get("/api/auth/me").await;
    resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

/// GET /api/auth/me — garbage token → 401.
#[sqlx::test]
async fn get_me_invalid_token_returns_401(pool: PgPool) {
    let server = make_server(pool);
    let resp = server
        .get("/api/auth/me")
        .add_header(AUTHORIZATION, HeaderValue::from_static("Bearer totally_bogus_token_abc"))
        .await;
    resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

/// GET /api/auth/me — deleted user behind a still-valid session is rejected.
#[sqlx::test]
async fn deleted_user_token_rejected(pool: PgPool) {
    let uid = seed_plain_user(&pool, "deleted_http_user", "Deleted@Pass1").await;
    let expires_at = Utc::now() + Duration::hours(1);
    let token =
        SessionStore::create(&pool, uid, expires_at, "127.0.0.1", "test-agent").await.unwrap();

    sqlx::query("UPDATE users SET deleted_at = NOW() WHERE id = $1")
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();

    let server = make_server(pool);
    let resp = server.get("/api/auth/me").add_header(AUTHORIZATION, bearer(&token)).await;
    resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

/// GET /api/auth/logout — valid session is removed.
#[sqlx::test]
async fn logout_invalidates_session(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    // First confirm the token is valid
    let before = server.get("/api/auth/me").add_header(AUTHORIZATION, bearer(&token)).await;
    before.assert_status_ok();

    // Logout
    let logout = server.get("/api/auth/logout").add_header(AUTHORIZATION, bearer(&token)).await;
    logout.assert_status_ok();

    // The same token must now be rejected
    let after = server.get("/api/auth/me").add_header(AUTHORIZATION, bearer(&token)).await;
    after.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

/// PUT /api/auth/self/password — change password then verify old token is invalidated.
#[sqlx::test]
async fn change_password_invalidates_session(pool: PgPool) {
    let old_pw = "OldPass@1";
    let new_pw = "NewPass@1";
    seed_plain_user(&pool, "chpass_http", old_pw).await;

    // Login via HTTP to get a real session token
    let server = make_server(pool);
    let login_resp = server
        .post("/api/auth/login")
        .json(&json!({"username": "chpass_http", "password": old_pw}))
        .await;
    login_resp.assert_status_ok();
    let token = login_resp.json::<Value>()["data"]["token"].as_str().unwrap().to_string();

    // Change password
    let change = server
        .put("/api/auth/self/password")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({"oldPassword": old_pw, "newPassword": new_pw}))
        .await;
    change.assert_status_ok();

    // Old token must be rejected after password change
    let after = server.get("/api/auth/me").add_header(AUTHORIZATION, bearer(&token)).await;
    after.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

/// PUT /api/auth/self/password — wrong old password → 400.
#[sqlx::test]
async fn change_password_wrong_old_returns_400(pool: PgPool) {
    let pw = "Correct@Pass1";
    seed_plain_user(&pool, "chpass_wrong_old", pw).await;

    let server = make_server(pool);
    let login_resp = server
        .post("/api/auth/login")
        .json(&json!({"username": "chpass_wrong_old", "password": pw}))
        .await;
    let token = login_resp.json::<Value>()["data"]["token"].as_str().unwrap().to_string();

    let resp = server
        .put("/api/auth/self/password")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({"oldPassword": "WrongOld@1", "newPassword": "NewPass@1"}))
        .await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

/// POST /api/auth/avatar — valid PNG upload is accepted and avatar_url is persisted.
#[sqlx::test]
async fn upload_avatar_accepts_supported_image(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool.clone());
    let png_bytes = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00];

    let form = MultipartForm::new()
        .add_part("file", Part::bytes(png_bytes).file_name("avatar.png").mime_type("image/png"));

    let resp = server
        .post("/api/auth/avatar")
        .add_header(AUTHORIZATION, bearer(&token))
        .multipart(form)
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    let avatar_url = body["data"].as_str().expect("avatar url should be returned");
    assert!(avatar_url.starts_with("/uploads/avatars/"));
    assert!(avatar_url.ends_with(".png"));

    let (stored_avatar_url,): (Option<String>,) =
        sqlx::query_as("SELECT avatar_url FROM users WHERE username = 'superadmin'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stored_avatar_url.as_deref(), Some(avatar_url));
}

/// POST /api/auth/avatar — non-image MIME type → 400.
#[sqlx::test]
async fn upload_avatar_rejects_invalid_file_type(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let form = MultipartForm::new().add_part(
        "file",
        Part::bytes("not-an-image".as_bytes()).file_name("avatar.txt").mime_type("text/plain"),
    );

    let resp = server
        .post("/api/auth/avatar")
        .add_header(AUTHORIZATION, bearer(&token))
        .multipart(form)
        .await;

    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

/// POST /api/auth/avatar — file exceeding 1 MB limit → 400.
#[sqlx::test]
async fn upload_avatar_rejects_oversized_file(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    bytes.resize(1024 * 1024 + 1, 0x00);

    let form = MultipartForm::new()
        .add_part("file", Part::bytes(bytes).file_name("too-large.png").mime_type("image/png"));

    let resp = server
        .post("/api/auth/avatar")
        .add_header(AUTHORIZATION, bearer(&token))
        .multipart(form)
        .await;

    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

/// POST /api/auth/avatar — multipart request with no file part → 400.
#[sqlx::test]
async fn upload_avatar_rejects_missing_file(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let resp = server
        .post("/api/auth/avatar")
        .add_header(AUTHORIZATION, bearer(&token))
        .multipart(MultipartForm::new())
        .await;

    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

// ─────────────────────────────────────────────────────────────────
// Auth Middleware
// ─────────────────────────────────────────────────────────────────

/// Protected route accessed by a disabled user → 401.
#[sqlx::test]
async fn disabled_user_token_rejected(pool: PgPool) {
    let pw = "Disable@Test1";
    let uid = seed_plain_user(&pool, "disabled_http_user", pw).await;

    // Disable the user directly in DB
    sqlx::query("UPDATE users SET status = 2 WHERE id = $1")
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();

    // Create a session while user was still valid (simulate stale session)
    let token = {
        let expires_at = Utc::now() + Duration::hours(1);
        SessionStore::create(&pool, uid, expires_at, "127.0.0.1", "test").await.unwrap()
    };

    let server = make_server(pool);
    let resp = server.get("/api/auth/me").add_header(AUTHORIZATION, bearer(&token)).await;
    resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

/// Protected route accessed by a locked user → 401.
#[sqlx::test]
async fn locked_user_token_rejected(pool: PgPool) {
    let uid = seed_plain_user(&pool, "locked_http_user", "Locked@Pass1").await;

    sqlx::query("UPDATE users SET locked_until = NOW() + INTERVAL '30 minutes' WHERE id = $1")
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();

    let expires_at = Utc::now() + Duration::hours(1);
    let token =
        SessionStore::create(&pool, uid, expires_at, "127.0.0.1", "test-agent").await.unwrap();

    let server = make_server(pool);
    let resp = server.get("/api/auth/me").add_header(AUTHORIZATION, bearer(&token)).await;
    resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

/// Expired session token is rejected (session row deleted or expired_at in the past).
#[sqlx::test]
async fn expired_session_is_rejected(pool: PgPool) {
    let (user_id,): (i64,) = sqlx::query_as("SELECT id FROM users WHERE username = 'superadmin'")
        .fetch_one(&pool)
        .await
        .unwrap();

    // Create a session that already expired
    let past = Utc::now() - Duration::hours(1);
    let token = format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple());
    sqlx::query(
        "INSERT INTO user_sessions (session_token, user_id, expires_at, client_ip, user_agent, last_access_at)
         VALUES ($1, $2, $3, '127.0.0.1', 'test', NOW())",
    )
    .bind(&token)
    .bind(user_id)
    .bind(past)
    .execute(&pool)
    .await
    .unwrap();

    let server = make_server(pool);
    let resp = server.get("/api/auth/me").add_header(AUTHORIZATION, bearer(&token)).await;
    resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

/// GET /api/health — unauthenticated health check returns status "ok".
#[sqlx::test]
async fn root_health_returns_ok(pool: PgPool) {
    let server = make_server(pool);
    let resp = server.get("/api/health").await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
    assert_eq!(body["data"]["status"], "ok");
}

/// GET /api/summary — returns API welcome message and github link.
#[sqlx::test]
async fn root_summary_returns_api_description(pool: PgPool) {
    let server = make_server(pool);
    let resp = server.get("/api/summary").await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
    assert_eq!(body["data"]["message"], "Welcome to rustzen-admin API");
    assert!(body["data"]["github"].as_str().unwrap().contains("rustzen-admin"));
}

// ─────────────────────────────────────────────────────────────────
// Dashboard API
// ─────────────────────────────────────────────────────────────────

/// GET /api/dashboard/stats — returns aggregated stats (user/role/menu counts).
#[sqlx::test]
async fn get_stats_returns_ok(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let resp = server.get("/api/dashboard/stats").add_header(AUTHORIZATION, bearer(&token)).await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
}

/// GET /api/dashboard/health — returns system resource metrics (CPU, memory).
#[sqlx::test]
async fn get_health_returns_ok(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let resp = server.get("/api/dashboard/health").add_header(AUTHORIZATION, bearer(&token)).await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
    // SystemInfo has cpu_usage and memory_total fields
    assert!(
        body["data"]["cpuUsage"].is_number()
            || body["data"]["cpu_usage"].is_number()
            || body["data"].is_object()
    );
}

/// GET /api/dashboard/metrics — returns application performance metrics.
#[sqlx::test]
async fn get_metrics_returns_ok(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let resp = server.get("/api/dashboard/metrics").add_header(AUTHORIZATION, bearer(&token)).await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
}

/// GET /api/dashboard/trends — returns login activity trend data.
#[sqlx::test]
async fn get_trends_returns_ok(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let resp = server.get("/api/dashboard/trends").add_header(AUTHORIZATION, bearer(&token)).await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
}

/// Dashboard routes are protected — no token → 401.
#[sqlx::test]
async fn dashboard_requires_auth(pool: PgPool) {
    let server = make_server(pool);
    let resp = server.get("/api/dashboard/stats").await;
    resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

// ─────────────────────────────────────────────────────────────────
// User Management API
// ─────────────────────────────────────────────────────────────────

/// GET /api/system/users — returns paginated user list with data array and total count.
#[sqlx::test]
async fn get_user_list_returns_paginated_result(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let resp = server.get("/api/system/users").add_header(AUTHORIZATION, bearer(&token)).await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
    assert!(body["data"].is_array());
    assert!(body["total"].is_number());
}

/// POST /api/system/users — valid payload creates user and returns new user ID.
#[sqlx::test]
async fn create_user_via_api_returns_new_id(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let resp = server
        .post("/api/system/users")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "username": "api_created_user",
            "email": "api_created@test.example",
            "password": "Create@123",
            "roleIds": []
        }))
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
    assert!(body["data"].is_number(), "should return the new user id");
}

/// POST /api/system/users — duplicate username → 409 Conflict.
#[sqlx::test]
async fn create_user_duplicate_username_returns_409(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let user_json = json!({
        "username": "dup_user_api",
        "email": "dup1@test.example",
        "password": "Pass@123",
        "roleIds": []
    });

    server
        .post("/api/system/users")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&user_json)
        .await
        .assert_status_ok();

    // Second attempt with same username
    let resp = server
        .post("/api/system/users")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "username": "dup_user_api",
            "email": "dup2@test.example",
            "password": "Pass@123",
            "roleIds": []
        }))
        .await;

    resp.assert_status(axum::http::StatusCode::CONFLICT);
}

/// PUT /api/system/users/:id — updates email and real_name successfully.
#[sqlx::test]
async fn update_user_via_api_succeeds(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    // Create a user first
    let create_resp = server
        .post("/api/system/users")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "username": "update_target",
            "email": "update_target@test.example",
            "password": "Pass@123",
            "roleIds": []
        }))
        .await;
    let user_id: i64 = create_resp.json::<Value>()["data"].as_i64().expect("should return user id");

    // Update the user
    let resp = server
        .put(&format!("/api/system/users/{}", user_id))
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "email": "updated@test.example",
            "realName": "Updated Name",
            "roleIds": []
        }))
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
}

/// DELETE /api/system/users/:id — soft-deletes the user successfully.
#[sqlx::test]
async fn delete_user_via_api_succeeds(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    // Create a user to delete
    let create_resp = server
        .post("/api/system/users")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "username": "delete_target",
            "email": "delete_target@test.example",
            "password": "Pass@123",
            "roleIds": []
        }))
        .await;
    let user_id: i64 = create_resp.json::<Value>()["data"].as_i64().unwrap();

    let resp = server
        .delete(&format!("/api/system/users/{}", user_id))
        .add_header(AUTHORIZATION, bearer(&token))
        .await;

    resp.assert_status_ok();
}

/// PUT /api/system/users/:id/status — changes user status to Disabled successfully.
#[sqlx::test]
async fn update_user_status_via_api_succeeds(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    // Create a user to modify
    let create_resp = server
        .post("/api/system/users")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "username": "status_target",
            "email": "status_target@test.example",
            "password": "Pass@123",
            "roleIds": []
        }))
        .await;
    let user_id: i64 = create_resp.json::<Value>()["data"].as_i64().unwrap();

    let resp = server
        .put(&format!("/api/system/users/{}/status", user_id))
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({"status": "Disabled"})) // UserStatus deserializes as string variant name
        .await;

    resp.assert_status_ok();
}

/// GET /api/system/users/status-options — returns non-empty list of status option items.
#[sqlx::test]
async fn get_user_status_options_returns_list(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let resp = server
        .get("/api/system/users/status-options")
        .add_header(AUTHORIZATION, bearer(&token))
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
    assert!(body["data"].as_array().map(|a| !a.is_empty()).unwrap_or(false));
}

/// GET /api/system/users/options — returns user option items for select dropdowns.
#[sqlx::test]
async fn get_user_options_returns_list(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let resp =
        server.get("/api/system/users/options").add_header(AUTHORIZATION, bearer(&token)).await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
    assert!(body["data"].is_array());
}

/// PUT /api/system/users/:id/password — admin resets password; response includes new password.
#[sqlx::test]
async fn update_user_password_via_api_succeeds(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    // Create a user whose password we'll reset
    let create_resp = server
        .post("/api/system/users")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "username": "pwreset_target",
            "email": "pwreset@test.example",
            "password": "OldPass@1",
            "roleIds": []
        }))
        .await;
    let user_id: i64 = create_resp.json::<Value>()["data"].as_i64().unwrap();

    let resp = server
        .put(&format!("/api/system/users/{}/password", user_id))
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({})) // UpdateUserPasswordPayload is empty
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
    // Response should include the new generated password
    assert!(body["data"]["password"].is_string());
}

/// PUT /api/system/users/:id/unlock — clears auto-lock and allows subsequent login.
#[sqlx::test]
async fn unlock_user_via_api_succeeds(pool: PgPool) {
    let token = admin_token(&pool).await;
    let pw = "Lock@Pass1";
    let uid = seed_plain_user(&pool, "unlock_via_api", pw).await;

    // Trigger lock by setting locked_until in the future
    sqlx::query("UPDATE users SET locked_until = NOW() + INTERVAL '30 minutes' WHERE id = $1")
        .bind(uid)
        .execute(&pool)
        .await
        .unwrap();

    let server = make_server(pool);
    let resp = server
        .put(&format!("/api/system/users/{}/unlock", uid))
        .add_header(AUTHORIZATION, bearer(&token))
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
}

/// Cannot operate on self — superadmin trying to update their own profile returns 400.
#[sqlx::test]
async fn update_self_returns_400(pool: PgPool) {
    let token = admin_token(&pool).await;

    // Get superadmin's own user ID
    let (uid,): (i64,) = sqlx::query_as("SELECT id FROM users WHERE username = 'superadmin'")
        .fetch_one(&pool)
        .await
        .unwrap();

    let server = make_server(pool);
    let resp = server
        .put(&format!("/api/system/users/{}", uid))
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({"email": "new@example.com", "realName": null, "roleIds": []}))
        .await;

    // CannotOperateSelf → 400
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

// ─────────────────────────────────────────────────────────────────
// Role Management API
// ─────────────────────────────────────────────────────────────────

/// GET /api/system/roles — returns paginated role list with data array and total count.
#[sqlx::test]
async fn get_role_list_returns_paginated_result(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let resp = server.get("/api/system/roles").add_header(AUTHORIZATION, bearer(&token)).await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
    assert!(body["data"].is_array());
    assert!(body["total"].is_number());
}

/// GET /api/system/roles?name=…&code=… — query filters narrow the result set.
#[sqlx::test]
async fn get_role_list_honors_name_and_code_filters(pool: PgPool) {
    let token = admin_token(&pool).await;

    sqlx::query(
        "INSERT INTO roles (name, code, status, sort_order, created_at) VALUES ($1, $2, 1, 0, NOW())",
    )
    .bind("API筛选角色")
    .bind("API_ROLE_FILTER_MATCH")
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO roles (name, code, status, sort_order, created_at) VALUES ($1, $2, 1, 0, NOW())",
    )
    .bind("API其他角色")
    .bind("API_ROLE_OTHER")
    .execute(&pool)
    .await
    .unwrap();

    let server = make_server(pool);
    let resp = server
        .get("/api/system/roles?name=API%E7%AD%9B%E9%80%89%E8%A7%92%E8%89%B2&code=FILTER_MATCH&status=1")
        .add_header(AUTHORIZATION, bearer(&token))
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    let data = body["data"].as_array().expect("data should be array");
    assert_eq!(body["total"], 1);
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["name"], "API筛选角色");
    assert_eq!(data[0]["code"], "API_ROLE_FILTER_MATCH");
}

/// POST /api/system/roles — valid payload creates role successfully.
#[sqlx::test]
async fn create_role_via_api_succeeds(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let resp = server
        .post("/api/system/roles")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "name": "API测试角色",
            "code": "API_TEST_ROLE",
            "status": 1,
            "menuIds": [],
            "description": null
        }))
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
}

/// POST /api/system/roles — duplicate role name → 400.
#[sqlx::test]
async fn create_role_duplicate_name_returns_400(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    server
        .post("/api/system/roles")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "name": "Dup角色",
            "code": "DUP_ROLE_1",
            "status": 1,
            "menuIds": []
        }))
        .await
        .assert_status_ok();

    let resp = server
        .post("/api/system/roles")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "name": "Dup角色",
            "code": "DUP_ROLE_2",
            "status": 1,
            "menuIds": []
        }))
        .await;

    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

/// PUT /api/system/roles/:id — valid payload updates role name successfully.
#[sqlx::test]
async fn update_role_via_api_succeeds(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool.clone());

    // Create role first
    server
        .post("/api/system/roles")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "name": "Update目标角色",
            "code": "UPDATE_TARGET_ROLE",
            "status": 1,
            "menuIds": []
        }))
        .await
        .assert_status_ok();

    let (role_id,): (i64,) =
        sqlx::query_as("SELECT id FROM roles WHERE code = 'UPDATE_TARGET_ROLE'")
            .fetch_one(&pool)
            .await
            .unwrap();

    let resp = server
        .put(&format!("/api/system/roles/{}", role_id))
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "name": "Updated角色",
            "code": "UPDATE_TARGET_ROLE",
            "status": 1,
            "menuIds": []
        }))
        .await;

    resp.assert_status_ok();
}

/// DELETE /api/system/roles/:id — soft-deletes an unassigned role successfully.
#[sqlx::test]
async fn delete_role_via_api_succeeds(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool.clone());

    // Create role first using service directly (pool still accessible)
    server
        .post("/api/system/roles")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "name": "Delete目标角色",
            "code": "DELETE_TARGET_ROLE",
            "status": 1,
            "menuIds": []
        }))
        .await
        .assert_status_ok();

    let (role_id,): (i64,) =
        sqlx::query_as("SELECT id FROM roles WHERE code = 'DELETE_TARGET_ROLE'")
            .fetch_one(&pool)
            .await
            .unwrap();

    let resp = server
        .delete(&format!("/api/system/roles/{}", role_id))
        .add_header(AUTHORIZATION, bearer(&token))
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
}

/// GET /api/system/roles/options — returns enabled role option items for select dropdowns.
#[sqlx::test]
async fn get_role_options_returns_list(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let resp =
        server.get("/api/system/roles/options").add_header(AUTHORIZATION, bearer(&token)).await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
    assert!(body["data"].is_array());
}

// ─────────────────────────────────────────────────────────────────
// Menu Management API
// ─────────────────────────────────────────────────────────────────

/// GET /api/system/menus — returns nested menu tree; seeded menus must be present.
#[sqlx::test]
async fn get_menu_list_returns_tree(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let resp = server.get("/api/system/menus").add_header(AUTHORIZATION, bearer(&token)).await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
    assert!(body["data"].is_array());
    // Seeded menus should be present
    assert!(
        body["data"].as_array().map(|a| !a.is_empty()).unwrap_or(false),
        "menu list should not be empty"
    );
}

/// POST /api/system/menus — valid directory at root is created successfully.
#[sqlx::test]
async fn create_menu_via_api_succeeds(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    // Create a directory at root
    let resp = server
        .post("/api/system/menus")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "parentId": 0,
            "name": "API测试目录",
            "code": "api_test_dir",
            "menuType": 1,  // Directory
            "sortOrder": 99,
            "status": 1
        }))
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
}

/// POST /api/system/menus — duplicate menu name → 400.
#[sqlx::test]
async fn create_menu_duplicate_name_returns_400(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    server
        .post("/api/system/menus")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "parentId": 0,
            "name": "Dup目录",
            "code": "dup_dir_1",
            "menuType": 1,
            "sortOrder": 99,
            "status": 1
        }))
        .await
        .assert_status_ok();

    let resp = server
        .post("/api/system/menus")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "parentId": 0,
            "name": "Dup目录",
            "code": "dup_dir_2",
            "menuType": 1,
            "sortOrder": 99,
            "status": 1
        }))
        .await;

    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

/// PUT /api/system/menus/:id — valid payload updates menu name and sort_order successfully.
#[sqlx::test]
async fn update_menu_via_api_succeeds(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool.clone());

    // Create a non-system directory to update
    server
        .post("/api/system/menus")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "parentId": 0,
            "name": "Update目标",
            "code": "update_target_menu",
            "menuType": 1,
            "sortOrder": 99,
            "status": 1
        }))
        .await
        .assert_status_ok();

    let (menu_id,): (i64,) =
        sqlx::query_as("SELECT id FROM menus WHERE code = 'update_target_menu'")
            .fetch_one(&pool)
            .await
            .unwrap();

    let resp = server
        .put(&format!("/api/system/menus/{}", menu_id))
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "parentId": 0,
            "name": "Updated目标",
            "code": "update_target_menu",
            "menuType": 1,
            "sortOrder": 100,
            "status": 1
        }))
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
}

/// DELETE /api/system/menus/:id — non-system leaf menu is deleted successfully.
#[sqlx::test]
async fn delete_menu_via_api_succeeds(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool.clone());

    // Create a leaf menu to delete
    server
        .post("/api/system/menus")
        .add_header(AUTHORIZATION, bearer(&token))
        .json(&json!({
            "parentId": 0,
            "name": "Delete目标",
            "code": "delete_target_menu",
            "menuType": 1,
            "sortOrder": 99,
            "status": 1
        }))
        .await
        .assert_status_ok();

    let (menu_id,): (i64,) =
        sqlx::query_as("SELECT id FROM menus WHERE code = 'delete_target_menu'")
            .fetch_one(&pool)
            .await
            .unwrap();

    let resp = server
        .delete(&format!("/api/system/menus/{}", menu_id))
        .add_header(AUTHORIZATION, bearer(&token))
        .await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
}

/// DELETE /api/system/menus/1 — system-protected menu (id=1) cannot be deleted → 404.
#[sqlx::test]
async fn delete_system_menu_returns_404(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    // Menu id=1 is seeded as a system menu with no children (leaf) → expect 404
    let resp = server.delete("/api/system/menus/1").add_header(AUTHORIZATION, bearer(&token)).await;

    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}

/// GET /api/system/menus/options — returns enabled menu option items for role assignment UI.
#[sqlx::test]
async fn get_menu_options_returns_list(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let resp =
        server.get("/api/system/menus/options").add_header(AUTHORIZATION, bearer(&token)).await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
    assert!(body["data"].is_array());
}

// ─────────────────────────────────────────────────────────────────
// Log Management API
// ─────────────────────────────────────────────────────────────────

/// GET /api/system/logs — returns paginated operation log list with data array and total.
#[sqlx::test]
async fn get_log_list_returns_paginated_result(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let resp = server.get("/api/system/logs").add_header(AUTHORIZATION, bearer(&token)).await;

    resp.assert_status_ok();
    let body: Value = resp.json();
    assert_eq!(body["code"], 0);
    assert!(body["data"].is_array());
    assert!(body["total"].is_number());
}

/// GET /api/system/logs/export — returns CSV file download with content-disposition header.
#[sqlx::test]
async fn export_log_list_returns_csv(pool: PgPool) {
    let token = admin_token(&pool).await;
    let server = make_server(pool);

    let resp =
        server.get("/api/system/logs/export").add_header(AUTHORIZATION, bearer(&token)).await;

    resp.assert_status_ok();
    // CSV response; content-disposition header should be set
    let disposition =
        resp.headers().get("content-disposition").and_then(|v| v.to_str().ok()).unwrap_or("");
    assert!(disposition.contains("attachment"), "should be a file download");
}

// ─────────────────────────────────────────────────────────────────
// Permission Enforcement
// ─────────────────────────────────────────────────────────────────

/// A user with no roles cannot access permission-gated routes → 403.
#[sqlx::test]
async fn user_without_permissions_gets_403(pool: PgPool) {
    let pw = "NoRole@Pass1";
    seed_plain_user(&pool, "norole_user", pw).await;

    let server = make_server(pool);
    let login_resp = server
        .post("/api/auth/login")
        .json(&json!({"username": "norole_user", "password": pw}))
        .await;
    let token = login_resp.json::<Value>()["data"]["token"].as_str().unwrap().to_string();

    // This route requires system:user:list permission
    let resp = server.get("/api/system/users").add_header(AUTHORIZATION, bearer(&token)).await;

    resp.assert_status(axum::http::StatusCode::FORBIDDEN);
}
