-- Add up migration script here
CREATE TABLE unit (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    name TEXT NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE ingredient (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    name TEXT NOT NULL,
    purchase_size INTEGER NOT NULL,
    unit_id UUID NOT NULL REFERENCES unit(id)
);

-- ingredient table trigger
CREATE TRIGGER ingredient_set_updated_at
  BEFORE UPDATE ON ingredient
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- unit table trigger
CREATE TRIGGER unit_set_updated_at
  BEFORE UPDATE ON unit
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();