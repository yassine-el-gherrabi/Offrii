DROP TRIGGER IF EXISTS trg_check_direct_circle_member_limit ON circle_members;
DROP FUNCTION IF EXISTS fn_check_direct_circle_member_limit();
UPDATE circles SET name = '' WHERE name IS NULL;
ALTER TABLE circles ALTER COLUMN name SET NOT NULL;
ALTER TABLE circles DROP COLUMN IF EXISTS is_direct;
