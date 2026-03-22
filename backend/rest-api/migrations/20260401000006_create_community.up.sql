-- ============================================================================
-- 006: Community wishes, reports, messages, blocks
-- ============================================================================

CREATE TABLE IF NOT EXISTS community_wishes (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title           VARCHAR(255) NOT NULL,
    description     TEXT,
    category        VARCHAR(30) NOT NULL
                    CHECK (category IN (
                        'education', 'clothing', 'health', 'religion',
                        'home', 'children', 'other'
                    )),
    status          VARCHAR(20) NOT NULL DEFAULT 'pending'
                    CHECK (status IN (
                        'pending', 'flagged', 'rejected',
                        'open', 'matched', 'fulfilled', 'closed', 'review'
                    )),
    is_anonymous    BOOLEAN     NOT NULL DEFAULT FALSE,
    matched_with    UUID        REFERENCES users(id) ON DELETE SET NULL,
    matched_at      TIMESTAMPTZ,
    fulfilled_at    TIMESTAMPTZ,
    closed_at       TIMESTAMPTZ,
    report_count    INT         NOT NULL DEFAULT 0,
    reopen_count    INT         NOT NULL DEFAULT 0,
    last_reopen_at  TIMESTAMPTZ,
    moderation_note TEXT,
    image_url       TEXT,
    links           TEXT[],
    og_image_url    TEXT,
    og_title        VARCHAR(500),
    og_site_name    VARCHAR(200),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER set_community_wishes_updated_at
    BEFORE UPDATE ON community_wishes
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

CREATE INDEX IF NOT EXISTS idx_cw_status_created ON community_wishes(status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_cw_owner          ON community_wishes(owner_id);
CREATE INDEX IF NOT EXISTS idx_cw_category_open  ON community_wishes(category, status) WHERE status = 'open';
CREATE INDEX IF NOT EXISTS idx_cw_matched        ON community_wishes(matched_with) WHERE matched_with IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_cw_pending        ON community_wishes(status) WHERE status IN ('pending', 'flagged');
CREATE INDEX IF NOT EXISTS idx_cw_fulfilled      ON community_wishes(fulfilled_at DESC) WHERE status = 'fulfilled';

-- Now that community_wishes exists, attach the cleanup trigger on users
CREATE TRIGGER trg_cleanup_matched_wishes
    BEFORE DELETE ON users
    FOR EACH ROW
    EXECUTE FUNCTION cleanup_matched_wishes_on_user_delete();

-- ---------------------------------------------------------------------------
-- Wish reports
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS wish_reports (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    wish_id     UUID        NOT NULL REFERENCES community_wishes(id) ON DELETE CASCADE,
    reporter_id UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reason      VARCHAR(50) NOT NULL DEFAULT 'inappropriate'
                CHECK (reason IN ('inappropriate', 'spam', 'scam', 'other')),
    details     TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (wish_id, reporter_id)
);

CREATE INDEX IF NOT EXISTS idx_wr_wish ON wish_reports(wish_id);

-- ---------------------------------------------------------------------------
-- Wish messages
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS wish_messages (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    wish_id    UUID        NOT NULL REFERENCES community_wishes(id) ON DELETE CASCADE,
    sender_id  UUID        REFERENCES users(id) ON DELETE SET NULL,
    body       TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_wm_wish_created ON wish_messages(wish_id, created_at);

-- ---------------------------------------------------------------------------
-- Wish blocks
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS wish_blocks (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    wish_id    UUID        NOT NULL REFERENCES community_wishes(id) ON DELETE CASCADE,
    user_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (wish_id, user_id)
);

-- ── Schema documentation ──
COMMENT ON TABLE community_wishes IS 'Charitable requests for help — includes AI moderation and donor matching (first come, first served)';
COMMENT ON COLUMN community_wishes.category IS 'Domain-specific: education, clothing, health, religion, home, children, other';
COMMENT ON COLUMN community_wishes.status IS 'State machine: pending → (AI) → open/flagged → matched → fulfilled/closed. review = community-reported';
COMMENT ON COLUMN community_wishes.is_anonymous IS 'Hides owner identity from the community — useful for sensitive categories (health, religion)';
COMMENT ON COLUMN community_wishes.matched_with IS 'The donor. Auto-set on first offer (first come, first served). Only one donor at a time';
COMMENT ON COLUMN community_wishes.report_count IS 'Incremented per unique reporter. At threshold (5), status auto-moves to review';
COMMENT ON COLUMN community_wishes.reopen_count IS 'Max 2 reopens allowed per wish, with 24h cooldown between each';
COMMENT ON COLUMN community_wishes.moderation_note IS 'Admin/AI explanation for why a wish was flagged or rejected';

COMMENT ON TABLE wish_reports IS 'Community moderation — users flag inappropriate wishes. One report per user per wish';
COMMENT ON COLUMN wish_reports.reason IS 'One of: inappropriate, spam, scam, other';

COMMENT ON TABLE wish_messages IS 'Private chat between wish owner and matched donor. Messages deleted when match ends (privacy)';
COMMENT ON COLUMN wish_messages.sender_id IS 'SET NULL on user deletion — message body preserved but sender anonymized';

COMMENT ON TABLE wish_blocks IS 'Per-user wish hiding — blocked wishes excluded from list queries';
