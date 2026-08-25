-- One dinner slot in a plan. Deleting the plan removes its slots (CASCADE); a recipe
-- that is planned cannot be deleted (RESTRICT). target_difficulty is the per-day
-- override (NULL = use the day's default). One dinner per day per plan.
CREATE TABLE planned_meal (
    id                UUID DEFAULT uuidv7() CONSTRAINT pk_planned_meal PRIMARY KEY,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    meal_plan_id      UUID NOT NULL CONSTRAINT fk_planned_meal_meal_plan REFERENCES meal_plan(id) ON DELETE CASCADE,
    recipe_id         UUID NOT NULL CONSTRAINT fk_planned_meal_recipe REFERENCES recipe(id) ON DELETE RESTRICT,
    scheduled_date    DATE NOT NULL,
    is_pinned         BOOLEAN NOT NULL DEFAULT false,
    target_difficulty SMALLINT CONSTRAINT chk_planned_meal_target_difficulty_range CHECK (target_difficulty BETWEEN 1 AND 5),
    CONSTRAINT uq_planned_meal UNIQUE (meal_plan_id, scheduled_date)
);

CREATE TRIGGER planned_meal_set_updated_at
  BEFORE UPDATE ON planned_meal
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
