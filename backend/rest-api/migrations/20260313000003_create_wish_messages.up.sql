CREATE TABLE wish_messages (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wish_id     UUID NOT NULL REFERENCES community_wishes(id) ON DELETE CASCADE,
    sender_id   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    body        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_wm_wish_created ON wish_messages(wish_id, created_at);
