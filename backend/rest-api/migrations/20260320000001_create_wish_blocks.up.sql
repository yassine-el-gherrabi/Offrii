CREATE TABLE wish_blocks (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    wish_id    UUID        NOT NULL REFERENCES community_wishes(id) ON DELETE CASCADE,
    user_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (wish_id, user_id)
);
