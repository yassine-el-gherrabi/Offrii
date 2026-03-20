-- Add username_customized flag to track whether the user has explicitly chosen their username.
ALTER TABLE users ADD COLUMN username_customized BOOLEAN NOT NULL DEFAULT false;

-- Backfill: users whose username does NOT match the auto-generated pattern
-- (base + '_' + 4 hex chars) are considered to have customized their username.
-- Auto-generated usernames always end with '_' followed by exactly 4 hex characters.
UPDATE users SET username_customized = true
WHERE username !~ '_[0-9a-f]{4}$';
