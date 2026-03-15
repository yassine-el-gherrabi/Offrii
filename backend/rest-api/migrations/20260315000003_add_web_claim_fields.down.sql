DROP INDEX IF EXISTS idx_items_web_claim_token;

ALTER TABLE items DROP COLUMN IF EXISTS web_claim_token;
ALTER TABLE items DROP COLUMN IF EXISTS claimed_via_link_id;
ALTER TABLE items DROP COLUMN IF EXISTS claimed_name;
ALTER TABLE items DROP COLUMN IF EXISTS claimed_via;

-- Restore the original constraint
ALTER TABLE items ADD CONSTRAINT items_claim_nullity_chk
  CHECK (
    (claimed_by IS NULL AND claimed_at IS NULL)
    OR (claimed_by IS NOT NULL AND claimed_at IS NOT NULL)
  );

-- Restore the trigger
CREATE OR REPLACE FUNCTION fn_clear_claimed_at_on_null_by()
RETURNS TRIGGER AS $$
BEGIN
  IF NEW.claimed_by IS NULL AND OLD.claimed_by IS NOT NULL THEN
    NEW.claimed_at := NULL;
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_items_clear_claimed_at
  BEFORE UPDATE ON items
  FOR EACH ROW
  EXECUTE FUNCTION fn_clear_claimed_at_on_null_by();
