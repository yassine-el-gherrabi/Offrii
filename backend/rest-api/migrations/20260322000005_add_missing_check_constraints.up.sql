-- Add CHECK constraints that were validated only in application code

-- notifications.type — all notification types used in the codebase
ALTER TABLE notifications
    ADD CONSTRAINT chk_notifications_type CHECK (
        type IN (
            'friend_request', 'friend_accepted',
            'circle_activity', 'circle_member_joined',
            'item_shared', 'item_claimed', 'item_unclaimed', 'item_received',
            'wish_moderation_approved', 'wish_moderation_flagged',
            'wish_offer', 'wish_offer_withdrawn', 'wish_closed',
            'wish_approved', 'wish_rejected', 'wish_confirmed',
            'wish_message'
        )
    );

-- circle_share_rules.share_mode
ALTER TABLE circle_share_rules
    ADD CONSTRAINT chk_share_mode CHECK (
        share_mode IN ('none', 'all', 'categories', 'selection')
    );

-- items.claimed_via
ALTER TABLE items
    ADD CONSTRAINT chk_claimed_via CHECK (
        claimed_via IS NULL OR claimed_via IN ('app', 'web')
    );
