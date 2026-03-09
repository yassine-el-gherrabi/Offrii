ALTER TABLE share_links
    DROP COLUMN label,
    DROP COLUMN permissions,
    DROP COLUMN scope,
    DROP COLUMN scope_data,
    DROP COLUMN is_active;
