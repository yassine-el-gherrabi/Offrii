DELETE FROM categories WHERE user_id IS NULL AND is_default = TRUE;
