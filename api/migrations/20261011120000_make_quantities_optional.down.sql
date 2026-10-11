-- Refuses while any row still holds a NULL: give those rows a value before reverting.
ALTER TABLE shopping_list_item ALTER COLUMN qty SET NOT NULL;
ALTER TABLE recipe_ingredient ALTER COLUMN qty SET NOT NULL;
ALTER TABLE ingredient ALTER COLUMN purchase_size SET NOT NULL;
