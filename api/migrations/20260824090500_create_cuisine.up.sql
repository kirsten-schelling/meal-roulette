-- Lookup vocabulary for a recipe's cuisine (italian, thai…).
-- Same nullable-ownership pattern as unit/protein.
CREATE TABLE cuisine (
    id           UUID DEFAULT uuidv7() CONSTRAINT pk_cuisine PRIMARY KEY,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    household_id UUID CONSTRAINT fk_cuisine_household REFERENCES household(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    CONSTRAINT uq_cuisine_household_name UNIQUE NULLS NOT DISTINCT (household_id, name)
);

CREATE TRIGGER cuisine_set_updated_at
  BEFORE UPDATE ON cuisine
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
