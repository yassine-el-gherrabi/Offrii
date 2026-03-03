-- Seed: dev test data (applied via docker-compose.override.yml only)
-- Fixed UUIDs for deterministic testing

-- 1 test user (password: "password123" hashed with bcrypt)
INSERT INTO users (id, email, pseudo, password_hash)
VALUES (
    'a0000000-0000-0000-0000-000000000001',
    'test@offrii.com',
    'testuser',
    '$2b$12$LJ3m4ys3Lk0TSwMCkV6tOeUK0BQNZ6ASHnOFBfqMDBGqoMKbJqCa'
);

-- 3 items with varied priorities and statuses
INSERT INTO items (id, user_id, name, budget, priority, category, status) VALUES
    ('b0000000-0000-0000-0000-000000000001', 'a0000000-0000-0000-0000-000000000001',
     'Casque Sony WH-1000XM5', 350.00, 'Besoin', 'Tech', 'Active'),
    ('b0000000-0000-0000-0000-000000000002', 'a0000000-0000-0000-0000-000000000001',
     'Livre "Designing Data-Intensive Applications"', 45.00, 'Envie', 'Livres', 'Active'),
    ('b0000000-0000-0000-0000-000000000003', 'a0000000-0000-0000-0000-000000000001',
     'Cours en ligne Rust', NULL, 'Urgent', 'Formation', 'Purchased');

-- 2 links on the first item
INSERT INTO item_links (item_id, url, title, price) VALUES
    ('b0000000-0000-0000-0000-000000000001',
     'https://www.amazon.fr/dp/B09XS7JWHH', 'Amazon FR', 349.99),
    ('b0000000-0000-0000-0000-000000000001',
     'https://www.fnac.com/Sony-WH-1000XM5', 'Fnac', 359.00);

-- 1 circle created by the test user
INSERT INTO circles (id, created_by, name, emoji)
VALUES (
    'c0000000-0000-0000-0000-000000000001',
    'a0000000-0000-0000-0000-000000000001',
    'Famille',
    '👨‍👩‍👧‍👦'
);

-- Test user as Admin of the circle
INSERT INTO circle_members (circle_id, user_id, role)
VALUES (
    'c0000000-0000-0000-0000-000000000001',
    'a0000000-0000-0000-0000-000000000001',
    'Admin'
);

-- First item shared in the circle
INSERT INTO item_visibility (item_id, circle_id)
VALUES (
    'b0000000-0000-0000-0000-000000000001',
    'c0000000-0000-0000-0000-000000000001'
);
