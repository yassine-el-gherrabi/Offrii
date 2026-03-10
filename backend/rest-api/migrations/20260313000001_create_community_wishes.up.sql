-- Add is_admin flag to users
ALTER TABLE users ADD COLUMN is_admin BOOLEAN NOT NULL DEFAULT FALSE;

-- Community wishes table
CREATE TABLE community_wishes (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
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
    is_anonymous    BOOLEAN NOT NULL DEFAULT FALSE,
    matched_with    UUID REFERENCES users(id) ON DELETE SET NULL,
    matched_at      TIMESTAMPTZ,
    fulfilled_at    TIMESTAMPTZ,
    closed_at       TIMESTAMPTZ,
    report_count    INT NOT NULL DEFAULT 0,
    reopen_count    INT NOT NULL DEFAULT 0,
    last_reopen_at  TIMESTAMPTZ,
    moderation_note TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Trigger for updated_at
CREATE TRIGGER set_community_wishes_updated_at
    BEFORE UPDATE ON community_wishes
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

-- Indexes
CREATE INDEX idx_cw_status_created ON community_wishes(status, created_at DESC);
CREATE INDEX idx_cw_owner ON community_wishes(owner_id);
CREATE INDEX idx_cw_category_open ON community_wishes(category, status) WHERE status = 'open';
CREATE INDEX idx_cw_matched ON community_wishes(matched_with) WHERE matched_with IS NOT NULL;
CREATE INDEX idx_cw_pending ON community_wishes(status) WHERE status IN ('pending', 'flagged');
