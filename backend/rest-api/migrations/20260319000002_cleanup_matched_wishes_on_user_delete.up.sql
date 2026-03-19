-- When a user is deleted, reset any wishes they were matched to back to 'open'
CREATE OR REPLACE FUNCTION cleanup_matched_wishes_on_user_delete()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE community_wishes
    SET status = 'open', matched_with = NULL, matched_at = NULL
    WHERE matched_with = OLD.id AND status = 'matched';
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_cleanup_matched_wishes
BEFORE DELETE ON users
FOR EACH ROW
EXECUTE FUNCTION cleanup_matched_wishes_on_user_delete();
