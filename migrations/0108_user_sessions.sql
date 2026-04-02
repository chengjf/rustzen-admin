-- ============================================================================
-- Migration: Persistent user sessions
-- Stores active sessions in the database so in-memory permission caches can
-- be restored after a server restart without requiring users to re-login.
-- One row per user (UNIQUE on user_id); upserted on every login.
-- ============================================================================

CREATE TABLE user_sessions (
    id          BIGSERIAL    PRIMARY KEY,
    user_id     BIGINT       NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permissions JSONB        NOT NULL DEFAULT '[]',
    is_system   BOOLEAN      NOT NULL DEFAULT FALSE,
    expires_at  TIMESTAMPTZ  NOT NULL,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_user_sessions_user_id UNIQUE (user_id)
);

CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);

COMMENT ON TABLE user_sessions IS 'Persistent user sessions; used to restore in-memory permission cache after server restart';
COMMENT ON COLUMN user_sessions.permissions IS 'JSON array of permission codes held at login time';
COMMENT ON COLUMN user_sessions.expires_at  IS 'Session expiry; matches JWT expiration issued at login';
