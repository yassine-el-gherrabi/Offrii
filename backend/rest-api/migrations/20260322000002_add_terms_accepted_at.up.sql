ALTER TABLE users ADD COLUMN terms_accepted_at TIMESTAMPTZ;
-- Backfill existing users (they implicitly accepted by using the service)
UPDATE users SET terms_accepted_at = created_at WHERE terms_accepted_at IS NULL;
