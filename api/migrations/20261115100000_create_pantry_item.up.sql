-- What the household estimates it has in stock: one number per ingredient, in the ingredient's
-- own unit. It is an estimate the user can overwrite at any time; ticking a purchase on the
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

-- pantry_added is what ticking the line put into the pantry, so unticking takes back exactly that.
-- is_covered marks a line the pantry already covers in full: it stays on the list, out of the
-- to-buy part, until the user adds it anyway.
ALTER TABLE shopping_list_item
    ADD COLUMN pantry_added NUMERIC(10, 2)
        CONSTRAINT chk_shopping_list_item_pantry_added_positive CHECK (pantry_added > 0),
    ADD COLUMN is_covered BOOLEAN NOT NULL DEFAULT false;
