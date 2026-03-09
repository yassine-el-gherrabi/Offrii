ALTER TABLE share_links
    ADD COLUMN label       VARCHAR(100),
    ADD COLUMN permissions VARCHAR(20) NOT NULL DEFAULT 'view_and_claim'
        CHECK (permissions IN ('view_only', 'view_and_claim')),
    ADD COLUMN scope       VARCHAR(20) NOT NULL DEFAULT 'all'
        CHECK (scope IN ('all', 'category', 'selection')),
    ADD COLUMN scope_data  JSONB,
    ADD COLUMN is_active   BOOLEAN NOT NULL DEFAULT TRUE;
