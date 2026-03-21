ALTER TABLE notifications DROP CONSTRAINT IF EXISTS chk_notifications_type;
ALTER TABLE circle_share_rules DROP CONSTRAINT IF EXISTS chk_share_mode;
ALTER TABLE items DROP CONSTRAINT IF EXISTS chk_claimed_via;
