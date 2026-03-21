-- =============================================================================
-- Offrii Dev Seed Data
-- =============================================================================
-- Idempotent: all INSERTs use ON CONFLICT DO NOTHING
-- Deterministic: all UUIDs are hardcoded
-- Run inside a transaction so partial failures roll back cleanly
-- Password for all users: DemoPass123x
-- =============================================================================

BEGIN;

-- ─────────────────────────────────────────────────────────────────────────────
-- 0. LOOK UP DEFAULT CATEGORY IDs
-- ─────────────────────────────────────────────────────────────────────────────
-- Categories are seeded by migration 20260304000006 and then made global by
-- 20260318000002 (user_id column dropped). We select them into temp variables
-- so the rest of the script can reference them reliably.
-- ─────────────────────────────────────────────────────────────────────────────

DO $$
DECLARE
    cat_tech    UUID;
    cat_mode    UUID;
    cat_maison  UUID;
    cat_loisirs UUID;
    cat_sante   UUID;
    cat_autre   UUID;
BEGIN
    SELECT id INTO cat_tech    FROM categories WHERE name = 'Tech'    LIMIT 1;
    SELECT id INTO cat_mode    FROM categories WHERE name = 'Mode'    LIMIT 1;
    SELECT id INTO cat_maison  FROM categories WHERE name = 'Maison'  LIMIT 1;
    SELECT id INTO cat_loisirs FROM categories WHERE name = 'Loisirs' LIMIT 1;
    SELECT id INTO cat_sante   FROM categories WHERE name = 'Santé'   LIMIT 1;
    SELECT id INTO cat_autre   FROM categories WHERE name = 'Autre'   LIMIT 1;

    -- Store in a temp table so plain SQL statements below can JOIN on it
    CREATE TEMP TABLE IF NOT EXISTS _cat (key TEXT PRIMARY KEY, id UUID);
    INSERT INTO _cat VALUES
        ('tech',    cat_tech),
        ('mode',    cat_mode),
        ('maison',  cat_maison),
        ('loisirs', cat_loisirs),
        ('sante',   cat_sante),
        ('autre',   cat_autre)
    ON CONFLICT (key) DO UPDATE SET id = EXCLUDED.id;
END $$;

-- =============================================================================
-- 1. USERS  (8 users)
-- =============================================================================
-- Password hash is Argon2 of "Password123!" for all email+password users.
-- Users with OAuth only have password_hash = NULL.
-- Final schema columns (after all migrations):
--   id, email, username, password_hash, display_name,
--   oauth_provider, oauth_provider_id, email_verified,
--   token_version, is_admin, username_customized, avatar_url,
--   created_at, updated_at
-- (reminder columns dropped by 20260321000004)
-- =============================================================================

INSERT INTO users (id, email, username, password_hash, display_name,
                   oauth_provider, oauth_provider_id, email_verified,
                   token_version, is_admin, username_customized, avatar_url,
                   created_at, updated_at)
VALUES
    -- u1: Yassine — admin, full profile, email+password, verified, old account
    ('a0000000-0000-4000-a000-000000000001',
     'yassine@demo.com', 'yassine',
     '$argon2id$v=19$m=19456,t=2,p=1$Rnb2dLECe4pmR8IpUTKXuA$u+Rr8yJU8n4XiI/gqWzVA8l1ebaORyH+JqvJb99oEIg',
     'Yassine', NULL, NULL, TRUE,
     1, TRUE, TRUE,
     'https://cdn.offrii.com/avatars/demo-yassine.jpg',
     NOW() - INTERVAL '30 days', NOW() - INTERVAL '30 days'),

    -- u2: Marie — regular user, full profile, verified
    ('a0000000-0000-4000-a000-000000000002',
     'marie@demo.com', 'marie_dupont',
     '$argon2id$v=19$m=19456,t=2,p=1$Rnb2dLECe4pmR8IpUTKXuA$u+Rr8yJU8n4XiI/gqWzVA8l1ebaORyH+JqvJb99oEIg',
     'Marie Dupont', NULL, NULL, TRUE,
     1, FALSE, TRUE,
     'https://cdn.offrii.com/avatars/demo-marie.jpg',
     NOW() - INTERVAL '14 days', NOW() - INTERVAL '14 days'),

    -- u3: Lucas — minimal user, not verified, auto username
    ('a0000000-0000-4000-a000-000000000003',
     'lucas@demo.com', 'lucas123',
     '$argon2id$v=19$m=19456,t=2,p=1$Rnb2dLECe4pmR8IpUTKXuA$u+Rr8yJU8n4XiI/gqWzVA8l1ebaORyH+JqvJb99oEIg',
     NULL, NULL, NULL, FALSE,
     1, FALSE, FALSE, NULL,
     NOW() - INTERVAL '7 days', NOW() - INTERVAL '7 days'),

    -- u4: Sophie — Google OAuth only, no password
    ('a0000000-0000-4000-a000-000000000004',
     'sophie@gmail.com', 'sophie_martin',
     NULL,
     'Sophie Martin', 'google', 'google_sophie_123', TRUE,
     1, FALSE, TRUE,
     'https://lh3.googleusercontent.com/demo-sophie',
     NOW() - INTERVAL '10 days', NOW() - INTERVAL '10 days'),

    -- u5: Thomas — Apple OAuth only, no password
    ('a0000000-0000-4000-a000-000000000005',
     'thomas@icloud.com', 'thomas_b',
     NULL,
     NULL, 'apple', 'apple_thomas_456', TRUE,
     1, FALSE, TRUE, NULL,
     NOW() - INTERVAL '5 days', NOW() - INTERVAL '5 days'),

    -- u6: Camille — email+password AND Google OAuth linked
    ('a0000000-0000-4000-a000-000000000006',
     'camille@demo.com', 'camille_r',
     '$argon2id$v=19$m=19456,t=2,p=1$Rnb2dLECe4pmR8IpUTKXuA$u+Rr8yJU8n4XiI/gqWzVA8l1ebaORyH+JqvJb99oEIg',
     'Camille R.', 'google', 'google_camille_789', TRUE,
     1, FALSE, TRUE, NULL,
     NOW() - INTERVAL '12 days', NOW() - INTERVAL '12 days'),

    -- u7: New User — very recent account (< 24h)
    ('a0000000-0000-4000-a000-000000000007',
     'newuser@demo.com', 'new_user',
     '$argon2id$v=19$m=19456,t=2,p=1$Rnb2dLECe4pmR8IpUTKXuA$u+Rr8yJU8n4XiI/gqWzVA8l1ebaORyH+JqvJb99oEIg',
     'Nouveau', NULL, NULL, TRUE,
     1, FALSE, TRUE, NULL,
     NOW() - INTERVAL '1 hour', NOW() - INTERVAL '1 hour'),

    -- u8: Reporter — used for reporting/blocking scenarios
    ('a0000000-0000-4000-a000-000000000008',
     'reporter@demo.com', 'reporter_user',
     '$argon2id$v=19$m=19456,t=2,p=1$Rnb2dLECe4pmR8IpUTKXuA$u+Rr8yJU8n4XiI/gqWzVA8l1ebaORyH+JqvJb99oEIg',
     'Reporter', NULL, NULL, TRUE,
     1, FALSE, TRUE, NULL,
     NOW() - INTERVAL '10 days', NOW() - INTERVAL '10 days')
ON CONFLICT (id) DO NOTHING;


-- =============================================================================
-- 2. ITEMS  (~20 items)
-- =============================================================================
-- Covers: all 3 statuses, all 3 priorities, with/without category, description,
-- price, image, links, OG metadata, private items, claimed items (app + web).
--
-- category_id uses a subselect from the temp _cat table.
-- =============================================================================

INSERT INTO items (id, user_id, name, description, estimated_price,
                   priority, category_id, status, is_private,
                   image_url, links, og_image_url, og_title, og_site_name,
                   claimed_by, claimed_at, claimed_via, claimed_name, claimed_via_link_id, web_claim_token,
                   created_at, updated_at)
VALUES
    -- ── Yassine's items (u1) ─────────────────────────────────────────────
    -- b01: Active, high priority, Tech, with links + OG data
    ('b0000000-0000-4000-a000-000000000001',
     'a0000000-0000-4000-a000-000000000001',
     'MacBook Pro M4', 'Le nouveau MacBook avec puce M4 Max',
     2999.00,
     1, (SELECT id FROM _cat WHERE key = 'tech'), 'active', FALSE,
     NULL, ARRAY['https://www.apple.com/fr/macbook-pro/'], 'https://www.apple.com/v/macbook-pro/og.jpg', 'MacBook Pro', 'Apple',
     NULL, NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '28 days', NOW() - INTERVAL '28 days'),

    -- b02: Active, medium priority, Mode, no links
    ('b0000000-0000-4000-a000-000000000002',
     'a0000000-0000-4000-a000-000000000001',
     'Veste en cuir Sandro', NULL,
     450.00,
     2, (SELECT id FROM _cat WHERE key = 'mode'), 'active', FALSE,
     'https://cdn.offrii.com/items/demo-veste.jpg', NULL, NULL, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '20 days', NOW() - INTERVAL '20 days'),

    -- b03: Active, low priority, no category, no price, private
    ('b0000000-0000-4000-a000-000000000003',
     'a0000000-0000-4000-a000-000000000001',
     'Journal intime Moleskine', 'Un beau carnet pour ecrire',
     NULL,
     3, NULL, 'active', TRUE,
     NULL, NULL, NULL, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '15 days', NOW() - INTERVAL '15 days'),

    -- b04: Purchased, claimed by Marie (app claim)
    ('b0000000-0000-4000-a000-000000000004',
     'a0000000-0000-4000-a000-000000000001',
     'Casque Sony WH-1000XM5', 'Casque a reduction de bruit',
     350.00,
     1, (SELECT id FROM _cat WHERE key = 'tech'), 'purchased', FALSE,
     NULL, ARRAY['https://www.sony.fr/headphones/wh-1000xm5'], 'https://www.sony.fr/og-xm5.jpg', 'WH-1000XM5', 'Sony',
     'a0000000-0000-4000-a000-000000000002', NOW() - INTERVAL '3 days', 'app', NULL, NULL, NULL,
     NOW() - INTERVAL '25 days', NOW() - INTERVAL '3 days'),

    -- b05: Deleted item
    ('b0000000-0000-4000-a000-000000000005',
     'a0000000-0000-4000-a000-000000000001',
     'Ancien souhait supprime', NULL,
     NULL,
     2, NULL, 'deleted', FALSE,
     NULL, NULL, NULL, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '29 days', NOW() - INTERVAL '10 days'),

    -- b06: Active, Maison category, with image
    ('b0000000-0000-4000-a000-000000000006',
     'a0000000-0000-4000-a000-000000000001',
     'Lampe Dyson Solarcycle', NULL,
     599.00,
     2, (SELECT id FROM _cat WHERE key = 'maison'), 'active', FALSE,
     'https://cdn.offrii.com/items/demo-lampe.jpg', ARRAY['https://www.dyson.fr/eclairage/lampes-de-bureau/solarcycle'], NULL, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '5 days', NOW() - INTERVAL '5 days'),

    -- b07: Active, Loisirs, claimed via web by anonymous
    ('b0000000-0000-4000-a000-000000000007',
     'a0000000-0000-4000-a000-000000000001',
     'Zelda Tears of the Kingdom', 'Edition collector Switch',
     69.99,
     2, (SELECT id FROM _cat WHERE key = 'loisirs'), 'active', FALSE,
     NULL, ARRAY['https://www.nintendo.fr/zelda-totk'], NULL, NULL, NULL,
     NULL, NOW() - INTERVAL '1 day', 'web', 'Maman', NULL, 'f0000000-0000-4000-a000-000000000001',
     NOW() - INTERVAL '18 days', NOW() - INTERVAL '1 day'),

    -- b08: Active, Sante, high priority
    ('b0000000-0000-4000-a000-000000000008',
     'a0000000-0000-4000-a000-000000000001',
     'Tapis de yoga Lululemon', 'Le modele Reversible 5mm',
     88.00,
     1, (SELECT id FROM _cat WHERE key = 'sante'), 'active', FALSE,
     NULL, NULL, NULL, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '2 days', NOW() - INTERVAL '2 days'),

    -- b09: Active, Autre category, minimal
    ('b0000000-0000-4000-a000-000000000009',
     'a0000000-0000-4000-a000-000000000001',
     'Carte cadeau FNAC 50EUR', NULL,
     50.00,
     3, (SELECT id FROM _cat WHERE key = 'autre'), 'active', FALSE,
     NULL, NULL, NULL, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '1 day', NOW() - INTERVAL '1 day'),

    -- ── Marie's items (u2) ───────────────────────────────────────────────
    -- b10: Active, high priority, Mode
    ('b0000000-0000-4000-a000-000000000010',
     'a0000000-0000-4000-a000-000000000002',
     'Sac Longchamp Le Pliage', 'Modele grand format en noir',
     145.00,
     1, (SELECT id FROM _cat WHERE key = 'mode'), 'active', FALSE,
     'https://cdn.offrii.com/items/demo-sac.jpg', ARRAY['https://www.longchamp.com/fr/le-pliage'], 'https://www.longchamp.com/og.jpg', 'Le Pliage', 'Longchamp',
     NULL, NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '12 days', NOW() - INTERVAL '12 days'),

    -- b11: Active, medium, Maison, claimed by Yassine (app)
    ('b0000000-0000-4000-a000-000000000011',
     'a0000000-0000-4000-a000-000000000002',
     'Bougie Diptyque Baies', 'La grande 300g',
     68.00,
     2, (SELECT id FROM _cat WHERE key = 'maison'), 'active', FALSE,
     NULL, NULL, NULL, NULL, NULL,
     'a0000000-0000-4000-a000-000000000001', NOW() - INTERVAL '2 days', 'app', NULL, NULL, NULL,
     NOW() - INTERVAL '10 days', NOW() - INTERVAL '2 days'),

    -- b12: Active, low priority, Tech, no description
    ('b0000000-0000-4000-a000-000000000012',
     'a0000000-0000-4000-a000-000000000002',
     'AirPods Pro 3', NULL,
     279.00,
     3, (SELECT id FROM _cat WHERE key = 'tech'), 'active', FALSE,
     NULL, ARRAY['https://www.apple.com/fr/airpods-pro/'], NULL, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '8 days', NOW() - INTERVAL '8 days'),

    -- b13: Purchased (self-purchased, no claimer)
    ('b0000000-0000-4000-a000-000000000013',
     'a0000000-0000-4000-a000-000000000002',
     'Livre "Devenir" de Michelle Obama', NULL,
     24.90,
     2, NULL, 'purchased', FALSE,
     NULL, NULL, NULL, NULL, NULL,
     NULL, NOW() - INTERVAL '5 days', NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '13 days', NOW() - INTERVAL '5 days'),

    -- b14: Active, private item
    ('b0000000-0000-4000-a000-000000000014',
     'a0000000-0000-4000-a000-000000000002',
     'Surprise pour anniversaire Yassine', 'Ne pas montrer!',
     200.00,
     1, NULL, 'active', TRUE,
     NULL, NULL, NULL, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '6 days', NOW() - INTERVAL '6 days'),

    -- ── Lucas's items (u3) ───────────────────────────────────────────────
    -- b15: Active, minimal item, no category/desc/price
    ('b0000000-0000-4000-a000-000000000015',
     'a0000000-0000-4000-a000-000000000003',
     'Un truc cool', NULL,
     NULL,
     2, NULL, 'active', FALSE,
     NULL, NULL, NULL, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '6 days', NOW() - INTERVAL '6 days'),

    -- b16: Active, Loisirs
    ('b0000000-0000-4000-a000-000000000016',
     'a0000000-0000-4000-a000-000000000003',
     'Manette PS5 DualSense', 'Couleur Cosmic Red',
     69.99,
     1, (SELECT id FROM _cat WHERE key = 'loisirs'), 'active', FALSE,
     NULL, NULL, NULL, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '4 days', NOW() - INTERVAL '4 days'),

    -- ── Sophie's items (u4) ─────────────────────────────────────────────
    -- b17: Active, high priority
    ('b0000000-0000-4000-a000-000000000017',
     'a0000000-0000-4000-a000-000000000004',
     'Parfum Chanel N5', 'Eau de parfum 100ml',
     135.00,
     1, (SELECT id FROM _cat WHERE key = 'mode'), 'active', FALSE,
     NULL, ARRAY['https://www.chanel.com/fr/parfums/n5/'], NULL, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '9 days', NOW() - INTERVAL '9 days'),

    -- ── Camille's items (u6) ─────────────────────────────────────────────
    -- b18: Active, Maison
    ('b0000000-0000-4000-a000-000000000018',
     'a0000000-0000-4000-a000-000000000006',
     'Plaid en laine Zara Home', NULL,
     59.99,
     2, (SELECT id FROM _cat WHERE key = 'maison'), 'active', FALSE,
     NULL, NULL, NULL, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '11 days', NOW() - INTERVAL '11 days'),

    -- ── New User's items (u7) ────────────────────────────────────────────
    -- b19: Active, just created
    ('b0000000-0000-4000-a000-000000000019',
     'a0000000-0000-4000-a000-000000000007',
     'Mon premier souhait', 'Je decouvre Offrii!',
     NULL,
     2, NULL, 'active', FALSE,
     NULL, NULL, NULL, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '30 minutes', NOW() - INTERVAL '30 minutes'),

    -- b20: Active, Yassine, multiple links
    ('b0000000-0000-4000-a000-000000000020',
     'a0000000-0000-4000-a000-000000000001',
     'Sneakers Nike Air Max 90', 'Coloris blanc/gris taille 43',
     150.00,
     2, (SELECT id FROM _cat WHERE key = 'mode'), 'active', FALSE,
     'https://cdn.offrii.com/items/demo-sneakers.jpg',
     ARRAY['https://www.nike.com/fr/air-max-90', 'https://www.zalando.fr/nike-air-max-90'],
     'https://www.nike.com/og-am90.jpg', 'Air Max 90', 'Nike',
     NULL, NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '3 days', NOW() - INTERVAL '3 days')
ON CONFLICT (id) DO NOTHING;


-- =============================================================================
-- 3. FRIENDSHIPS & FRIEND REQUESTS
-- =============================================================================
-- Friendships require user_a_id < user_b_id (canonical ordering).
-- UUID ordering: ...0001 < ...0002 < ...0003 < ...0004 < ...0006 < ...0008
-- =============================================================================

-- Friend requests (accepted ones + pending + declined)
INSERT INTO friend_requests (id, from_user_id, to_user_id, status, created_at)
VALUES
    -- u1→u2: accepted (friendship exists)
    ('f1000000-0000-4000-a000-000000000001',
     'a0000000-0000-4000-a000-000000000001', 'a0000000-0000-4000-a000-000000000002',
     'accepted', NOW() - INTERVAL '13 days'),

    -- u1→u3: accepted (friendship exists)
    ('f1000000-0000-4000-a000-000000000002',
     'a0000000-0000-4000-a000-000000000001', 'a0000000-0000-4000-a000-000000000003',
     'accepted', NOW() - INTERVAL '6 days'),

    -- u2→u6: accepted (friendship exists)
    ('f1000000-0000-4000-a000-000000000003',
     'a0000000-0000-4000-a000-000000000002', 'a0000000-0000-4000-a000-000000000006',
     'accepted', NOW() - INTERVAL '11 days'),

    -- u4→u1: pending friend request
    ('f1000000-0000-4000-a000-000000000004',
     'a0000000-0000-4000-a000-000000000004', 'a0000000-0000-4000-a000-000000000001',
     'pending', NOW() - INTERVAL '2 days'),

    -- u3→u2: declined friend request
    ('f1000000-0000-4000-a000-000000000005',
     'a0000000-0000-4000-a000-000000000003', 'a0000000-0000-4000-a000-000000000002',
     'declined', NOW() - INTERVAL '5 days')
ON CONFLICT (from_user_id, to_user_id) DO NOTHING;

-- Actual friendships (canonical ordering: user_a_id < user_b_id)
INSERT INTO friendships (user_a_id, user_b_id, created_at)
VALUES
    -- u1 ↔ u2  (0001 < 0002)
    ('a0000000-0000-4000-a000-000000000001', 'a0000000-0000-4000-a000-000000000002',
     NOW() - INTERVAL '13 days'),

    -- u1 ↔ u3  (0001 < 0003)
    ('a0000000-0000-4000-a000-000000000001', 'a0000000-0000-4000-a000-000000000003',
     NOW() - INTERVAL '6 days'),

    -- u2 ↔ u6  (0002 < 0006)
    ('a0000000-0000-4000-a000-000000000002', 'a0000000-0000-4000-a000-000000000006',
     NOW() - INTERVAL '11 days')
ON CONFLICT (user_a_id, user_b_id) DO NOTHING;


-- =============================================================================
-- 4. CIRCLES
-- =============================================================================
-- Direct circles (is_direct = true) are auto-created with friendships.
-- The trigger add_circle_owner_as_member auto-inserts the owner into
-- circle_members, so we must NOT insert the owner manually.
-- =============================================================================

-- c1: Direct circle u1↔u2
INSERT INTO circles (id, name, owner_id, is_direct, image_url, created_at)
VALUES ('c0000000-0000-4000-a000-000000000001', NULL,
        'a0000000-0000-4000-a000-000000000001', TRUE, NULL,
        NOW() - INTERVAL '13 days')
ON CONFLICT (id) DO NOTHING;

-- c2: Direct circle u1↔u3
INSERT INTO circles (id, name, owner_id, is_direct, image_url, created_at)
VALUES ('c0000000-0000-4000-a000-000000000002', NULL,
        'a0000000-0000-4000-a000-000000000001', TRUE, NULL,
        NOW() - INTERVAL '6 days')
ON CONFLICT (id) DO NOTHING;

-- c3: Group circle "Famille" owned by u2 (Marie), with image
INSERT INTO circles (id, name, owner_id, is_direct, image_url, created_at)
VALUES ('c0000000-0000-4000-a000-000000000003', 'Famille',
        'a0000000-0000-4000-a000-000000000002', FALSE,
        'https://cdn.offrii.com/circles/demo-famille.jpg',
        NOW() - INTERVAL '10 days')
ON CONFLICT (id) DO NOTHING;

-- c4: Group circle "Amis proches" owned by u1, no image
INSERT INTO circles (id, name, owner_id, is_direct, image_url, created_at)
VALUES ('c0000000-0000-4000-a000-000000000004', 'Amis proches',
        'a0000000-0000-4000-a000-000000000001', FALSE, NULL,
        NOW() - INTERVAL '8 days')
ON CONFLICT (id) DO NOTHING;


-- =============================================================================
-- 5. CIRCLE MEMBERS
-- =============================================================================
-- Owner is auto-inserted by trigger, so we only add non-owner members.
-- Direct circles: add the "other" user.
-- Group circles: add non-owner members.
-- =============================================================================

INSERT INTO circle_members (circle_id, user_id, role, joined_at)
VALUES
    -- c1 (direct u1↔u2): u2 is the non-owner member
    ('c0000000-0000-4000-a000-000000000001', 'a0000000-0000-4000-a000-000000000002',
     'member', NOW() - INTERVAL '13 days'),

    -- c2 (direct u1↔u3): u3 is the non-owner member
    ('c0000000-0000-4000-a000-000000000002', 'a0000000-0000-4000-a000-000000000003',
     'member', NOW() - INTERVAL '6 days'),

    -- c3 (Famille, owner=u2): add u1, u3, u6
    ('c0000000-0000-4000-a000-000000000003', 'a0000000-0000-4000-a000-000000000001',
     'member', NOW() - INTERVAL '10 days'),
    ('c0000000-0000-4000-a000-000000000003', 'a0000000-0000-4000-a000-000000000003',
     'member', NOW() - INTERVAL '9 days'),
    ('c0000000-0000-4000-a000-000000000003', 'a0000000-0000-4000-a000-000000000006',
     'member', NOW() - INTERVAL '9 days'),

    -- c4 (Amis proches, owner=u1): add u2, u4
    ('c0000000-0000-4000-a000-000000000004', 'a0000000-0000-4000-a000-000000000002',
     'member', NOW() - INTERVAL '8 days'),
    ('c0000000-0000-4000-a000-000000000004', 'a0000000-0000-4000-a000-000000000004',
     'member', NOW() - INTERVAL '7 days')
ON CONFLICT (circle_id, user_id) DO NOTHING;


-- =============================================================================
-- 6. CIRCLE SHARE RULES
-- =============================================================================
-- Defines what each user shares with each direct circle friend.
-- share_mode: 'none' | 'all' | 'categories' | 'selection'
-- =============================================================================

INSERT INTO circle_share_rules (circle_id, user_id, share_mode, category_ids, created_at, updated_at)
VALUES
    -- c1: Yassine shares 'all' with Marie
    ('c0000000-0000-4000-a000-000000000001', 'a0000000-0000-4000-a000-000000000001',
     'all', '{}',
     NOW() - INTERVAL '13 days', NOW() - INTERVAL '13 days'),

    -- c1: Marie shares 'categories' (Tech + Mode) with Yassine
    ('c0000000-0000-4000-a000-000000000001', 'a0000000-0000-4000-a000-000000000002',
     'categories',
     (SELECT ARRAY[id] FROM _cat WHERE key = 'tech') || (SELECT ARRAY[id] FROM _cat WHERE key = 'mode'),
     NOW() - INTERVAL '12 days', NOW() - INTERVAL '12 days'),

    -- c2: Yassine shares 'selection' with Lucas (items managed via circle_items)
    ('c0000000-0000-4000-a000-000000000002', 'a0000000-0000-4000-a000-000000000001',
     'selection', '{}',
     NOW() - INTERVAL '6 days', NOW() - INTERVAL '6 days'),

    -- c2: Lucas shares 'none' (hasn't configured sharing yet)
    ('c0000000-0000-4000-a000-000000000002', 'a0000000-0000-4000-a000-000000000003',
     'none', '{}',
     NOW() - INTERVAL '6 days', NOW() - INTERVAL '6 days')
ON CONFLICT (circle_id, user_id) DO NOTHING;


-- =============================================================================
-- 7. CIRCLE ITEMS (items shared to circles)
-- =============================================================================

INSERT INTO circle_items (circle_id, item_id, shared_by, shared_at)
VALUES
    -- Yassine shares b01 (MacBook) and b06 (Lampe) to direct circle with Lucas (c2)
    ('c0000000-0000-4000-a000-000000000002', 'b0000000-0000-4000-a000-000000000001',
     'a0000000-0000-4000-a000-000000000001', NOW() - INTERVAL '5 days'),
    ('c0000000-0000-4000-a000-000000000002', 'b0000000-0000-4000-a000-000000000006',
     'a0000000-0000-4000-a000-000000000001', NOW() - INTERVAL '4 days'),

    -- Yassine shares b01 (MacBook) to group circle "Amis proches" (c4)
    ('c0000000-0000-4000-a000-000000000004', 'b0000000-0000-4000-a000-000000000001',
     'a0000000-0000-4000-a000-000000000001', NOW() - INTERVAL '7 days'),

    -- Marie shares b10 (Sac Longchamp) to group circle "Famille" (c3)
    ('c0000000-0000-4000-a000-000000000003', 'b0000000-0000-4000-a000-000000000010',
     'a0000000-0000-4000-a000-000000000002', NOW() - INTERVAL '9 days')
ON CONFLICT (circle_id, item_id) DO NOTHING;


-- =============================================================================
-- 8. CIRCLE EVENTS
-- =============================================================================

INSERT INTO circle_events (id, circle_id, actor_id, event_type, target_item_id, target_user_id, created_at)
VALUES
    -- u2 joined c3 (Famille) — auto event
    ('ca000000-0000-4000-a000-000000000001',
     'c0000000-0000-4000-a000-000000000003', 'a0000000-0000-4000-a000-000000000002',
     'member_joined', NULL, 'a0000000-0000-4000-a000-000000000002',
     NOW() - INTERVAL '10 days'),

    -- u1 joined c3 (Famille)
    ('ca000000-0000-4000-a000-000000000002',
     'c0000000-0000-4000-a000-000000000003', 'a0000000-0000-4000-a000-000000000001',
     'member_joined', NULL, 'a0000000-0000-4000-a000-000000000001',
     NOW() - INTERVAL '10 days'),

    -- Marie shared sac Longchamp to Famille
    ('ca000000-0000-4000-a000-000000000003',
     'c0000000-0000-4000-a000-000000000003', 'a0000000-0000-4000-a000-000000000002',
     'item_shared', 'b0000000-0000-4000-a000-000000000010', NULL,
     NOW() - INTERVAL '9 days'),

    -- Yassine shared MacBook to c2
    ('ca000000-0000-4000-a000-000000000004',
     'c0000000-0000-4000-a000-000000000002', 'a0000000-0000-4000-a000-000000000001',
     'item_shared', 'b0000000-0000-4000-a000-000000000001', NULL,
     NOW() - INTERVAL '5 days'),

    -- Yassine claimed Marie's bougie in c1
    ('ca000000-0000-4000-a000-000000000005',
     'c0000000-0000-4000-a000-000000000001', 'a0000000-0000-4000-a000-000000000001',
     'item_claimed', 'b0000000-0000-4000-a000-000000000011', NULL,
     NOW() - INTERVAL '2 days'),

    -- Marie received Sony headphones from Yassine
    ('ca000000-0000-4000-a000-000000000006',
     'c0000000-0000-4000-a000-000000000001', 'a0000000-0000-4000-a000-000000000002',
     'item_received', 'b0000000-0000-4000-a000-000000000004', NULL,
     NOW() - INTERVAL '3 days')
ON CONFLICT (id) DO NOTHING;


-- =============================================================================
-- 9. CIRCLE INVITES
-- =============================================================================

INSERT INTO circle_invites (id, circle_id, token, created_by, expires_at, max_uses, use_count, created_at)
VALUES
    -- Active invite to Famille (c3), max 5 uses, 2 used
    ('c1000000-0000-4000-a000-000000000001',
     'c0000000-0000-4000-a000-000000000003', 'inv_famille_abc123def456',
     'a0000000-0000-4000-a000-000000000002',
     NOW() + INTERVAL '7 days', 5, 2,
     NOW() - INTERVAL '3 days'),

    -- Expired invite to Amis proches (c4)
    ('c1000000-0000-4000-a000-000000000002',
     'c0000000-0000-4000-a000-000000000004', 'inv_amis_expired_xyz789',
     'a0000000-0000-4000-a000-000000000001',
     NOW() - INTERVAL '1 day', 1, 0,
     NOW() - INTERVAL '8 days')
ON CONFLICT (id) DO NOTHING;


-- =============================================================================
-- 10. SHARE LINKS  (4 links)
-- =============================================================================
-- Covers: all scopes (all, category, selection), permissions, expired, deactivated
-- =============================================================================

INSERT INTO share_links (id, user_id, token, label, permissions, scope, scope_data, is_active, expires_at, created_at)
VALUES
    -- e1: Yassine's "all items" link, view_and_claim, active, no expiry
    ('e0000000-0000-4000-a000-000000000001',
     'a0000000-0000-4000-a000-000000000001',
     'sl_yassine_all_abc123456789', 'Ma liste complete',
     'view_and_claim', 'all', NULL, TRUE, NULL,
     NOW() - INTERVAL '20 days'),

    -- e2: Marie's category link (Tech only), view_only, active
    ('e0000000-0000-4000-a000-000000000002',
     'a0000000-0000-4000-a000-000000000002',
     'sl_marie_tech_def456789012', 'Idees tech',
     'view_only', 'category',
     (SELECT jsonb_build_object('category_ids', jsonb_build_array(id::text)) FROM _cat WHERE key = 'tech'),
     TRUE, NOW() + INTERVAL '30 days',
     NOW() - INTERVAL '10 days'),

    -- e3: Yassine's selection link (specific items), view_and_claim, active
    ('e0000000-0000-4000-a000-000000000003',
     'a0000000-0000-4000-a000-000000000001',
     'sl_yassine_sel_ghi789012345', 'Pour Noel',
     'view_and_claim', 'selection',
     '{"item_ids": ["b0000000-0000-4000-a000-000000000001", "b0000000-0000-4000-a000-000000000006"]}',
     TRUE, NULL,
     NOW() - INTERVAL '5 days'),

    -- e4: Yassine's expired and deactivated link
    ('e0000000-0000-4000-a000-000000000004',
     'a0000000-0000-4000-a000-000000000001',
     'sl_yassine_old_jkl012345678', 'Ancien lien',
     'view_only', 'all', NULL, FALSE,
     NOW() - INTERVAL '10 days',
     NOW() - INTERVAL '25 days')
ON CONFLICT (id) DO NOTHING;


-- Back-fill claimed_via_link_id on web-claimed item (share_links now exist)
UPDATE items SET claimed_via_link_id = 'e0000000-0000-4000-a000-000000000001'
WHERE id = 'b0000000-0000-4000-a000-000000000006' AND claimed_via_link_id IS NULL;

-- =============================================================================
-- 11. COMMUNITY WISHES  (8 wishes)
-- =============================================================================
-- Covers all statuses: pending, open, matched, fulfilled, closed, rejected, flagged
-- Covers categories: education, clothing, health, home, children, other
-- Covers: anonymous, with image, with links, reported, at active limit
-- =============================================================================

INSERT INTO community_wishes (id, owner_id, title, description, category, status,
                              is_anonymous, matched_with, matched_at, fulfilled_at,
                              closed_at, report_count, reopen_count, last_reopen_at,
                              moderation_note, image_url, links,
                              og_image_url, og_title, og_site_name,
                              created_at, updated_at)
VALUES
    -- d1: PENDING — awaiting moderation
    ('d0000000-0000-4000-a000-000000000001',
     'a0000000-0000-4000-a000-000000000003', -- Lucas
     'Manuels scolaires pour le lycee', 'J ai besoin de manuels pour la rentree de septembre',
     'education', 'pending',
     FALSE, NULL, NULL, NULL, NULL, 0, 0, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '1 day', NOW() - INTERVAL '1 day'),

    -- d2: OPEN — available for matching
    ('d0000000-0000-4000-a000-000000000002',
     'a0000000-0000-4000-a000-000000000007', -- New User
     'Vetements chauds pour l hiver', 'Taille M, manteau et echarpe',
     'clothing', 'open',
     FALSE, NULL, NULL, NULL, NULL, 0, 0, NULL, NULL,
     'https://cdn.offrii.com/wishes/demo-vetements.jpg', NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '3 days', NOW() - INTERVAL '3 days'),

    -- d3: MATCHED — someone offered to help
    ('d0000000-0000-4000-a000-000000000003',
     'a0000000-0000-4000-a000-000000000003', -- Lucas
     'Medicaments pour allergie', 'Antihistaminiques en pharmacie',
     'health', 'matched',
     TRUE, -- anonymous
     'a0000000-0000-4000-a000-000000000002', -- matched with Marie
     NOW() - INTERVAL '1 day', NULL, NULL, 0, 0, NULL, NULL,
     NULL, ARRAY['https://www.doctissimo.fr/allergie'], NULL, NULL, NULL,
     NOW() - INTERVAL '5 days', NOW() - INTERVAL '1 day'),

    -- d4: FULFILLED — wish has been fulfilled
    ('d0000000-0000-4000-a000-000000000004',
     'a0000000-0000-4000-a000-000000000006', -- Camille
     'Articles de cuisine pour premier appartement', 'Casseroles, poeles, ustensiles de base',
     'home', 'fulfilled',
     FALSE,
     'a0000000-0000-4000-a000-000000000001', -- matched with Yassine
     NOW() - INTERVAL '7 days', NOW() - INTERVAL '2 days', NULL, 0, 0, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '10 days', NOW() - INTERVAL '2 days'),

    -- d5: CLOSED — closed by owner
    ('d0000000-0000-4000-a000-000000000005',
     'a0000000-0000-4000-a000-000000000003', -- Lucas
     'Jouets pour ma petite soeur', 'Elle a 5 ans, aime les puzzles',
     'children', 'closed',
     FALSE, NULL, NULL, NULL, NOW() - INTERVAL '4 days', 0, 0, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '12 days', NOW() - INTERVAL '4 days'),

    -- d6: REJECTED — rejected by moderation
    ('d0000000-0000-4000-a000-000000000006',
     'a0000000-0000-4000-a000-000000000008', -- Reporter (testing as wish creator too)
     'Demande non conforme', 'Contenu problematique',
     'other', 'rejected',
     FALSE, NULL, NULL, NULL, NULL, 0, 0, NULL,
     'Wish does not meet community guidelines',
     NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '8 days', NOW() - INTERVAL '7 days'),

    -- d7: FLAGGED — auto-flagged from reports
    ('d0000000-0000-4000-a000-000000000007',
     'a0000000-0000-4000-a000-000000000007', -- New User
     'Demande signale', 'Ce souhait a ete signale par la communaute',
     'other', 'flagged',
     FALSE, NULL, NULL, NULL, NULL, 3, 0, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL,
     NOW() - INTERVAL '6 days', NOW() - INTERVAL '2 days'),

    -- d8: OPEN — reopened wish (reopen_count > 0)
    ('d0000000-0000-4000-a000-000000000008',
     'a0000000-0000-4000-a000-000000000006', -- Camille
     'Fournitures scolaires pour la rentree', 'Cahiers, stylos, trousse',
     'education', 'open',
     TRUE, -- anonymous
     NULL, NULL, NULL, NULL, 0, 1, NOW() - INTERVAL '1 day', NULL,
     NULL, ARRAY['https://www.amazon.fr/fournitures-scolaires'],
     NULL, NULL, NULL,
     NOW() - INTERVAL '14 days', NOW() - INTERVAL '1 day')
ON CONFLICT (id) DO NOTHING;


-- =============================================================================
-- 12. WISH MESSAGES
-- =============================================================================

INSERT INTO wish_messages (id, wish_id, sender_id, body, created_at)
VALUES
    -- Messages on the matched wish (d3: Lucas ↔ Marie)
    ('aa000000-0000-4000-a000-000000000001',
     'd0000000-0000-4000-a000-000000000003', 'a0000000-0000-4000-a000-000000000002',
     'Bonjour, je peux vous aider avec les medicaments. Quelle marque preferez-vous?',
     NOW() - INTERVAL '1 day'),

    ('aa000000-0000-4000-a000-000000000002',
     'd0000000-0000-4000-a000-000000000003', 'a0000000-0000-4000-a000-000000000003',
     'Merci beaucoup! Cetirizine si possible.',
     NOW() - INTERVAL '23 hours'),

    -- Messages on the fulfilled wish (d4: Camille ↔ Yassine)
    ('aa000000-0000-4000-a000-000000000003',
     'd0000000-0000-4000-a000-000000000004', 'a0000000-0000-4000-a000-000000000001',
     'J ai un set de cuisine complet a donner, ca vous interesse?',
     NOW() - INTERVAL '6 days'),

    ('aa000000-0000-4000-a000-000000000004',
     'd0000000-0000-4000-a000-000000000004', 'a0000000-0000-4000-a000-000000000006',
     'Oui, c est parfait! Merci enormement!',
     NOW() - INTERVAL '6 days')
ON CONFLICT (id) DO NOTHING;


-- =============================================================================
-- 13. WISH REPORTS
-- =============================================================================

INSERT INTO wish_reports (id, wish_id, reporter_id, reason, details, created_at)
VALUES
    -- Reports on the flagged wish (d7)
    ('ab000000-0000-4000-a000-000000000001',
     'd0000000-0000-4000-a000-000000000007', 'a0000000-0000-4000-a000-000000000008',
     'inappropriate', 'Le contenu semble inapproprie pour la plateforme',
     NOW() - INTERVAL '3 days'),

    ('ab000000-0000-4000-a000-000000000002',
     'd0000000-0000-4000-a000-000000000007', 'a0000000-0000-4000-a000-000000000001',
     'spam', NULL,
     NOW() - INTERVAL '2 days'),

    ('ab000000-0000-4000-a000-000000000003',
     'd0000000-0000-4000-a000-000000000007', 'a0000000-0000-4000-a000-000000000002',
     'other', 'Pas clair ce qui est demande',
     NOW() - INTERVAL '2 days')
ON CONFLICT (wish_id, reporter_id) DO NOTHING;


-- =============================================================================
-- 14. WISH BLOCKS
-- =============================================================================

INSERT INTO wish_blocks (id, wish_id, user_id, created_at)
VALUES
    -- u8 (Reporter) blocks the flagged wish d7
    ('ac000000-0000-4000-a000-000000000001',
     'd0000000-0000-4000-a000-000000000007', 'a0000000-0000-4000-a000-000000000008',
     NOW() - INTERVAL '3 days'),

    -- u1 (Yassine) blocks the rejected wish d6
    ('ac000000-0000-4000-a000-000000000002',
     'd0000000-0000-4000-a000-000000000006', 'a0000000-0000-4000-a000-000000000001',
     NOW() - INTERVAL '7 days')
ON CONFLICT (wish_id, user_id) DO NOTHING;


-- =============================================================================
-- 15. NOTIFICATIONS
-- =============================================================================
-- Types observed in codebase: friend_request, friend_accepted, item_shared,
--   item_claimed, circle_activity, circle_member_joined, circle_added,
--   wish_message, wish_moderation_approved, wish_moderation_flagged,
--   wish_closed, wish_offer, wish_offer_withdrawn, wish_offer_rejected,
--   wish_confirmed, wish_reported, wish_approved, wish_rejected
-- =============================================================================

INSERT INTO notifications (id, user_id, type, title, body, read, circle_id, item_id, wish_id, actor_id, created_at)
VALUES
    -- Friend request notification (u1 receives from u4)
    ('bb000000-0000-4000-a000-000000000001',
     'a0000000-0000-4000-a000-000000000001',
     'friend_request', 'Nouvelle demande d ami',
     'Sophie Martin vous a envoye une demande d ami',
     FALSE, NULL, NULL, NULL, 'a0000000-0000-4000-a000-000000000004',
     NOW() - INTERVAL '2 days'),

    -- Friend accepted notification (u2 receives when u1 accepted)
    ('bb000000-0000-4000-a000-000000000002',
     'a0000000-0000-4000-a000-000000000002',
     'friend_accepted', 'Demande acceptee',
     'Yassine a accepte votre demande d ami',
     TRUE, NULL, NULL, NULL, 'a0000000-0000-4000-a000-000000000001',
     NOW() - INTERVAL '13 days'),

    -- Item claimed notification (u1 receives: Marie claimed Sony headphones)
    ('bb000000-0000-4000-a000-000000000003',
     'a0000000-0000-4000-a000-000000000001',
     'item_claimed', 'Souhait reserve',
     'Marie Dupont a reserve "Casque Sony WH-1000XM5"',
     FALSE,
     'c0000000-0000-4000-a000-000000000001',
     'b0000000-0000-4000-a000-000000000004',
     NULL,
     'a0000000-0000-4000-a000-000000000002',
     NOW() - INTERVAL '3 days'),

    -- Item shared notification (u2 receives: Yassine shared MacBook to circle)
    ('bb000000-0000-4000-a000-000000000004',
     'a0000000-0000-4000-a000-000000000002',
     'item_shared', 'Nouvel article partage',
     'Yassine a partage "MacBook Pro M4" dans Amis proches',
     TRUE,
     'c0000000-0000-4000-a000-000000000004',
     'b0000000-0000-4000-a000-000000000001',
     NULL,
     'a0000000-0000-4000-a000-000000000001',
     NOW() - INTERVAL '7 days'),

    -- Circle member joined (u2 receives: u4 joined Amis proches)
    ('bb000000-0000-4000-a000-000000000005',
     'a0000000-0000-4000-a000-000000000002',
     'circle_member_joined', 'Nouveau membre',
     'Sophie Martin a rejoint Amis proches',
     TRUE,
     'c0000000-0000-4000-a000-000000000004',
     NULL, NULL,
     'a0000000-0000-4000-a000-000000000004',
     NOW() - INTERVAL '7 days'),

    -- Wish message notification (Lucas receives message from Marie on d3)
    ('bb000000-0000-4000-a000-000000000006',
     'a0000000-0000-4000-a000-000000000003',
     'wish_message', 'Nouveau message',
     'Vous avez recu un nouveau message concernant votre souhait',
     FALSE, NULL, NULL,
     'd0000000-0000-4000-a000-000000000003',
     'a0000000-0000-4000-a000-000000000002',
     NOW() - INTERVAL '1 day'),

    -- Wish offer notification (Lucas receives: Marie offered on d3)
    ('bb000000-0000-4000-a000-000000000007',
     'a0000000-0000-4000-a000-000000000003',
     'wish_offer', 'Nouvelle proposition d aide',
     'Quelqu un souhaite vous aider avec "Medicaments pour allergie"',
     TRUE, NULL, NULL,
     'd0000000-0000-4000-a000-000000000003',
     'a0000000-0000-4000-a000-000000000002',
     NOW() - INTERVAL '1 day'),

    -- Wish confirmed notification (Yassine receives: Camille confirmed d4)
    ('bb000000-0000-4000-a000-000000000008',
     'a0000000-0000-4000-a000-000000000001',
     'wish_confirmed', 'Souhait confirme',
     'Camille R. a confirme la reception de votre aide',
     TRUE, NULL, NULL,
     'd0000000-0000-4000-a000-000000000004',
     'a0000000-0000-4000-a000-000000000006',
     NOW() - INTERVAL '2 days'),

    -- Wish moderation flagged (u7 receives: d7 was flagged)
    ('bb000000-0000-4000-a000-000000000009',
     'a0000000-0000-4000-a000-000000000007',
     'wish_moderation_flagged', 'Souhait signale',
     'Votre souhait a ete signale et est en cours de revision',
     FALSE, NULL, NULL,
     'd0000000-0000-4000-a000-000000000007',
     NULL,
     NOW() - INTERVAL '2 days'),

    -- Wish rejected notification (u8 receives: d6 was rejected)
    ('bb000000-0000-4000-a000-000000000010',
     'a0000000-0000-4000-a000-000000000008',
     'wish_rejected', 'Souhait refuse',
     'Votre souhait ne respecte pas les regles de la communaute',
     TRUE, NULL, NULL,
     'd0000000-0000-4000-a000-000000000006',
     NULL,
     NOW() - INTERVAL '7 days'),

    -- Web claim notification (u1 receives: someone claimed Zelda via web)
    ('bb000000-0000-4000-a000-000000000011',
     'a0000000-0000-4000-a000-000000000001',
     'item_claimed', 'Souhait reserve depuis le web',
     'Maman a reserve "Zelda Tears of the Kingdom" via votre lien de partage',
     FALSE, NULL,
     'b0000000-0000-4000-a000-000000000007',
     NULL, NULL,
     NOW() - INTERVAL '1 day')
ON CONFLICT (id) DO NOTHING;


-- =============================================================================
-- 16. PUSH TOKENS
-- =============================================================================

INSERT INTO push_tokens (id, user_id, token, platform, created_at)
VALUES
    -- Yassine: iOS device
    ('ad000000-0000-4000-a000-000000000001',
     'a0000000-0000-4000-a000-000000000001',
     'apns_demo_token_yassine_iphone_abc123', 'ios',
     NOW() - INTERVAL '28 days'),

    -- Yassine: second iOS device (iPad)
    ('ad000000-0000-4000-a000-000000000002',
     'a0000000-0000-4000-a000-000000000001',
     'apns_demo_token_yassine_ipad_def456', 'ios',
     NOW() - INTERVAL '15 days'),

    -- Marie: iOS device
    ('ad000000-0000-4000-a000-000000000003',
     'a0000000-0000-4000-a000-000000000002',
     'apns_demo_token_marie_iphone_ghi789', 'ios',
     NOW() - INTERVAL '12 days'),

    -- Lucas: Android device
    ('ad000000-0000-4000-a000-000000000004',
     'a0000000-0000-4000-a000-000000000003',
     'fcm_demo_token_lucas_android_jkl012', 'android',
     NOW() - INTERVAL '6 days')
ON CONFLICT (user_id, token) DO NOTHING;


-- =============================================================================
-- 17. REFRESH TOKENS
-- =============================================================================

INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, revoked_at, created_at)
VALUES
    -- Active token for Yassine
    ('ae000000-0000-4000-a000-000000000001',
     'a0000000-0000-4000-a000-000000000001',
     'sha256_demo_active_yassine_abc123def456ghi789',
     NOW() + INTERVAL '30 days', NULL,
     NOW() - INTERVAL '1 day'),

    -- Revoked token for Yassine (old session)
    ('ae000000-0000-4000-a000-000000000002',
     'a0000000-0000-4000-a000-000000000001',
     'sha256_demo_revoked_yassine_jkl012mno345pqr678',
     NOW() + INTERVAL '15 days', NOW() - INTERVAL '5 days',
     NOW() - INTERVAL '20 days'),

    -- Active token for Marie
    ('ae000000-0000-4000-a000-000000000003',
     'a0000000-0000-4000-a000-000000000002',
     'sha256_demo_active_marie_stu901vwx234yz567',
     NOW() + INTERVAL '30 days', NULL,
     NOW() - INTERVAL '2 days'),

    -- Expired token for Lucas
    ('ae000000-0000-4000-a000-000000000004',
     'a0000000-0000-4000-a000-000000000003',
     'sha256_demo_expired_lucas_abc789def012ghi345',
     NOW() - INTERVAL '2 days', NULL,
     NOW() - INTERVAL '32 days')
ON CONFLICT (id) DO NOTHING;


-- =============================================================================
-- 18. EMAIL VERIFICATION TOKENS
-- =============================================================================

INSERT INTO email_verification_tokens (id, user_id, token, expires_at, created_at)
VALUES
    -- Active token for Lucas (unverified user)
    ('af000000-0000-4000-a000-000000000001',
     'a0000000-0000-4000-a000-000000000003',
     'verify_lucas_demo_token_abc123def456ghi789jkl012mno345pqr678st',
     NOW() + INTERVAL '24 hours',
     NOW() - INTERVAL '1 hour'),

    -- Expired token for Yassine (already verified, token expired)
    ('af000000-0000-4000-a000-000000000002',
     'a0000000-0000-4000-a000-000000000001',
     'verify_yassine_expired_uvw456xyz789abc012def345ghi678jkl901mn',
     NOW() - INTERVAL '29 days',
     NOW() - INTERVAL '30 days')
ON CONFLICT (id) DO NOTHING;


-- =============================================================================
-- CLEANUP temp table
-- =============================================================================
DROP TABLE IF EXISTS _cat;

COMMIT;
