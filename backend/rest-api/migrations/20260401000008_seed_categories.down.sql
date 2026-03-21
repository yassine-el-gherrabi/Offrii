DELETE FROM categories
WHERE is_default = TRUE
  AND name IN ('Tech', 'Mode', 'Maison', 'Loisirs', 'Santé', 'Autre');
