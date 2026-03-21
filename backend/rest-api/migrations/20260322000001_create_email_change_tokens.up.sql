CREATE TABLE email_change_tokens (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    new_email  VARCHAR(255) NOT NULL,
    token      VARCHAR(64)  NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ  NOT NULL DEFAULT NOW() + INTERVAL '1 hour',
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_email_change_token ON email_change_tokens(token);
CREATE INDEX idx_email_change_user  ON email_change_tokens(user_id);
