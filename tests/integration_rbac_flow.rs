/// End-to-end RBAC lifecycle integration tests.
///
/// These tests cover the full business flow:
///   角色创建 → 用户创建 → 登录锁定 → 管理员解锁 → 登录成功 → 权限验证
///
/// Each `#[sqlx::test]` spins up a fresh migrated database (including seed data
/// from 0105_seed.sql) and tears it down after the test.
use rustzen_admin::{
    common::error::ServiceError,
    features::{
        auth::service::AuthService,
        system::{
            role::{dto::CreateRoleDto, service::RoleService},
            user::{dto::CreateUserDto, service::UserService},
        },
    },
};
use sqlx::PgPool;

// ─────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────

fn role_dto(name: &str, code: &str, menu_ids: Vec<i64>) -> CreateRoleDto {
    CreateRoleDto {
        name: name.to_string(),
        code: code.to_string(),
        status: 1,
        sort_order: None,
        menu_ids,
        description: None,
    }
}

fn user_dto(username: &str, password: &str, role_ids: Vec<i64>) -> CreateUserDto {
    CreateUserDto {
        username: username.to_string(),
        email: format!("{}@integration.test", username),
        password: password.to_string(),
        real_name: None,
        status: None,
        role_ids,
    }
}

async fn simulate_failed_logins(pool: &PgPool, username: &str, n: usize) {
    for i in 0..n {
        let result =
            AuthService::verify_login(pool, username, "intentionally_wrong_password_xyz!!").await;
        assert!(
            matches!(
                result,
                Err(ServiceError::InvalidCredentials) | Err(ServiceError::UserIsAutoLocked(_))
            ),
            "attempt {i}: expected InvalidCredentials or UserIsAutoLocked, got {result:?}"
        );
    }
}

// ─────────────────────────────────────────────
// Test 1: Role creation validation
// ─────────────────────────────────────────────

/// 角色创建闭环：名称/编码唯一性 + 菜单路径完整性
#[sqlx::test]
async fn test_role_creation_with_validation(pool: PgPool) {
    // Resolve seeded menu IDs by code so the test is not coupled to auto-increment values.
    let (dir_id,): (i64,) = sqlx::query_as("SELECT id FROM menus WHERE code = 'system'")
        .fetch_one(&pool)
        .await
        .expect("seed menu 'system' must exist");

    let (menu_id,): (i64,) = sqlx::query_as("SELECT id FROM menus WHERE code = 'system:user:list'")
        .fetch_one(&pool)
        .await
        .expect("seed menu 'system:user:list' must exist");

    let (btn_create,): (i64,) =
        sqlx::query_as("SELECT id FROM menus WHERE code = 'system:user:create'")
            .fetch_one(&pool)
            .await
            .expect("seed menu 'system:user:create' must exist");

    let full_path = vec![dir_id, menu_id, btn_create];

    // 1. First creation should succeed
    RoleService::create_role(&pool, role_dto("验证角色", "VALIDATION_R", full_path.clone()))
        .await
        .expect("first creation should succeed");

    // 2. Duplicate name should fail
    let dup_name =
        RoleService::create_role(&pool, role_dto("验证角色", "OTHER_CODE", vec![])).await;
    assert!(
        matches!(dup_name, Err(ServiceError::InvalidOperation(_))),
        "duplicate name should be rejected"
    );

    // 3. Duplicate code should fail
    let dup_code =
        RoleService::create_role(&pool, role_dto("不同名称", "VALIDATION_R", vec![])).await;
    assert!(
        matches!(dup_code, Err(ServiceError::InvalidOperation(_))),
        "duplicate code should be rejected"
    );

    // 4. Incomplete menu path: child nodes without their parent
    let broken = RoleService::create_role(
        &pool,
        role_dto("路径残缺角色", "BROKEN_R", vec![menu_id, btn_create]),
    )
    .await;
    assert!(
        matches!(broken, Err(ServiceError::InvalidOperation(_))),
        "incomplete menu path should be rejected"
    );

    // 5. Non-existent menu ID
    let bad_id =
        RoleService::create_role(&pool, role_dto("坏菜单角色", "BAD_MENU_R", vec![999_999])).await;
    assert!(matches!(bad_id, Err(ServiceError::InvalidOperation(_))));

    // 6. Full valid path succeeds again
    RoleService::create_role(&pool, role_dto("完整路径角色", "FULL_PATH_R", full_path))
        .await
        .expect("complete menu path should succeed");
}

// ─────────────────────────────────────────────
// Test 2: User creation validation
// ─────────────────────────────────────────────

/// 用户创建闭环：username/email 唯一性 + 角色合法性
#[sqlx::test]
async fn test_user_creation_with_validation(pool: PgPool) {
    // Create a valid role first
    let role_id = {
        use rustzen_admin::features::system::role::repo::RoleRepository;
        RoleRepository::create(&pool, "用户测试角色", "USER_TEST_R", None, 1, 0, &[]).await.unwrap()
    };

    // 1. Create user with valid role → success
    let uid = UserService::create_user(&pool, user_dto("usertest1", "Pass@1234", vec![role_id]))
        .await
        .expect("first user creation should succeed");
    assert!(uid > 0);

    // 2. Duplicate username
    let dup_user =
        UserService::create_user(&pool, user_dto("usertest1", "Pass@1234", vec![])).await;
    assert!(
        matches!(dup_user, Err(ServiceError::UsernameConflict)),
        "duplicate username should be rejected"
    );

    // 3. Duplicate email: create a user with the same email as usertest1
    let dto_dup_email = CreateUserDto {
        username: "usertest2".to_string(),
        email: "usertest1@integration.test".to_string(), // same as usertest1
        password: "Pass@1234".to_string(),
        real_name: None,
        status: None,
        role_ids: vec![],
    };
    let dup_email = UserService::create_user(&pool, dto_dup_email).await;
    assert!(
        matches!(dup_email, Err(ServiceError::EmailConflict)),
        "duplicate email should be rejected"
    );

    // 4. Non-existent role ID
    let bad_role =
        UserService::create_user(&pool, user_dto("usertest3", "Pass@1234", vec![999_999])).await;
    assert!(
        matches!(bad_role, Err(ServiceError::InvalidOperation(_))),
        "non-existent role should be rejected"
    );

    // 5. Duplicate role IDs in the list
    let dup_roles =
        UserService::create_user(&pool, user_dto("usertest4", "Pass@1234", vec![role_id, role_id]))
            .await;
    assert!(
        matches!(dup_roles, Err(ServiceError::InvalidOperation(_))),
        "duplicate role IDs should be rejected"
    );
}

// ─────────────────────────────────────────────
// Test 3: Login lockout flow
// ─────────────────────────────────────────────

/// 登录锁定闭环：错误密码 → 5次后锁定 → 管理员解锁 → 登录成功
#[sqlx::test]
async fn test_login_lockout_flow(pool: PgPool) {
    let username = "lockout_flow_user";
    let password = "Correct@Pass1";

    // Create a test user
    let uid = UserService::create_user(&pool, user_dto(username, password, vec![]))
        .await
        .expect("user creation should succeed");

    // 1. Wrong password → InvalidCredentials
    let wrong = AuthService::verify_login(&pool, username, "wrong!!!").await;
    assert!(matches!(wrong, Err(ServiceError::InvalidCredentials)));

    // 2. 4 more wrong attempts (total 5) → triggers auto-lock
    simulate_failed_logins(&pool, username, 4).await;

    // 3. Verify locked (even with correct password)
    let locked = AuthService::verify_login(&pool, username, password).await;
    assert!(
        matches!(locked, Err(ServiceError::UserIsAutoLocked(_))),
        "account should be auto-locked after 5 failures: {:?}",
        locked
    );

    // 4. Admin unlocks the user
    UserService::unlock_user(&pool, uid).await.expect("unlock should succeed");

    // 5. Correct password now succeeds
    let success = AuthService::verify_login(&pool, username, password).await;
    assert!(success.is_ok(), "login should succeed after unlock: {:?}", success.err());
}

// ─────────────────────────────────────────────
// Test 4: Full RBAC lifecycle
// ─────────────────────────────────────────────

/// 完整 RBAC 闭环：菜单 → 角色 → 用户 → 登录 → 权限验证
#[sqlx::test]
async fn test_full_rbac_lifecycle(pool: PgPool) {
    // ── Step 1: Resolve seeded menu IDs by code (avoids hardcoding auto-increment values) ──
    let (dir_id,): (i64,) = sqlx::query_as("SELECT id FROM menus WHERE code = 'system'")
        .fetch_one(&pool)
        .await
        .expect("seed menu 'system' must exist");

    let (menu_id,): (i64,) = sqlx::query_as("SELECT id FROM menus WHERE code = 'system:user:list'")
        .fetch_one(&pool)
        .await
        .expect("seed menu 'system:user:list' must exist");

    let (btn_create,): (i64,) =
        sqlx::query_as("SELECT id FROM menus WHERE code = 'system:user:create'")
            .fetch_one(&pool)
            .await
            .expect("seed menu 'system:user:create' must exist");

    let (btn_delete,): (i64,) =
        sqlx::query_as("SELECT id FROM menus WHERE code = 'system:user:delete'")
            .fetch_one(&pool)
            .await
            .expect("seed menu 'system:user:delete' must exist");

    // ── Step 2: Create role with complete menu path ────────────────────────
    RoleService::create_role(
        &pool,
        role_dto("RBAC测试角色", "RBAC_TEST", vec![dir_id, menu_id, btn_create, btn_delete]),
    )
    .await
    .expect("role creation should succeed");

    let (role_id,): (i64,) = sqlx::query_as("SELECT id FROM roles WHERE code = 'RBAC_TEST'")
        .fetch_one(&pool)
        .await
        .unwrap();

    // ── Step 3: Create user with that role ────────────────────────────────
    let username = "rbac_test_user";
    let password = "RbacPass@1";

    let uid = UserService::create_user(&pool, user_dto(username, password, vec![role_id]))
        .await
        .expect("user creation should succeed");

    // ── Step 4: Trigger lockout (5 wrong passwords) ───────────────────────
    simulate_failed_logins(&pool, username, 5).await;

    let locked = AuthService::verify_login(&pool, username, password).await;
    assert!(
        matches!(locked, Err(ServiceError::UserIsAutoLocked(_))),
        "expected auto-lock: {:?}",
        locked
    );

    // ── Step 5: Admin unlocks ─────────────────────────────────────────────
    UserService::unlock_user(&pool, uid).await.expect("unlock should succeed");

    // ── Step 6: Correct login succeeds ────────────────────────────────────
    AuthService::verify_login(&pool, username, password)
        .await
        .expect("login should succeed after unlock");

    // ── Step 7: Verify permissions ────────────────────────────────────────
    let info =
        AuthService::get_login_info(&pool, uid).await.expect("get_login_info should succeed");

    // Should contain the button permission codes from the assigned role
    assert!(
        info.permissions.contains(&"system:user:create".to_string()),
        "should have system:user:create, got: {:?}",
        info.permissions
    );
    assert!(
        info.permissions.contains(&"system:user:delete".to_string()),
        "should have system:user:delete, got: {:?}",
        info.permissions
    );

    // ── Step 8: Should NOT contain unassigned permissions ─────────────────
    assert!(
        !info.permissions.contains(&"system:role:create".to_string()),
        "should NOT have system:role:create"
    );
    assert!(
        !info.permissions.contains(&"system:menu:create".to_string()),
        "should NOT have system:menu:create"
    );
}
