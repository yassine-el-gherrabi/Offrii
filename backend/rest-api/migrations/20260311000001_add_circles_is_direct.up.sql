ALTER TABLE circles ADD COLUMN is_direct BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE circles ALTER COLUMN name DROP NOT NULL;

CREATE OR REPLACE FUNCTION fn_check_direct_circle_member_limit()
RETURNS TRIGGER AS $$
DECLARE v_is_direct BOOLEAN; v_count INTEGER;
BEGIN
    SELECT is_direct INTO v_is_direct FROM circles WHERE id = NEW.circle_id;
    IF v_is_direct THEN
        SELECT COUNT(*) INTO v_count FROM circle_members WHERE circle_id = NEW.circle_id;
        IF v_count >= 2 THEN RAISE EXCEPTION 'direct circles cannot have more than 2 members'; END IF;
    END IF;
    RETURN NEW;
END; $$ LANGUAGE plpgsql;

CREATE TRIGGER trg_check_direct_circle_member_limit
    BEFORE INSERT ON circle_members
    FOR EACH ROW EXECUTE FUNCTION fn_check_direct_circle_member_limit();
