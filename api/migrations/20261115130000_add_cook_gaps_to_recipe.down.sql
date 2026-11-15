ALTER TABLE recipe
  DROP CONSTRAINT chk_recipe_gap_order,
  DROP COLUMN max_gap_weeks,
  DROP COLUMN min_gap_weeks;
