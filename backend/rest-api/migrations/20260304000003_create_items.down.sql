DROP TRIGGER IF EXISTS trg_items_updated_at ON items;
DROP INDEX IF EXISTS idx_items_created_at;
DROP INDEX IF EXISTS idx_items_user_priority;
DROP INDEX IF EXISTS idx_items_user_status;
DROP TABLE IF EXISTS items;
