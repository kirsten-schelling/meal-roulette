-- Shared trigger function that stamps updated_at on every row update.
-- Lives in its own migration because every table's updated_at trigger depends on it;
-- keeping it separate means reverting a single table never drops this shared function.
CREATE OR REPLACE FUNCTION set_updated_at() RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;
