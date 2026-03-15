-- Add web claim tracking columns
ALTER TABLE items ADD COLUMN IF NOT EXISTS claimed_via VARCHAR(20);
ALTER TABLE items ADD COLUMN IF NOT EXISTS claimed_name VARCHAR(100);
ALTER TABLE items ADD COLUMN IF NOT EXISTS claimed_via_link_id UUID REFERENCES share_links(id) ON DELETE SET NULL;
ALTER TABLE items ADD COLUMN IF NOT EXISTS web_claim_token UUID;

-- Drop the old constraint that required claimed_by and claimed_at to be both null or both non-null
-- (web claims have claimed_at set but claimed_by NULL)
ALTER TABLE items DROP CONSTRAINT IF EXISTS items_claim_nullity_chk;

-- Drop the trigger that clears claimed_at when claimed_by becomes NULL
-- (web claims legitimately have claimed_by=NULL with claimed_at set)
DROP TRIGGER IF EXISTS trg_items_clear_claimed_at ON items;
DROP FUNCTION IF EXISTS fn_clear_claimed_at_on_null_by();

-- Backfill: mark existing claims as 'app' claims
UPDATE items SET claimed_via = 'app' WHERE claimed_by IS NOT NULL AND claimed_via IS NULL;

-- Index for web_claim_token lookups
CREATE INDEX idx_items_web_claim_token ON items(web_claim_token) WHERE web_claim_token IS NOT NULL;
