ALTER TABLE users ADD COLUMN IF NOT EXISTS last_active_at TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN IF NOT EXISTS inactivity_notice_sent_at TIMESTAMPTZ;
-- Backfill: set last_active_at to the most recent known activity
UPDATE users SET last_active_at = COALESCE(updated_at, created_at) WHERE last_active_at IS NULL;
