-- The permanent record of what was actually cooked (intent lives in planned_meal; this
-- is fact). Source of truth for recency and min/max-interval rules. planned_meal_id is
-- nullable and SET NULL, so unplanned cooks are recordable and history survives if a plan
-- is deleted. A cooked recipe cannot be deleted (RESTRICT) — that would erase the record.
CREATE TABLE cook_log (
    id              UUID DEFAULT uuidv7() CONSTRAINT pk_cook_log PRIMARY KEY,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    household_id    UUID NOT NULL CONSTRAINT fk_cook_log_household REFERENCES household(id) ON DELETE CASCADE,
    recipe_id       UUID NOT NULL CONSTRAINT fk_cook_log_recipe REFERENCES recipe(id) ON DELETE RESTRICT,
    planned_meal_id UUID CONSTRAINT fk_cook_log_planned_meal REFERENCES planned_meal(id) ON DELETE SET NULL,
    cooked_on       DATE NOT NULL
);

CREATE TRIGGER cook_log_set_updated_at
  BEFORE UPDATE ON cook_log
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
