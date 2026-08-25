-- Ingredients that make up a recipe. Quantity is in the ingredient's own unit
-- (simple mode — no conversion). Deleting a recipe removes its lines (CASCADE);
-- an ingredient in use cannot be deleted (RESTRICT).
CREATE TABLE recipe_ingredient (
    id            UUID DEFAULT uuidv7() CONSTRAINT pk_recipe_ingredient PRIMARY KEY,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    recipe_id     UUID NOT NULL CONSTRAINT fk_recipe_ingredient_recipe REFERENCES recipe(id) ON DELETE CASCADE,
    ingredient_id UUID NOT NULL CONSTRAINT fk_recipe_ingredient_ingredient REFERENCES ingredient(id) ON DELETE RESTRICT,
    qty           NUMERIC(10, 2) NOT NULL CONSTRAINT chk_recipe_ingredient_qty_positive CHECK (qty > 0),
    note          TEXT,
    is_optional   BOOLEAN NOT NULL DEFAULT false,
    CONSTRAINT uq_recipe_ingredient UNIQUE (recipe_id, ingredient_id)
);

CREATE TRIGGER recipe_ingredient_set_updated_at
  BEFORE UPDATE ON recipe_ingredient
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
