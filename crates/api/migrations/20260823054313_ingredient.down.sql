-- Add down migration script here

DROP TRIGGER unit_set_updated_at ON unit;

DROP TRIGGER ingredient_set_updated_at ON ingredient;

DROP TABLE ingredient;

DROP TABLE unit;
