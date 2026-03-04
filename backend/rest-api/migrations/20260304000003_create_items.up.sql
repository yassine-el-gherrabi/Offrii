CREATE TABLE items (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name             VARCHAR(255) NOT NULL,
    description      TEXT,
    url              VARCHAR(2048),
    estimated_price  DECIMAL(10,2),
    priority         SMALLINT NOT NULL DEFAULT 2
                     CHECK (priority BETWEEN 1 AND 3),
    category_id      UUID REFERENCES categories(id) ON DELETE SET NULL,
    status           VARCHAR(20) NOT NULL DEFAULT 'active'
                     CHECK (status IN ('active', 'purchased', 'deleted')),
    purchased_at     TIMESTAMPTZ,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_items_user_status ON items(user_id, status);
CREATE INDEX idx_items_user_priority ON items(user_id, priority);
CREATE INDEX idx_items_created_at ON items(created_at);

CREATE TRIGGER trg_items_updated_at
    BEFORE UPDATE ON items
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
