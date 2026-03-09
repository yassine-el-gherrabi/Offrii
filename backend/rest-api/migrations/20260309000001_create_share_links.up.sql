CREATE TABLE share_links (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token      VARCHAR(32) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ           -- NULL = never expires
);

CREATE INDEX idx_share_links_token   ON share_links(token);
CREATE INDEX idx_share_links_user_id ON share_links(user_id);
