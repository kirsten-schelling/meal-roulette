-- One rolled week (the plan header). household_id is NOT NULL — there is no such thing
-- as a global meal plan. seed is stored so a roll can be reproduced.
CREATE TABLE meal_plan (
    id           UUID DEFAULT uuidv7() CONSTRAINT pk_meal_plan PRIMARY KEY,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    household_id UUID NOT NULL CONSTRAINT fk_meal_plan_household REFERENCES household(id) ON DELETE CASCADE,
    start_date   DATE NOT NULL,
    seed         BIGINT,
    name         TEXT
);

CREATE TRIGGER meal_plan_set_updated_at
  BEFORE UPDATE ON meal_plan
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
