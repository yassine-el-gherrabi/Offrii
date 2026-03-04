CREATE TABLE push_tokens (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token            VARCHAR(500) NOT NULL,
    platform         VARCHAR(10) NOT NULL
                     CHECK (platform IN ('ios', 'android')),
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, token)
);
