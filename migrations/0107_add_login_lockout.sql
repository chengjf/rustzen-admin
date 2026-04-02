-- ============================================================================
-- Migration: Add login lockout columns to users table
-- Tracks consecutive failed login attempts and temporary account lockout.
-- After 5 consecutive failures the account is locked for 30 minutes.
-- The lock is lifted automatically on the next login attempt after expiry.
-- ============================================================================

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS failed_login_attempts SMALLINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS locked_until TIMESTAMPTZ;

COMMENT ON COLUMN users.failed_login_attempts IS 'Consecutive failed login attempts since last successful login or auto-unlock';
COMMENT ON COLUMN users.locked_until IS 'Account locked until this timestamp due to too many failed login attempts; NULL means not locked';

-- ============================================================================
-- Update get_login_credentials to also return lockout state
-- ============================================================================

DROP FUNCTION IF EXISTS get_login_credentials(VARCHAR);

CREATE FUNCTION get_login_credentials(p_username VARCHAR(50))
RETURNS TABLE (
    id                     BIGINT,
    password_hash          VARCHAR(255),
    status                 SMALLINT,
    is_system              BOOLEAN,
    failed_login_attempts  SMALLINT,
    locked_until           TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        u.id,
        u.password_hash,
        u.status,
        u.is_system,
        u.failed_login_attempts,
        u.locked_until
    FROM users u
    WHERE u.username = p_username
      AND u.deleted_at IS NULL;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION get_login_credentials(VARCHAR) IS 'Retrieves user credentials for login authentication, including lockout state';
