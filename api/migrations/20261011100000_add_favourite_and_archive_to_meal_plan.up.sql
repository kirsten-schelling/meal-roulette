-- A favourite plan is one worth keeping; an archived plan drops out of the active history
-- and is never the "latest" plan, but keeps its days. archived_at NULL = active.
ALTER TABLE meal_plan
  ADD COLUMN is_favourite BOOLEAN NOT NULL DEFAULT false,
  ADD COLUMN archived_at  TIMESTAMPTZ;
