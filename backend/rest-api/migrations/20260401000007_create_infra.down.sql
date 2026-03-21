ALTER TABLE items DROP CONSTRAINT IF EXISTS fk_items_claimed_via_link;
DROP TABLE IF EXISTS share_links;
DROP TABLE IF EXISTS notifications;
DROP TABLE IF EXISTS refresh_tokens;
DROP TABLE IF EXISTS push_tokens;
