DROP TRIGGER IF EXISTS trg_items_set_purchased_at ON items;
DROP FUNCTION IF EXISTS set_purchased_at();
DROP TRIGGER IF EXISTS trg_items_updated_at ON items;
DROP TABLE IF EXISTS items;
