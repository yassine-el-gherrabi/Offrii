-- ============================================================================
-- 007: Infrastructure — push tokens, refresh tokens, notifications, share links
-- ============================================================================

-- ---------------------------------------------------------------------------
-- Push tokens
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS push_tokens (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token      VARCHAR(500) NOT NULL,
    platform   VARCHAR(10) NOT NULL
               CHECK (platform IN ('ios', 'android')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, token)
);

-- ---------------------------------------------------------------------------
-- Refresh tokens
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS refresh_tokens (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) UNIQUE NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_active_expires
    ON refresh_tokens(expires_at)
    WHERE revoked_at IS NULL;

-- ---------------------------------------------------------------------------
-- Notifications
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS notifications (
    id        UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id   UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type      VARCHAR(50) NOT NULL,
    title     TEXT        NOT NULL,
    body      TEXT        NOT NULL,
    read      BOOLEAN     NOT NULL DEFAULT FALSE,
    circle_id UUID        REFERENCES circles(id) ON DELETE SET NULL,
    item_id   UUID        REFERENCES items(id) ON DELETE SET NULL,
    wish_id   UUID        REFERENCES community_wishes(id) ON DELETE SET NULL,
    actor_id  UUID        REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_notifications_type CHECK (
        type IN (
            'friend_request', 'friend_accepted',
            'circle_activity', 'circle_added', 'circle_member_joined',
            'item_shared', 'item_claimed', 'item_unclaimed', 'item_received',
            'wish_moderation_approved', 'wish_moderation_flagged',
            'wish_offer', 'wish_offer_withdrawn', 'wish_offer_rejected', 'wish_closed',
            'wish_approved', 'wish_rejected', 'wish_confirmed',
            'wish_message', 'wish_reported'
        )
    )
);

CREATE INDEX IF NOT EXISTS idx_notifications_user_unread
    ON notifications(user_id, read) WHERE read = FALSE;
CREATE INDEX IF NOT EXISTS idx_notifications_user_created
    ON notifications(user_id, created_at DESC);

-- ---------------------------------------------------------------------------
-- Share links
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS share_links (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token       VARCHAR(32) NOT NULL UNIQUE,
    label       VARCHAR(100),
    permissions VARCHAR(20) NOT NULL DEFAULT 'view_and_claim'
                CHECK (permissions IN ('view_only', 'view_and_claim')),
    scope       VARCHAR(20) NOT NULL DEFAULT 'all'
                CHECK (scope IN ('all', 'category', 'selection')),
    scope_data  JSONB,
    is_active   BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ  -- NULL = never expires
);

CREATE INDEX IF NOT EXISTS idx_share_links_user_id ON share_links(user_id);

-- Now add the FK from items.claimed_via_link_id -> share_links
ALTER TABLE items
    ADD CONSTRAINT fk_items_claimed_via_link
    FOREIGN KEY (claimed_via_link_id) REFERENCES share_links(id) ON DELETE SET NULL;

-- ── Schema documentation ──
COMMENT ON TABLE push_tokens IS 'Device tokens for APNs (iOS) push notifications. One user can have multiple devices';

COMMENT ON TABLE refresh_tokens IS 'Long-lived JWT refresh tokens for session management';
COMMENT ON COLUMN refresh_tokens.token_hash IS 'SHA256 hash of the actual token — raw token is never stored for security';
COMMENT ON COLUMN refresh_tokens.revoked_at IS 'Set on logout or password change. NULL = active token';

COMMENT ON TABLE notifications IS 'In-app notification feed with polymorphic context (circle, item, wish, actor). 20 types. Cleaned up after 6 months';
COMMENT ON COLUMN notifications.type IS '20 types covering: friend events, circle activity, item claims, wish moderation, wish messaging';
COMMENT ON COLUMN notifications.read IS 'Read/unread state for badge count. Partial index on unread for fast count queries';
COMMENT ON COLUMN notifications.actor_id IS 'The user who triggered the notification — used to display "X did Y" in the feed';

COMMENT ON TABLE share_links IS 'Public shareable URLs for wishlists — allows viewing and optionally claiming items without authentication';
COMMENT ON COLUMN share_links.permissions IS 'view_only = browse items, view_and_claim = browse + claim items';
COMMENT ON COLUMN share_links.scope IS 'all = entire wishlist, category = specific categories, selection = hand-picked items';
COMMENT ON COLUMN share_links.scope_data IS 'JSONB payload — contains category IDs or item IDs depending on scope';
COMMENT ON COLUMN share_links.expires_at IS 'NULL = link never expires';
