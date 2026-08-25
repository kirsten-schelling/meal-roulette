-- Recipe <-> tag junction (many-to-many). Deleting a recipe removes its tag links
-- (CASCADE); a tag in use cannot be deleted (RESTRICT).
CREATE TABLE recipe_tag (
    id         UUID DEFAULT uuidv7() CONSTRAINT pk_recipe_tag PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    recipe_id  UUID NOT NULL CONSTRAINT fk_recipe_tag_recipe REFERENCES recipe(id) ON DELETE CASCADE,
    tag_id     UUID NOT NULL CONSTRAINT fk_recipe_tag_tag REFERENCES tag(id) ON DELETE RESTRICT,
    CONSTRAINT uq_recipe_tag UNIQUE (recipe_id, tag_id)
);

CREATE TRIGGER recipe_tag_set_updated_at
  BEFORE UPDATE ON recipe_tag
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
