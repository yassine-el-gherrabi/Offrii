CREATE INDEX idx_items_category_id ON items(category_id);

CREATE OR REPLACE FUNCTION set_purchased_at()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF NEW.status = 'purchased' AND NEW.purchased_at IS NULL THEN
            NEW.purchased_at = NOW();
        END IF;
        RETURN NEW;
    END IF;

    IF NEW.status = 'purchased'
       AND (OLD.status IS DISTINCT FROM 'purchased')
       AND NEW.purchased_at IS NULL
    THEN
        NEW.purchased_at = NOW();
    END IF;

    IF NEW.status != 'purchased' AND OLD.status = 'purchased' THEN
        NEW.purchased_at = NULL;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_items_set_purchased_at
    BEFORE INSERT OR UPDATE ON items
    FOR EACH ROW EXECUTE FUNCTION set_purchased_at();
