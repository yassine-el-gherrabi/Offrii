CREATE TABLE notifications (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type        VARCHAR(50) NOT NULL,
    title       TEXT NOT NULL,
    body        TEXT NOT NULL,
    read        BOOLEAN NOT NULL DEFAULT FALSE,
    circle_id   UUID REFERENCES circles(id) ON DELETE SET NULL,
    item_id     UUID REFERENCES items(id) ON DELETE SET NULL,
    actor_id    UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notifications_user_unread ON notifications(user_id, read) WHERE read = FALSE;
CREATE INDEX idx_notifications_user_created ON notifications(user_id, created_at DESC);
