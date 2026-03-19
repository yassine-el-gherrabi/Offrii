-- Change sender_id from ON DELETE CASCADE to ON DELETE SET NULL
-- so messages are preserved when a user account is deleted
ALTER TABLE wish_messages
    ALTER COLUMN sender_id DROP NOT NULL;

ALTER TABLE wish_messages
    DROP CONSTRAINT IF EXISTS wish_messages_sender_id_fkey;

ALTER TABLE wish_messages
    ADD CONSTRAINT wish_messages_sender_id_fkey
    FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE SET NULL;
