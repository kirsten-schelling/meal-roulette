-- A plan has at most one shopping list; regenerating updates that list in place. Ad-hoc
-- lists (meal_plan_id NULL) are unaffected, since NULLs never collide here.
ALTER TABLE shopping_list
  ADD CONSTRAINT uq_shopping_list_meal_plan UNIQUE (meal_plan_id);
