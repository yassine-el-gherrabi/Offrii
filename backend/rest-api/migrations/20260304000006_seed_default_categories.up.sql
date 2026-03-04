INSERT INTO categories (user_id, name, icon, is_default, position)
VALUES
    (NULL, 'Tech',    'laptop',  TRUE, 1),
    (NULL, 'Mode',    'tshirt',  TRUE, 2),
    (NULL, 'Maison',  'home',    TRUE, 3),
    (NULL, 'Loisirs', 'gamepad', TRUE, 4),
    (NULL, 'Santé',   'heart',   TRUE, 5),
    (NULL, 'Autre',   'tag',     TRUE, 6)
ON CONFLICT DO NOTHING;
