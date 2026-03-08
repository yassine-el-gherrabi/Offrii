ALTER TABLE items
  ADD COLUMN claimed_by UUID REFERENCES users(id) ON DELETE SET NULL,
  ADD COLUMN claimed_at TIMESTAMPTZ,
  ADD CONSTRAINT items_claim_nullity_chk
    CHECK (
      (claimed_by IS NULL AND claimed_at IS NULL)
      OR (claimed_by IS NOT NULL AND claimed_at IS NOT NULL)
    );

-- Clear claimed_at when claimed_by is set to NULL (e.g. FK ON DELETE SET NULL).
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

CREATE INDEX idx_items_claimed_by ON items(claimed_by) WHERE claimed_by IS NOT NULL;
