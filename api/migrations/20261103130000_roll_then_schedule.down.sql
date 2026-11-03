-- Dinners without a date go after the plan's last dated one, in roll order, so the restored
-- (meal_plan_id, scheduled_date) uniqueness holds.
UPDATE planned_meal
SET scheduled_date = undated.last_date + undated.position
FROM (
    SELECT planned_meal.id,
           COALESCE(
               (SELECT MAX(dated.scheduled_date) FROM planned_meal AS dated
                WHERE dated.meal_plan_id = planned_meal.meal_plan_id),
               meal_plan.start_date - 1,
               meal_plan.created_at::date - 1
           ) AS last_date,
           ROW_NUMBER() OVER (PARTITION BY planned_meal.meal_plan_id ORDER BY planned_meal.day_index)::int AS position
    FROM planned_meal
    JOIN meal_plan ON meal_plan.id = planned_meal.meal_plan_id
    WHERE planned_meal.scheduled_date IS NULL
) AS undated
WHERE planned_meal.id = undated.id;

UPDATE meal_plan
SET start_date = COALESCE(
    (SELECT MIN(planned_meal.scheduled_date) FROM planned_meal WHERE planned_meal.meal_plan_id = meal_plan.id),
    meal_plan.created_at::date
)
WHERE start_date IS NULL;

ALTER TABLE meal_plan ALTER COLUMN start_date SET NOT NULL;

ALTER TABLE planned_meal
  DROP CONSTRAINT uq_planned_meal_day_index,
  DROP CONSTRAINT chk_planned_meal_day_index_positive,
  ALTER COLUMN scheduled_date SET NOT NULL,
  DROP COLUMN day_index;
