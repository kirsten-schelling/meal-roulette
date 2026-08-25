-- The MVP "sleep" feature: a persistent, per-household choice to keep an ingredient off
-- the shopping list until a wake date. One sleep per ingredient per household. Both FKs
-- CASCADE — a sleep record is meaningless without its household or ingredient.
CREATE TABLE slept_ingredient (
    id            UUID DEFAULT uuidv7() CONSTRAINT pk_slept_ingredient PRIMARY KEY,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    household_id  UUID NOT NULL CONSTRAINT fk_slept_ingredient_household REFERENCES household(id) ON DELETE CASCADE,
    ingredient_id UUID NOT NULL CONSTRAINT fk_slept_ingredient_ingredient REFERENCES ingredient(id) ON DELETE CASCADE,
    sleep_until   DATE NOT NULL,
    CONSTRAINT uq_slept_ingredient UNIQUE (household_id, ingredient_id)
);

CREATE TRIGGER slept_ingredient_set_updated_at
  BEFORE UPDATE ON slept_ingredient
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
