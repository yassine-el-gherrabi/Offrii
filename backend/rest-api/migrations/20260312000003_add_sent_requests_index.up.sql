CREATE INDEX idx_friend_requests_from_user
    ON friend_requests(from_user_id) WHERE status = 'pending';
