CREATE TABLE circle_items (
    circle_id UUID NOT NULL REFERENCES circles(id) ON DELETE CASCADE,
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    shared_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    shared_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (circle_id, item_id)
);
CREATE INDEX idx_circle_items_item_id ON circle_items(item_id);

CREATE TABLE circle_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    circle_id UUID NOT NULL REFERENCES circles(id) ON DELETE CASCADE,
    actor_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    event_type VARCHAR(30) NOT NULL CHECK (event_type IN (
        'item_shared','item_unshared','item_claimed','item_unclaimed','member_joined','member_left'
    )),
    target_item_id UUID REFERENCES items(id) ON DELETE SET NULL,
    target_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_circle_events_circle_created ON circle_events(circle_id, created_at DESC);
