-- A roll no longer picks dates: it produces dinners in roll order (day_index, 1-based), and
-- dates are put on them afterwards. scheduled_date is NULL until then; the existing
-- (meal_plan_id, scheduled_date) unique constraint keeps a date to one dinner per plan, and
-- NULLs never collide in it. meal_plan.start_date records the date the dinners were last laid
-- out from, so it is NULL until a plan is scheduled.
ALTER TABLE planned_meal ADD COLUMN day_index SMALLINT;

UPDATE planned_meal
SET day_index = ordered.day_index
FROM (
    SELECT id, ROW_NUMBER() OVER (PARTITION BY meal_plan_id ORDER BY scheduled_date) AS day_index
    FROM planned_meal
) AS ordered
WHERE planned_meal.id = ordered.id;

ALTER TABLE planned_meal
  ALTER COLUMN day_index SET NOT NULL,
  ALTER COLUMN scheduled_date DROP NOT NULL,
  ADD CONSTRAINT chk_planned_meal_day_index_positive CHECK (day_index >= 1),
  ADD CONSTRAINT uq_planned_meal_day_index UNIQUE (meal_plan_id, day_index);

ALTER TABLE meal_plan ALTER COLUMN start_date DROP NOT NULL;
