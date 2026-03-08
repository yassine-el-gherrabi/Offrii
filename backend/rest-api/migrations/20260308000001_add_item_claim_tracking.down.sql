DROP INDEX IF EXISTS idx_items_claimed_by;
ALTER TABLE items
  DROP CONSTRAINT IF EXISTS items_claim_nullity_chk,
  DROP COLUMN claimed_at,
  DROP COLUMN claimed_by;
