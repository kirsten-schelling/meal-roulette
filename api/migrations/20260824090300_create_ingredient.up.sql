-- A purchasable ingredient and the unit it is measured/bought in.
-- household_id nullable (NULL = global preset). unit_id is RESTRICT: a unit in use
-- cannot be deleted out from under an ingredient.
CREATE TABLE ingredient (
    id            UUID DEFAULT uuidv7() CONSTRAINT pk_ingredient PRIMARY KEY,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    household_id  UUID CONSTRAINT fk_ingredient_household REFERENCES household(id) ON DELETE CASCADE,
    unit_id       UUID NOT NULL CONSTRAINT fk_ingredient_unit REFERENCES unit(id) ON DELETE RESTRICT,
    name          TEXT NOT NULL,
    purchase_size INTEGER NOT NULL CONSTRAINT chk_ingredient_purchase_size_positive CHECK (purchase_size > 0),
    CONSTRAINT uq_ingredient_household_name UNIQUE NULLS NOT DISTINCT (household_id, name)
);

CREATE TRIGGER ingredient_set_updated_at
  BEFORE UPDATE ON ingredient
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
