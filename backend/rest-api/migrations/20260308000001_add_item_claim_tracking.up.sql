ALTER TABLE items
  ADD COLUMN claimed_by UUID REFERENCES users(id) ON DELETE SET NULL,
  ADD COLUMN claimed_at TIMESTAMPTZ;

CREATE INDEX idx_items_claimed_by ON items(claimed_by) WHERE claimed_by IS NOT NULL;
