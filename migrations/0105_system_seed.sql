
-- ============================================================================
-- Module: Seed initial super admin user data.
-- ============================================================================

INSERT INTO users (username, email, password_hash, real_name, status, is_system)
VALUES (
    'superadmin',
    'superadmin@example.com',
    -- Password: rustzen@123 (argon2id hash)
    '$argon2id$v=19$m=19456,t=2,p=1$i2SSaoqEMMwYzJQPXhVHfg$k1Y5bZ/k5SxEoEroG+UFzCW8aKzK1o/DWKKDU34FiPI',
    '超级管理员',
    1,
    TRUE
)
ON CONFLICT (username) WHERE deleted_at IS NULL DO NOTHING;

-- ============================================================================
-- Module: Seed initial roles.
-- ============================================================================

INSERT INTO roles (name, code, description, status, is_system, sort_order)
VALUES
    ('系统管理员', 'SYSTEM_ADMIN', '系统管理员，具有所有系统功能的完全访问权限', 1, TRUE, 1)
ON CONFLICT (code) WHERE deleted_at IS NULL DO NOTHING;

-- ============================================================================
-- Module: Seed initial system menu structure.
-- Description: Final menu structure with proper hierarchy and permissions.
-- ============================================================================

INSERT INTO menus (id, parent_id, name, code, menu_type, sort_order, status, is_system)
VALUES
    -- Root level
    (1, 0, '系统超级管理员', '*', 1, 1, 2, TRUE),  -- Super admin wildcard
    (2, 0, '系统管理', 'system', 1, 1, 1, TRUE),   -- System directory (no permission code)
    
    -- User Management (Menu with page permission)
    (3, 2, '用户管理', 'system:user:list', 2, 1, 1, TRUE),
    (4, 3, '用户创建', 'system:user:create', 3, 1, 1, TRUE),
    (5, 3, '用户编辑', 'system:user:update', 3, 2, 1, TRUE),
    (6, 3, '用户详情', 'system:user:detail', 3, 3, 1, TRUE),
    (7, 3, '用户删除', 'system:user:delete', 3, 4, 1, TRUE),
    (8, 3, '用户状态', 'system:user:status', 3, 5, 1, TRUE),
    (9, 3, '重置密码', 'system:user:password', 3, 6, 1, TRUE),

    -- Role Management (Menu with page permission)
    (10, 2, '角色管理', 'system:role:list', 2, 2, 1, TRUE),
    (11, 10, '角色创建', 'system:role:create', 3, 1, 1, TRUE),
    (12, 10, '角色编辑', 'system:role:update', 3, 2, 1, TRUE),
    (13, 10, '角色详情', 'system:role:detail', 3, 3, 1, TRUE),
    (14, 10, '角色删除', 'system:role:delete', 3, 4, 1, TRUE),

    -- Menu Management (Menu with page permission)
    (15, 2, '菜单管理', 'system:menu:list', 2, 3, 1, TRUE),
    (16, 15, '菜单创建', 'system:menu:create', 3, 1, 1, TRUE),
    (17, 15, '菜单编辑', 'system:menu:update', 3, 2, 1, TRUE),
    (18, 15, '菜单详情', 'system:menu:detail', 3, 3, 1, TRUE),
    (19, 15, '菜单删除', 'system:menu:delete', 3, 4, 1, TRUE),

    -- Operation Logs (Menu with page permission)
    (25, 2, '操作日志', 'system:log:list', 2, 4, 1, TRUE),
    (26, 25, '日志详情', 'system:log:detail', 3, 1, 1, TRUE),
    (27, 25, '导出日志', 'system:log:export', 3, 2, 1, TRUE)
ON CONFLICT (id) DO UPDATE SET
    parent_id = EXCLUDED.parent_id,
    name = EXCLUDED.name,
    code = EXCLUDED.code,
    menu_type = EXCLUDED.menu_type,
    sort_order = EXCLUDED.sort_order,
    status = EXCLUDED.status,
    is_system = EXCLUDED.is_system;

-- ============================================================================
-- Module: Seed initial role_menus data.
-- Description: Assign super admin wildcard permission to SYSTEM_ADMIN role.
-- ============================================================================

INSERT INTO role_menus (role_id, menu_id, created_at)
SELECT r.id, m.id, NOW()
FROM roles r, menus m
WHERE r.code = 'SYSTEM_ADMIN' AND m.code = '*'
ON CONFLICT (role_id, menu_id) DO NOTHING;

-- ============================================================================
-- Module: sync serial sequence
-- ============================================================================
SELECT setval(pg_get_serial_sequence('menus', 'id'), (SELECT MAX(id) FROM menus));
