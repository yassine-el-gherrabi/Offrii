-- ============================================================================
-- 001: Users, email tokens, connection logs
-- ============================================================================

-- Reusable trigger function for updated_at columns
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE IF NOT EXISTS users (
    id                        UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    email                     VARCHAR(255) UNIQUE NOT NULL,
    username                  VARCHAR(30)  NOT NULL,
    password_hash             VARCHAR(255),
    display_name              VARCHAR(100),
    oauth_provider            VARCHAR(20),
    oauth_provider_id         VARCHAR(255),
    email_verified            BOOLEAN     NOT NULL DEFAULT false,
    token_version             INT         NOT NULL DEFAULT 1,
    is_admin                  BOOLEAN     NOT NULL DEFAULT false,
    username_customized       BOOLEAN     NOT NULL DEFAULT false,
    avatar_url                VARCHAR(2048),
    terms_accepted_at         TIMESTAMPTZ,
    last_active_at            TIMESTAMPTZ,
    inactivity_notice_sent_at TIMESTAMPTZ,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Username format: starts with lowercase letter, 3-30 chars, lowercase alphanumeric + underscore
ALTER TABLE users ADD CONSTRAINT users_username_unique UNIQUE (username);
ALTER TABLE users ADD CONSTRAINT users_username_format CHECK (username ~ '^[a-z][a-z0-9_]{2,29}$');

-- Partial unique index for OAuth provider lookups
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_oauth
    ON users (oauth_provider, oauth_provider_id)
    WHERE oauth_provider IS NOT NULL;

CREATE TRIGGER trg_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- When a user is deleted, reset any wishes they were matched to back to 'open'
CREATE OR REPLACE FUNCTION cleanup_matched_wishes_on_user_delete()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE community_wishes
    SET status = 'open', matched_with = NULL, matched_at = NULL
    WHERE matched_with = OLD.id AND status = 'matched';
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

-- NOTE: trigger created after community_wishes table exists (in 006)

-- ---------------------------------------------------------------------------
-- Connection logs
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS connection_logs (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ip         VARCHAR(45) NOT NULL,
    user_agent VARCHAR(512) NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_connection_logs_user    ON connection_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_connection_logs_created ON connection_logs(created_at);

-- ---------------------------------------------------------------------------
-- Email verification tokens
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS email_verification_tokens (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token      VARCHAR(64) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '24 hours',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ---------------------------------------------------------------------------
-- Email change tokens
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS email_change_tokens (
    id         UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    new_email  VARCHAR(255) NOT NULL,
    token      VARCHAR(64)  NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ  NOT NULL DEFAULT NOW() + INTERVAL '1 hour',
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_email_change_user ON email_change_tokens(user_id);

-- ── Schema documentation ──
COMMENT ON TABLE users IS 'Core identity — supports email/password and OAuth (Google, Apple) authentication';
COMMENT ON COLUMN users.password_hash IS 'Argon2id hash. NULL for OAuth-only accounts (no password set)';
COMMENT ON COLUMN users.token_version IS 'Incremented on password change — instantly invalidates all existing JWTs across devices';
COMMENT ON COLUMN users.username_customized IS 'false = auto-generated from display_name/email, true = explicitly chosen by user';
COMMENT ON COLUMN users.inactivity_notice_sent_at IS 'Prevents sending repeated inactivity warnings to the same user';
COMMENT ON COLUMN users.terms_accepted_at IS 'Legal requirement — tracks when the user accepted the terms of service';

COMMENT ON TABLE connection_logs IS 'Login audit trail — IP and user-agent per authentication event. Retained 12 months (legal requirement)';
COMMENT ON COLUMN connection_logs.ip IS 'IPv4 or IPv6 address (VARCHAR(45) covers both formats)';

COMMENT ON TABLE email_verification_tokens IS 'One-time tokens sent via email to confirm ownership. 24-hour expiry';
COMMENT ON TABLE email_change_tokens IS 'Tokens for email modification requests. 1-hour expiry (shorter than verification for security)';
