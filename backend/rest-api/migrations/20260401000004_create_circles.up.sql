-- ============================================================================
-- 004: Circles, members, items, events, share rules, invites
-- ============================================================================

CREATE TABLE IF NOT EXISTS circles (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name       VARCHAR(100),  -- NULL for direct circles
    owner_id   UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_direct  BOOLEAN     NOT NULL DEFAULT false,
    image_url  TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS circle_members (
    circle_id UUID        NOT NULL REFERENCES circles(id) ON DELETE CASCADE,
    user_id   UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role      VARCHAR(20) NOT NULL DEFAULT 'member'
              CHECK (role IN ('owner', 'member')),
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (circle_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_circle_members_user ON circle_members(user_id);

-- Auto-add circle owner as member with role='owner'
CREATE OR REPLACE FUNCTION add_circle_owner_as_member()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO circle_members (circle_id, user_id, role)
    VALUES (NEW.id, NEW.owner_id, 'owner')
    ON CONFLICT (circle_id, user_id) DO UPDATE
        SET role = 'owner';
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_circles_add_owner_member
    AFTER INSERT ON circles
    FOR EACH ROW EXECUTE FUNCTION add_circle_owner_as_member();

-- Enforce max 2 members in direct circles
CREATE OR REPLACE FUNCTION fn_check_direct_circle_member_limit()
RETURNS TRIGGER AS $$
DECLARE v_is_direct BOOLEAN; v_count INTEGER;
BEGIN
    SELECT is_direct INTO v_is_direct FROM circles WHERE id = NEW.circle_id;
    IF v_is_direct THEN
        SELECT COUNT(*) INTO v_count FROM circle_members WHERE circle_id = NEW.circle_id;
        IF v_count >= 2 THEN RAISE EXCEPTION 'direct circles cannot have more than 2 members'; END IF;
    END IF;
    RETURN NEW;
END; $$ LANGUAGE plpgsql;

CREATE TRIGGER trg_check_direct_circle_member_limit
    BEFORE INSERT ON circle_members
    FOR EACH ROW EXECUTE FUNCTION fn_check_direct_circle_member_limit();

-- ---------------------------------------------------------------------------
-- Circle items (shared wishlist items)
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS circle_items (
    circle_id UUID        NOT NULL REFERENCES circles(id) ON DELETE CASCADE,
    item_id   UUID        NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    shared_by UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    shared_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (circle_id, item_id)
);

CREATE INDEX IF NOT EXISTS idx_circle_items_item_id ON circle_items(item_id);

-- ---------------------------------------------------------------------------
-- Circle events (activity feed)
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS circle_events (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    circle_id      UUID        NOT NULL REFERENCES circles(id) ON DELETE CASCADE,
    actor_id       UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    event_type     VARCHAR(30) NOT NULL CHECK (event_type IN (
        'item_shared', 'item_unshared', 'item_claimed',
        'item_unclaimed', 'member_joined', 'member_left',
        'item_received', 'share_rule_set', 'share_rule_removed'
    )),
    target_item_id UUID        REFERENCES items(id) ON DELETE SET NULL,
    target_user_id UUID        REFERENCES users(id) ON DELETE SET NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_circle_events_circle_created
    ON circle_events(circle_id, created_at DESC);

-- ---------------------------------------------------------------------------
-- Circle share rules (for direct circles)
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS circle_share_rules (
    circle_id    UUID        NOT NULL REFERENCES circles(id) ON DELETE CASCADE,
    user_id      UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    share_mode   VARCHAR(20) NOT NULL DEFAULT 'none',
    category_ids UUID[]      NOT NULL DEFAULT '{}',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (circle_id, user_id),
    CONSTRAINT chk_share_mode CHECK (
        share_mode IN ('none', 'all', 'categories', 'selection')
    )
);

CREATE INDEX IF NOT EXISTS idx_circle_share_rules_user ON circle_share_rules(user_id);

-- ---------------------------------------------------------------------------
-- Circle invites
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS circle_invites (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    circle_id  UUID        NOT NULL REFERENCES circles(id) ON DELETE CASCADE,
    token      VARCHAR(32) NOT NULL UNIQUE,
    created_by UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL,
    max_uses   INTEGER     NOT NULL DEFAULT 1,
    use_count  INTEGER     NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_circle_invites_circle_id ON circle_invites(circle_id);

-- ── Schema documentation ──
COMMENT ON TABLE circles IS 'Groups for sharing wishlists. A direct circle (is_direct=true) is a 1-on-1 pair with max 2 members';
COMMENT ON COLUMN circles.name IS 'NULL for direct circles — the UI shows the other member''s name instead';
COMMENT ON COLUMN circles.is_direct IS 'true = 1-on-1 pair enforced by trigger (max 2 members), false = group circle';

COMMENT ON TABLE circle_members IS 'Users belonging to a circle. Owner auto-added via trigger on circle creation';
COMMENT ON COLUMN circle_members.role IS 'owner = can manage members/settings, member = can view/share items';

COMMENT ON TABLE circle_items IS 'Items shared into a circle — makes them visible to all circle members';
COMMENT ON COLUMN circle_items.shared_by IS 'The user who shared this item into the circle (may differ from item owner via share rules)';

COMMENT ON TABLE circle_events IS 'Activity feed for circles — immutable audit log of all actions';

COMMENT ON TABLE circle_share_rules IS 'Controls what items a user''s circles can see. Defaults to none (explicit opt-in sharing)';
COMMENT ON COLUMN circle_share_rules.share_mode IS 'none = share nothing, all = share everything, categories = share specific categories, selection = hand-picked items';
COMMENT ON COLUMN circle_share_rules.category_ids IS 'UUID array of categories to share — used when share_mode = categories';

COMMENT ON TABLE circle_invites IS 'Tokenized circle invitations with usage limits and expiration';
COMMENT ON COLUMN circle_invites.max_uses IS 'How many times this invite can be used. 1 = single-use';
COMMENT ON COLUMN circle_invites.use_count IS 'Incremented each time someone joins via this invite';
