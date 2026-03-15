ALTER TABLE community_wishes DROP COLUMN IF EXISTS og_site_name;
ALTER TABLE community_wishes DROP COLUMN IF EXISTS og_title;
ALTER TABLE community_wishes DROP COLUMN IF EXISTS og_image_url;

ALTER TABLE items DROP COLUMN IF EXISTS og_site_name;
ALTER TABLE items DROP COLUMN IF EXISTS og_title;
ALTER TABLE items DROP COLUMN IF EXISTS og_image_url;
ALTER TABLE items DROP COLUMN IF EXISTS links;
ALTER TABLE items DROP COLUMN IF EXISTS image_url;
