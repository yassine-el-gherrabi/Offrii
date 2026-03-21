-- Drop reminder-related indexes
DROP INDEX IF EXISTS idx_users_utc_reminder_hour;

-- Drop reminder columns (feature abandoned)
ALTER TABLE users
    DROP COLUMN IF EXISTS reminder_freq,
    DROP COLUMN IF EXISTS reminder_time,
    DROP COLUMN IF EXISTS utc_reminder_hour,
    DROP COLUMN IF EXISTS timezone,
    DROP COLUMN IF EXISTS locale;
