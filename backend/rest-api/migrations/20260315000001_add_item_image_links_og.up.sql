-- Items: add image upload, multi-links, and OpenGraph metadata
ALTER TABLE items ADD COLUMN IF NOT EXISTS image_url TEXT;
ALTER TABLE items ADD COLUMN IF NOT EXISTS links TEXT[];
ALTER TABLE items ADD COLUMN IF NOT EXISTS og_image_url TEXT;
ALTER TABLE items ADD COLUMN IF NOT EXISTS og_title VARCHAR(500);
ALTER TABLE items ADD COLUMN IF NOT EXISTS og_site_name VARCHAR(200);

-- Migrate existing single url → links[0]
UPDATE items SET links = ARRAY[url] WHERE url IS NOT NULL AND links IS NULL;

-- Community wishes: add OpenGraph metadata (image_url and links already exist)
ALTER TABLE community_wishes ADD COLUMN IF NOT EXISTS og_image_url TEXT;
ALTER TABLE community_wishes ADD COLUMN IF NOT EXISTS og_title VARCHAR(500);
ALTER TABLE community_wishes ADD COLUMN IF NOT EXISTS og_site_name VARCHAR(200);
