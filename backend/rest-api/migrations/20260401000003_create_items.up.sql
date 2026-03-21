-- ============================================================================
-- 003: Items (final state, no url column)
-- ============================================================================

CREATE TABLE IF NOT EXISTS items (
    id                UUID           PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id           UUID           NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name              VARCHAR(255)   NOT NULL,
    description       TEXT,
    estimated_price   DECIMAL(10,2),
    priority          SMALLINT       NOT NULL DEFAULT 2
                      CHECK (priority BETWEEN 1 AND 3),
    category_id       UUID           REFERENCES categories(id) ON DELETE SET NULL,
    status            VARCHAR(20)    NOT NULL DEFAULT 'active'
                      CHECK (status IN ('active', 'purchased', 'deleted')),
    purchased_at      TIMESTAMPTZ,
    created_at        TIMESTAMPTZ    NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ    NOT NULL DEFAULT NOW(),

    -- Claim fields
    claimed_by        UUID           REFERENCES users(id) ON DELETE SET NULL,
    claimed_at        TIMESTAMPTZ,
    claimed_via       VARCHAR(20),
    claimed_name      VARCHAR(100),
    claimed_via_link_id UUID,        -- FK added after share_links table exists
    web_claim_token   UUID,

    -- Media & OpenGraph
    image_url         TEXT,
    links             TEXT[],
    og_image_url      TEXT,
    og_title          VARCHAR(500),
    og_site_name      VARCHAR(200),

    -- Visibility
    is_private        BOOLEAN        NOT NULL DEFAULT FALSE,

    CONSTRAINT chk_claimed_via CHECK (
        claimed_via IS NULL OR claimed_via IN ('app', 'web')
    )
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_items_user_status     ON items(user_id, status);
CREATE INDEX IF NOT EXISTS idx_items_user_priority   ON items(user_id, priority);
CREATE INDEX IF NOT EXISTS idx_items_created_at      ON items(created_at);
CREATE INDEX IF NOT EXISTS idx_items_category_id     ON items(category_id);
CREATE INDEX IF NOT EXISTS idx_items_claimed_by      ON items(claimed_by) WHERE claimed_by IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_items_web_claim_token ON items(web_claim_token) WHERE web_claim_token IS NOT NULL;

-- updated_at trigger
CREATE TRIGGER trg_items_updated_at
    BEFORE UPDATE ON items
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Auto-set purchased_at when status changes to/from 'purchased'
CREATE OR REPLACE FUNCTION set_purchased_at()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF NEW.status = 'purchased' THEN
            IF NEW.purchased_at IS NULL THEN
                NEW.purchased_at = NOW();
            END IF;
        ELSE
            NEW.purchased_at = NULL;
        END IF;
        RETURN NEW;
    END IF;

    IF NEW.status = 'purchased'
       AND (OLD.status IS DISTINCT FROM 'purchased')
       AND NEW.purchased_at IS NULL
    THEN
        NEW.purchased_at = NOW();
    END IF;

    IF NEW.status != 'purchased' AND OLD.status = 'purchased' THEN
        NEW.purchased_at = NULL;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_items_set_purchased_at
    BEFORE INSERT OR UPDATE ON items
    FOR EACH ROW EXECUTE FUNCTION set_purchased_at();
