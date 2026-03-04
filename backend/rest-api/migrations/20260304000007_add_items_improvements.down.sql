DROP TRIGGER IF EXISTS trg_items_set_purchased_at ON items;
DROP FUNCTION IF EXISTS set_purchased_at();
DROP INDEX IF EXISTS idx_items_category_id;
