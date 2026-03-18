-- Re-add user_id column
ALTER TABLE categories ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE CASCADE;

-- Recreate per-user unique index
CREATE UNIQUE INDEX uq_categories_user_name
    ON categories (user_id, name) WHERE user_id IS NOT NULL;
