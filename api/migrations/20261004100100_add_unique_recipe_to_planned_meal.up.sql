-- A recipe appears at most once in a plan: the solver may not pick it twice and
-- a swap may not reuse a recipe already on another day of the same plan.
ALTER TABLE planned_meal
  ADD CONSTRAINT uq_planned_meal_recipe UNIQUE (meal_plan_id, recipe_id);
