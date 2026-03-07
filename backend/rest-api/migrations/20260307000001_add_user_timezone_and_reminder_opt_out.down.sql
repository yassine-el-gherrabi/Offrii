DROP INDEX IF EXISTS idx_users_utc_reminder_hour;

ALTER TABLE users DROP CONSTRAINT users_reminder_freq_check;

-- Reset 'never' rows to the legacy default before reinstating the old constraint
UPDATE users SET reminder_freq = 'weekly' WHERE reminder_freq = 'never';

ALTER TABLE users ADD CONSTRAINT users_reminder_freq_check
    CHECK (reminder_freq IN ('daily', 'weekly', 'monthly'));

ALTER TABLE users
    DROP COLUMN locale,
    DROP COLUMN utc_reminder_hour,
    DROP COLUMN timezone;
