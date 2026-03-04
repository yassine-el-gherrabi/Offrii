CREATE TABLE categories (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id          UUID REFERENCES users(id) ON DELETE CASCADE,
    name             VARCHAR(100) NOT NULL,
    icon             VARCHAR(50),
    is_default       BOOLEAN NOT NULL DEFAULT FALSE,
    position         INTEGER NOT NULL DEFAULT 0,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX uq_categories_user_name
    ON categories (user_id, name) WHERE user_id IS NOT NULL;

CREATE UNIQUE INDEX uq_categories_default_name
    ON categories (name) WHERE user_id IS NULL;
