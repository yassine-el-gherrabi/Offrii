DROP TABLE IF EXISTS email_change_tokens;
DROP TABLE IF EXISTS email_verification_tokens;
DROP TABLE IF EXISTS connection_logs;
DROP TRIGGER IF EXISTS trg_users_updated_at ON users;
DROP FUNCTION IF EXISTS cleanup_matched_wishes_on_user_delete();
DROP TABLE IF EXISTS users;
DROP FUNCTION IF EXISTS set_updated_at();
