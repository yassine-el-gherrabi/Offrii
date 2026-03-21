-- Index on refresh_tokens(user_id) already exists from 20260304000008.
-- Clean up expired tokens.
DELETE FROM refresh_tokens WHERE expires_at < NOW();
