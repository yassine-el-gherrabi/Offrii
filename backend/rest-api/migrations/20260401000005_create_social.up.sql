-- ============================================================================
-- 005: Friendships
-- ============================================================================

CREATE TABLE IF NOT EXISTS friend_requests (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    from_user_id UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    to_user_id   UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status       VARCHAR(20) NOT NULL DEFAULT 'pending'
                 CHECK (status IN ('pending', 'accepted', 'declined', 'cancelled')),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (from_user_id, to_user_id)
);

CREATE INDEX IF NOT EXISTS idx_friend_requests_to_user
    ON friend_requests(to_user_id) WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS idx_friend_requests_from_user
    ON friend_requests(from_user_id) WHERE status = 'pending';

CREATE TABLE IF NOT EXISTS friendships (
    user_a_id  UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    user_b_id  UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_a_id, user_b_id),
    CHECK (user_a_id < user_b_id)
);

CREATE INDEX IF NOT EXISTS idx_friendships_a ON friendships(user_a_id);
CREATE INDEX IF NOT EXISTS idx_friendships_b ON friendships(user_b_id);
