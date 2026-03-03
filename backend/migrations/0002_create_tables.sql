-- Migration 0002: Create tables
-- 17 tables in FK dependency order

-- 1. users
CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email         VARCHAR(255) NOT NULL UNIQUE,
    pseudo        VARCHAR(50)  NOT NULL UNIQUE,
    password_hash TEXT         NOT NULL,
    avatar_url    TEXT,
    birthday      DATE,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- 2. items
CREATE TABLE items (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name         VARCHAR(255) NOT NULL,
    budget       NUMERIC(10, 2),
    priority     priority     NOT NULL DEFAULT 'Envie',
    category     VARCHAR(100),
    notes        TEXT,
    image_url    TEXT,
    status       item_status  NOT NULL DEFAULT 'Active',
    purchased_at TIMESTAMPTZ,
    actual_price NUMERIC(10, 2),
    deleted_at   TIMESTAMPTZ,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- 3. item_links
CREATE TABLE item_links (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id    UUID         NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    url        TEXT         NOT NULL,
    title      VARCHAR(255),
    price      NUMERIC(10, 2),
    created_at TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- 4. circles
CREATE TABLE circles (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_by UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name       VARCHAR(100) NOT NULL,
    emoji      VARCHAR(10),
    created_at TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- 5. circle_members
CREATE TABLE circle_members (
    id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    circle_id UUID        NOT NULL REFERENCES circles(id) ON DELETE CASCADE,
    user_id   UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role      member_role NOT NULL DEFAULT 'Member',
    joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (circle_id, user_id)
);

-- 6. circle_invitations
CREATE TABLE circle_invitations (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    circle_id  UUID         NOT NULL REFERENCES circles(id) ON DELETE CASCADE,
    invited_by UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token      VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ  NOT NULL,
    used_by    UUID         REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- 7. item_visibility
CREATE TABLE item_visibility (
    id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id   UUID        NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    circle_id UUID        NOT NULL REFERENCES circles(id) ON DELETE CASCADE,
    shared_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (item_id, circle_id)
);

-- 8. reservations
CREATE TABLE reservations (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id     UUID               NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    reserved_by UUID               NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    circle_id   UUID               NOT NULL REFERENCES circles(id) ON DELETE CASCADE,
    status      reservation_status NOT NULL DEFAULT 'Reserved',
    created_at  TIMESTAMPTZ        NOT NULL DEFAULT now()
);

-- 9. thematic_lists
CREATE TABLE thematic_lists (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name       VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- 10. thematic_list_items
CREATE TABLE thematic_list_items (
    id       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    list_id  UUID NOT NULL REFERENCES thematic_lists(id) ON DELETE CASCADE,
    item_id  UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    position INT  NOT NULL,

    UNIQUE (list_id, item_id)
);

-- 11. list_visibility
CREATE TABLE list_visibility (
    id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    list_id   UUID        NOT NULL REFERENCES thematic_lists(id) ON DELETE CASCADE,
    circle_id UUID        NOT NULL REFERENCES circles(id) ON DELETE CASCADE,
    shared_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (list_id, circle_id)
);

-- 12. community_wishes
CREATE TABLE community_wishes (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title       VARCHAR(255) NOT NULL,
    description TEXT,
    category    VARCHAR(100),
    link        TEXT,
    status      wish_status  NOT NULL DEFAULT 'Pending',
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- 13. wish_offers
CREATE TABLE wish_offers (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wish_id    UUID         NOT NULL REFERENCES community_wishes(id) ON DELETE CASCADE,
    offered_by UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status     offer_status NOT NULL DEFAULT 'Pending',
    created_at TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- 14. conversations
CREATE TABLE conversations (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wish_offer_id UUID        NOT NULL UNIQUE REFERENCES wish_offers(id) ON DELETE CASCADE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 15. messages
CREATE TABLE messages (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID        NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    sender_id       UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content         TEXT        NOT NULL,
    read_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 16. notifications
CREATE TABLE notifications (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type       VARCHAR(50)  NOT NULL,
    title      VARCHAR(255) NOT NULL,
    body       TEXT,
    read       BOOLEAN      NOT NULL DEFAULT false,
    data       JSONB,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- 17. reports
CREATE TABLE reports (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reported_by  UUID          NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_type  report_target NOT NULL,
    target_id    UUID          NOT NULL,
    reason       TEXT          NOT NULL,
    status       report_status NOT NULL DEFAULT 'Pending',
    admin_notes  TEXT,
    reviewed_at  TIMESTAMPTZ,
    reviewed_by  UUID          REFERENCES users(id) ON DELETE SET NULL,
    created_at   TIMESTAMPTZ   NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ   NOT NULL DEFAULT now()
);
