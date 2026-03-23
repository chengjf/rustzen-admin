-- ============================================================================
-- Module: Add missing user status and password permissions.
-- These permissions are used in the frontend but were missing from the seed data.
-- ============================================================================

INSERT INTO menus (parent_id, name, code, menu_type, sort_order, status, is_system)
SELECT
    m.id as parent_id,
    'User Status' as name,
    'system:user:status' as code,
    3 as menu_type,
    6 as sort_order,
    1 as status,
    TRUE as is_system
FROM menus m
WHERE m.code = 'system:user:*'
ON CONFLICT (code) DO NOTHING;

INSERT INTO menus (parent_id, name, code, menu_type, sort_order, status, is_system)
SELECT
    m.id as parent_id,
    'User Password' as name,
    'system:user:password' as code,
    3 as menu_type,
    7 as sort_order,
    1 as status,
    TRUE as is_system
FROM menus m
WHERE m.code = 'system:user:*'
ON CONFLICT (code) DO NOTHING;
