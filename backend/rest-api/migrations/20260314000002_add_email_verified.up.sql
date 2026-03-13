ALTER TABLE users ADD COLUMN IF NOT EXISTS email_verified BOOLEAN NOT NULL DEFAULT false;

-- OAuth users have their email verified by the provider
UPDATE users SET email_verified = true WHERE oauth_provider IS NOT NULL;
