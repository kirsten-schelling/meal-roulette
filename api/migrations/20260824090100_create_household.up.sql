-- Ownership anchor. A household owns recipes, ingredients, plans, and lists.
-- Runs solo against a single seeded row until auth/members arrive.
CREATE TABLE household (
    id         UUID DEFAULT uuidv7() CONSTRAINT pk_household PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    name       TEXT NOT NULL
);

CREATE TRIGGER household_set_updated_at
  BEFORE UPDATE ON household
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
