-- ============================================================================
-- Migration: Add effective_status computed column to user_with_roles view
--
-- effective_status resolves the two-field lock representation into one value:
--   status=1 + locked_until > NOW()  →  4  (auto-locked, treated as Locked)
--   status=1 + no active lock        →  1  (Normal)
--   any other status value           →  as-is
--
-- All filtering and display logic now reads effective_status, so the repo
-- needs no special-case SQL for the Locked state.
-- ============================================================================

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
    CASE
        WHEN u.status != 1              THEN u.status
        WHEN u.locked_until > NOW()     THEN 4
        ELSE 1
    END::SMALLINT AS effective_status,
    u.is_system,
    u.last_login_at,
    u.created_at,
    u.updated_at,
    COALESCE(
        JSON_AGG(
            JSON_BUILD_OBJECT('label', r.name, 'value', r.id)
            ORDER BY r.id
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

COMMENT ON VIEW user_with_roles IS
    'User list with roles. effective_status merges status + locked_until into one queryable value.';
