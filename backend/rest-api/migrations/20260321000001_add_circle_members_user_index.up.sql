-- Index for "list circles by user" queries (PK is (circle_id, user_id) — only covers circle_id-first lookups)
CREATE INDEX IF NOT EXISTS idx_circle_members_user ON circle_members(user_id);
