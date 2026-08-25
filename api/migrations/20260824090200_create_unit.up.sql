-- Units of measure (g, ml, each). household_id is nullable: NULL = a global preset
-- shared by everyone; a value = owned by that household. NULLS NOT DISTINCT makes two
-- global rows with the same name collide, which the default (NULLs distinct) would allow.
CREATE TABLE unit (
    id           UUID DEFAULT uuidv7() CONSTRAINT pk_unit PRIMARY KEY,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    household_id UUID CONSTRAINT fk_unit_household REFERENCES household(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    description  TEXT,
    CONSTRAINT uq_unit_household_name UNIQUE NULLS NOT DISTINCT (household_id, name)
);

CREATE TRIGGER unit_set_updated_at
  BEFORE UPDATE ON unit
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
