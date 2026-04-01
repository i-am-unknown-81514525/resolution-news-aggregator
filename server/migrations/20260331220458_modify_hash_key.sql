-- Add migration script here
CREATE EXTENSION IF NOT EXISTS btree_gist;

CREATE INDEX idx_string_array_search ON unify USING GIN (hash_key);

CREATE OR REPLACE FUNCTION protect_unique_hash_keys()
    RETURNS TRIGGER AS $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM unify
        WHERE hash_key && NEW.hash_key
          AND id IS DISTINCT FROM NEW.id
    ) THEN
        RAISE EXCEPTION 'One of the strings in hash_key is already assigned to another record';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_ensure_unique_hash_keys
    BEFORE INSERT OR UPDATE ON unify
    FOR EACH ROW EXECUTE FUNCTION protect_unique_hash_keys();


