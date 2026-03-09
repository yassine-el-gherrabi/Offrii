-- Step 1: Add nullable column
ALTER TABLE users ADD COLUMN username VARCHAR(30);

-- Step 2: Backfill existing users
-- Uses display_name (or email prefix if absent), lowercased, non-alpha stripped,
-- then appends '_' + 4 hex chars derived from the user's UUID.
-- Ensures the result starts with a letter and is 3-30 chars.
UPDATE users SET username = LEFT(
    CASE
        WHEN LENGTH(REGEXP_REPLACE(LOWER(COALESCE(NULLIF(display_name, ''), SPLIT_PART(email, '@', 1))), '[^a-z0-9]', '', 'g')) = 0
            THEN 'user'
        WHEN SUBSTRING(REGEXP_REPLACE(LOWER(COALESCE(NULLIF(display_name, ''), SPLIT_PART(email, '@', 1))), '[^a-z0-9]', '', 'g'), 1, 1) ~ '[0-9]'
            THEN 'u' || REGEXP_REPLACE(LOWER(COALESCE(NULLIF(display_name, ''), SPLIT_PART(email, '@', 1))), '[^a-z0-9]', '', 'g')
        ELSE REGEXP_REPLACE(LOWER(COALESCE(NULLIF(display_name, ''), SPLIT_PART(email, '@', 1))), '[^a-z0-9]', '', 'g')
    END,
    25  -- leave room for _xxxx suffix
) || '_' || SUBSTR(MD5(id::text), 1, 4);

-- Step 3: Enforce constraints
ALTER TABLE users ALTER COLUMN username SET NOT NULL;
ALTER TABLE users ADD CONSTRAINT users_username_unique UNIQUE (username);
ALTER TABLE users ADD CONSTRAINT users_username_format CHECK (username ~ '^[a-z][a-z0-9_]{2,29}$');
