-- Map any existing cancelled rows to declined before restoring the constraint
UPDATE friend_requests SET status = 'declined' WHERE status = 'cancelled';

-- Revert to original status check (remove 'cancelled')
ALTER TABLE friend_requests DROP CONSTRAINT IF EXISTS friend_requests_status_check;
ALTER TABLE friend_requests ADD CONSTRAINT friend_requests_status_check
    CHECK (status IN ('pending', 'accepted', 'declined'));
