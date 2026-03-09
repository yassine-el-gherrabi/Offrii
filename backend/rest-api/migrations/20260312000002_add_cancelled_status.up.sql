-- Allow 'cancelled' status for friend requests (sender cancels)
ALTER TABLE friend_requests DROP CONSTRAINT IF EXISTS friend_requests_status_check;
ALTER TABLE friend_requests ADD CONSTRAINT friend_requests_status_check
    CHECK (status IN ('pending', 'accepted', 'declined', 'cancelled'));
