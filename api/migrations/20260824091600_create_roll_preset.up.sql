-- A saved set of roll-page settings (days, constraints, must-use ingredients),
-- stored as an opaque JSON payload. Same nullable-ownership pattern as the lookups.
CREATE TABLE roll_preset (
    id           UUID DEFAULT uuidv7() CONSTRAINT pk_roll_preset PRIMARY KEY,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    household_id UUID CONSTRAINT fk_roll_preset_household REFERENCES household(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    settings     JSONB NOT NULL,
    is_default   BOOLEAN NOT NULL DEFAULT false,
    CONSTRAINT uq_roll_preset_household_name UNIQUE NULLS NOT DISTINCT (household_id, name)
);

-- At most one default preset per household (a NULL household is the shared/global set).
CREATE UNIQUE INDEX uq_roll_preset_one_default
  ON roll_preset (household_id)
  NULLS NOT DISTINCT
  WHERE is_default;

CREATE TRIGGER roll_preset_set_updated_at
  BEFORE UPDATE ON roll_preset
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
