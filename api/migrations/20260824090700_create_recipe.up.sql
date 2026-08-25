-- Recipe header plus the attributes the roller weighs.
-- household_id nullable (NULL = a global/starter recipe). protein and cuisine are
-- optional single-valued lookups (RESTRICT: reassign before deleting a lookup in use).
-- No last_cooked column: recency is derived from cook_log (MAX(cooked_on)).
CREATE TABLE recipe (
    id              UUID DEFAULT uuidv7() CONSTRAINT pk_recipe PRIMARY KEY,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    household_id    UUID CONSTRAINT fk_recipe_household REFERENCES household(id) ON DELETE CASCADE,
    main_protein_id UUID CONSTRAINT fk_recipe_protein REFERENCES protein(id) ON DELETE RESTRICT,
    cuisine_id      UUID CONSTRAINT fk_recipe_cuisine REFERENCES cuisine(id) ON DELETE RESTRICT,
    title           TEXT NOT NULL,
    difficulty      SMALLINT NOT NULL CONSTRAINT chk_recipe_difficulty_range CHECK (difficulty BETWEEN 1 AND 5),
    time_mins       SMALLINT NOT NULL CONSTRAINT chk_recipe_time_mins_positive CHECK (time_mins > 0),
    servings        SMALLINT NOT NULL CONSTRAINT chk_recipe_servings_positive CHECK (servings > 0),
    healthiness     SMALLINT CONSTRAINT chk_recipe_healthiness_range CHECK (healthiness BETWEEN 1 AND 5),
    CONSTRAINT uq_recipe_household_title UNIQUE NULLS NOT DISTINCT (household_id, title)
);

CREATE TRIGGER recipe_set_updated_at
  BEFORE UPDATE ON recipe
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
