-- Add down migration script here
DROP TRIGGER recipe_set_updated_at ON recipe;

DROP FUNCTION set_updated_at;

DROP TABLE recipe;