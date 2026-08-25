-- A generated shopping list (the header). meal_plan_id is nullable and SET NULL so a
-- list can be ad-hoc or outlive the plan it came from. household_id is NOT NULL.
CREATE TABLE shopping_list (
    id           UUID DEFAULT uuidv7() CONSTRAINT pk_shopping_list PRIMARY KEY,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    household_id UUID NOT NULL CONSTRAINT fk_shopping_list_household REFERENCES household(id) ON DELETE CASCADE,
    meal_plan_id UUID CONSTRAINT fk_shopping_list_meal_plan REFERENCES meal_plan(id) ON DELETE SET NULL,
    name         TEXT
);

CREATE TRIGGER shopping_list_set_updated_at
  BEFORE UPDATE ON shopping_list
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
