-- Recipe table initial migration
CREATE TABLE recipe (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    title       TEXT NOT NULL,
    difficulty  SMALLINT NOT NULL,
    time_mins   SMALLINT NOT NULL,
    last_cooked DATE
);

-- Updated at function
CREATE OR REPLACE FUNCTION set_updated_at() RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- recipe table trigger
CREATE TRIGGER recipe_set_updated_at
  BEFORE UPDATE ON recipe
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();