ALTER TABLE notifications ADD COLUMN wish_id UUID REFERENCES community_wishes(id) ON DELETE SET NULL;
