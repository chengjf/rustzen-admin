-- ============================================================================
-- Migration: Add locked_until to user_with_roles view
-- Needed so the API can reflect auto-lockout state in UserItemResp.status
-- without changing the status column itself.
-- ============================================================================

-- PostgreSQL requires DROP + CREATE when adding a column in the middle of a view.
DROP VIEW IF EXISTS user_with_roles;

CREATE VIEW user_with_roles AS
SELECT
    u.id,
    u.username,
    u.email,
    u.real_name,
    u.password_hash,
    u.avatar_url,
    u.status,
    u.locked_until,
    u.is_system,
    u.last_login_at,
    u.created_at,
    u.updated_at,
    COALESCE(
        JSON_AGG(
            JSON_BUILD_OBJECT(
                'label', r.name,
                'value', r.id
            ) ORDER BY r.id
        ) FILTER (WHERE r.id IS NOT NULL),
        '[]'::json
    ) AS roles
FROM users u
LEFT JOIN user_roles ur ON u.id = ur.user_id
LEFT JOIN roles r ON ur.role_id = r.id AND r.deleted_at IS NULL
WHERE u.deleted_at IS NULL
GROUP BY u.id, u.username, u.email, u.real_name, u.password_hash,
         u.avatar_url, u.status, u.locked_until, u.is_system,
         u.last_login_at, u.created_at, u.updated_at;
