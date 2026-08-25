-- Lookup vocabulary for a recipe's main protein (chicken, beef, tofu…).
-- household-extensible presets, same nullable-ownership pattern as unit.
CREATE TABLE protein (
    id           UUID DEFAULT uuidv7() CONSTRAINT pk_protein PRIMARY KEY,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    household_id UUID CONSTRAINT fk_protein_household REFERENCES household(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    CONSTRAINT uq_protein_household_name UNIQUE NULLS NOT DISTINCT (household_id, name)
);

CREATE TRIGGER protein_set_updated_at
  BEFORE UPDATE ON protein
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
