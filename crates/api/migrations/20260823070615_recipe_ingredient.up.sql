-- Add up migration script here
CREATE TABLE recipe_ingredient (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    recipe_id UUID NOT NULL REFERENCES recipe(id) ON DELETE CASCADE,
    ingredient_id UUID NOT NULL REFERENCES ingredient(id) ON DELETE RESTRICT,
    qty NUMERIC(10, 2) NOT NULL CHECK (qty > 0),
    note TEXT,
    optional BOOLEAN NOT NULL DEFAULT false,
    UNIQUE (recipe_id, ingredient_id)
);

-- recipe_ingredient table trigger
CREATE TRIGGER recipe_ingredient_set_updated_at
  BEFORE UPDATE ON recipe_ingredient
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
