-- Add down migration script here
DROP TRIGGER recipe_ingredient_set_updated_at ON recipe_ingredient;

DROP TABLE recipe_ingredient;
