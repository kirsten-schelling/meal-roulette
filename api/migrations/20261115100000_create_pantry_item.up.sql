-- What the household estimates it has in stock: one number per ingredient, in the ingredient's
-- own unit. It is an estimate the user can overwrite at any time; a purchase confirmed on the
-- shopping list adds to it and a cooked dinner takes from it. A count never goes below 0.
-- Both FKs CASCADE: a count is meaningless without its household or ingredient.
CREATE TABLE pantry_item (
    id            UUID DEFAULT uuidv7() CONSTRAINT pk_pantry_item PRIMARY KEY,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    household_id  UUID NOT NULL CONSTRAINT fk_pantry_item_household REFERENCES household(id) ON DELETE CASCADE,
    ingredient_id UUID NOT NULL CONSTRAINT fk_pantry_item_ingredient REFERENCES ingredient(id) ON DELETE CASCADE,
    qty           NUMERIC(10, 2) NOT NULL CONSTRAINT chk_pantry_item_qty_not_negative CHECK (qty >= 0),
    CONSTRAINT uq_pantry_item UNIQUE (household_id, ingredient_id)
);

CREATE TRIGGER pantry_item_set_updated_at
  BEFORE UPDATE ON pantry_item
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
