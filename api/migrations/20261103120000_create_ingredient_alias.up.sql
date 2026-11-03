-- A remembered import choice: a normalised recipe-page phrase ("large eggs" becomes "egg")
-- mapped to the library ingredient the user picked for it, so the next import skips the guess.
-- household_id nullable (NULL = global), like ingredient. Deleting the ingredient drops its aliases.
CREATE TABLE ingredient_alias (
    id            UUID DEFAULT uuidv7() CONSTRAINT pk_ingredient_alias PRIMARY KEY,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    household_id  UUID CONSTRAINT fk_ingredient_alias_household REFERENCES household(id) ON DELETE CASCADE,
    ingredient_id UUID NOT NULL CONSTRAINT fk_ingredient_alias_ingredient REFERENCES ingredient(id) ON DELETE CASCADE,
    phrase        TEXT NOT NULL CONSTRAINT chk_ingredient_alias_phrase_normalised CHECK (phrase <> '' AND phrase = lower(btrim(phrase))),
    CONSTRAINT uq_ingredient_alias_household_phrase UNIQUE NULLS NOT DISTINCT (household_id, phrase)
);

CREATE INDEX ix_ingredient_alias_ingredient ON ingredient_alias (ingredient_id);

CREATE TRIGGER ingredient_alias_set_updated_at
  BEFORE UPDATE ON ingredient_alias
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
