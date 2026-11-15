-- How often a recipe should come round, in weeks, counted from its last cook in cook_log:
-- the roller keeps it out until min_gap_weeks have passed and pulls it in once max_gap_weeks
-- have. NULL = no limit that way.
ALTER TABLE recipe
  ADD COLUMN min_gap_weeks SMALLINT
    CONSTRAINT chk_recipe_min_gap_weeks_range CHECK (min_gap_weeks BETWEEN 1 AND 52),
  ADD COLUMN max_gap_weeks SMALLINT
    CONSTRAINT chk_recipe_max_gap_weeks_range CHECK (max_gap_weeks BETWEEN 1 AND 52),
  ADD CONSTRAINT chk_recipe_gap_order CHECK (min_gap_weeks <= max_gap_weeks);
