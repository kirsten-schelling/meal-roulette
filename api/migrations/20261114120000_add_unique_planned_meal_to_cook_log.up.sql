-- A planned dinner is cooked at most once; unplanned cooks (NULL) are unlimited.
ALTER TABLE cook_log
  ADD CONSTRAINT uq_cook_log_planned_meal UNIQUE (planned_meal_id);
