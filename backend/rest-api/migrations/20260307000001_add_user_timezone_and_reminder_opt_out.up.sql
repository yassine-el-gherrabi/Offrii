ALTER TABLE users
    ADD COLUMN timezone TEXT NOT NULL DEFAULT 'UTC',
    ADD COLUMN utc_reminder_hour SMALLINT NOT NULL DEFAULT 10,
    ADD COLUMN locale VARCHAR(10) NOT NULL DEFAULT 'fr';

ALTER TABLE users DROP CONSTRAINT users_reminder_freq_check;
ALTER TABLE users ADD CONSTRAINT users_reminder_freq_check
    CHECK (reminder_freq IN ('never', 'daily', 'weekly', 'monthly'));

CREATE INDEX idx_users_utc_reminder_hour
    ON users(utc_reminder_hour) WHERE reminder_freq != 'never';
