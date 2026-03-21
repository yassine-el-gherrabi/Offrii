-- ============================================================================
-- 002: Categories (global only, no user_id column)
-- ============================================================================

CREATE TABLE IF NOT EXISTS categories (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name       VARCHAR(100) NOT NULL,
    icon       VARCHAR(50),
    is_default BOOLEAN     NOT NULL DEFAULT FALSE,
    position   INTEGER     NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_categories_default_name
    ON categories (name);
