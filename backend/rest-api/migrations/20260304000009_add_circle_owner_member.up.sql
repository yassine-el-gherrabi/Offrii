CREATE OR REPLACE FUNCTION add_circle_owner_as_member()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO circle_members (circle_id, user_id, role)
    VALUES (NEW.id, NEW.owner_id, 'owner');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_circles_add_owner_member
    AFTER INSERT ON circles
    FOR EACH ROW EXECUTE FUNCTION add_circle_owner_as_member();
