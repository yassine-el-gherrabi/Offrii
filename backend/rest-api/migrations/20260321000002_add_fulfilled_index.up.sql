CREATE INDEX IF NOT EXISTS idx_cw_fulfilled ON community_wishes(fulfilled_at DESC) WHERE status = 'fulfilled';
