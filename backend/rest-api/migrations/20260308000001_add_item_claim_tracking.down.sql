DROP INDEX IF EXISTS idx_items_claimed_by;
ALTER TABLE items DROP COLUMN claimed_at, DROP COLUMN claimed_by;
