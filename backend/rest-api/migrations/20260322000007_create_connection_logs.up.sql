CREATE TABLE connection_logs (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ip         VARCHAR(45) NOT NULL,
    user_agent VARCHAR(512) NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_connection_logs_user ON connection_logs(user_id);
CREATE INDEX idx_connection_logs_created ON connection_logs(created_at);
