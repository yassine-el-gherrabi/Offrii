-- Migration 0003: Create indexes
-- 7 indexes for query performance (UNIQUE constraints already create implicit indexes)

-- Items by user filtered by status (frequent query)
CREATE INDEX idx_items_user_status
    ON items(user_id, status);

-- Active reservation unique per item/circle (partial unique index)
CREATE UNIQUE INDEX idx_reservations_active
    ON reservations(item_id, circle_id)
    WHERE status = 'Reserved';

-- Message pagination by conversation
CREATE INDEX idx_messages_conversation_created
    ON messages(conversation_id, created_at);

-- Community wishes feed sorted by status + date
CREATE INDEX idx_community_wishes_status_created
    ON community_wishes(status, created_at DESC);

-- Notification center: unread first, then by date
CREATE INDEX idx_notifications_user_read_created
    ON notifications(user_id, read, created_at DESC);

-- Moderation queue sorted by status + date
CREATE INDEX idx_reports_status_created
    ON reports(status, created_at DESC);

-- Item links lookup by item
CREATE INDEX idx_item_links_item_id
    ON item_links(item_id);
