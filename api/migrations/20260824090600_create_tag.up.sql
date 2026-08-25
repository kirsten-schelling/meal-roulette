-- Free-form recipe labels (quick, vegetarian, comfort…). Many-per-recipe, attached
-- via recipe_tag. Same nullable-ownership pattern as the other lookups.
CREATE TABLE tag (
    id           UUID DEFAULT uuidv7() CONSTRAINT pk_tag PRIMARY KEY,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    household_id UUID CONSTRAINT fk_tag_household REFERENCES household(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    CONSTRAINT uq_tag_household_name UNIQUE NULLS NOT DISTINCT (household_id, name)
);

CREATE TRIGGER tag_set_updated_at
  BEFORE UPDATE ON tag
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
