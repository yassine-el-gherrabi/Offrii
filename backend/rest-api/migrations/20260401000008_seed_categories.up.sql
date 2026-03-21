-- ============================================================================
-- 008: Seed default categories
-- ============================================================================

INSERT INTO categories (name, icon, is_default, position)
VALUES
    ('Tech',    'laptop',  TRUE, 1),
    ('Mode',    'tshirt',  TRUE, 2),
    ('Maison',  'home',    TRUE, 3),
    ('Loisirs', 'gamepad', TRUE, 4),
    ('Santé',   'heart',   TRUE, 5),
    ('Autre',   'tag',     TRUE, 6)
ON CONFLICT DO NOTHING;
