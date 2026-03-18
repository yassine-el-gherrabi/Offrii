-- Share rules for direct circles: defines what a user shares with a friend
-- share_mode: 'none' | 'all' | 'categories' | 'selection'
-- For 'categories' mode, category_ids contains the UUIDs of shared categories
-- For 'selection' mode, items are stored in circle_items (existing behavior)
-- For 'all' mode, all active non-private items are visible dynamically

CREATE TABLE circle_share_rules (
    circle_id    UUID NOT NULL REFERENCES circles(id) ON DELETE CASCADE,
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    share_mode   VARCHAR(20) NOT NULL DEFAULT 'none',
    category_ids UUID[] NOT NULL DEFAULT '{}',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (circle_id, user_id)
);

CREATE INDEX idx_circle_share_rules_user ON circle_share_rules(user_id);
