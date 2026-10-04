-- The rules a plan was rolled under (cuisine and protein day counts), kept so a
-- single day can be re-rolled against the same rules. The JSON shape is
-- domain::roll::RollConstraints; '{}' is the empty rule set.
ALTER TABLE meal_plan
  ADD COLUMN roll_constraints JSONB NOT NULL DEFAULT '{}'::jsonb;
