ALTER TABLE circle_events DROP CONSTRAINT IF EXISTS circle_events_event_type_check;
ALTER TABLE circle_events ADD CONSTRAINT circle_events_event_type_check
    CHECK (event_type IN (
        'item_shared', 'item_unshared', 'item_claimed',
        'item_unclaimed', 'member_joined', 'member_left',
        'item_received', 'share_rule_set', 'share_rule_removed'
    ));
