CREATE INDEX IF NOT EXISTS idx_circle_invites_token ON circle_invites(token);
CREATE INDEX IF NOT EXISTS idx_verification_token ON email_verification_tokens(token);
CREATE INDEX IF NOT EXISTS idx_email_change_token ON email_change_tokens(token);
