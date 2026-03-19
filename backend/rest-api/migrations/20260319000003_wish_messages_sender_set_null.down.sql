ALTER TABLE wish_messages
    DROP CONSTRAINT IF EXISTS wish_messages_sender_id_fkey;

ALTER TABLE wish_messages
    ADD CONSTRAINT wish_messages_sender_id_fkey
    FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE;

-- Cannot re-add NOT NULL if rows with NULL exist
