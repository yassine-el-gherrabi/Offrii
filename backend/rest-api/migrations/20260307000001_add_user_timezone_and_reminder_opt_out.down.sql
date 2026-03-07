DROP INDEX IF EXISTS idx_users_utc_reminder_hour;

ALTER TABLE users DROP CONSTRAINT users_reminder_freq_check;
ALTER TABLE users ADD CONSTRAINT users_reminder_freq_check
    CHECK (reminder_freq IN ('daily', 'weekly', 'monthly'));

ALTER TABLE users
    DROP COLUMN locale,
    DROP COLUMN utc_reminder_hour,
    DROP COLUMN timezone;
