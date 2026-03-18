-- Reassign items from per-user categories to the matching global default
-- by matching on the icon field (unique per default category).
UPDATE items i
SET category_id = g.id
FROM categories uc
JOIN categories g ON g.icon = uc.icon AND g.user_id IS NULL
WHERE i.category_id = uc.id
  AND uc.user_id IS NOT NULL;

-- Items that had a custom (non-default) category → set to NULL
UPDATE items
SET category_id = NULL
WHERE category_id IN (
    SELECT id FROM categories WHERE user_id IS NOT NULL
);

-- Delete all per-user categories (the copies + any custom ones)
DELETE FROM categories WHERE user_id IS NOT NULL;

-- Drop the per-user unique index (no longer needed)
DROP INDEX IF EXISTS uq_categories_user_name;

-- Drop the user_id column entirely
ALTER TABLE categories DROP COLUMN user_id;
