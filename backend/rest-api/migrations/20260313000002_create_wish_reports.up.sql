CREATE TABLE wish_reports (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wish_id     UUID NOT NULL REFERENCES community_wishes(id) ON DELETE CASCADE,
    reporter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reason      VARCHAR(50) NOT NULL DEFAULT 'inappropriate'
                CHECK (reason IN ('inappropriate', 'spam', 'scam', 'other')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (wish_id, reporter_id)
);

CREATE INDEX idx_wr_wish ON wish_reports(wish_id);
