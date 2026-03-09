-- The UNIQUE constraint on share_links.token already creates an implicit unique index.
-- This explicit index is redundant.
DROP INDEX IF EXISTS idx_share_links_token;
