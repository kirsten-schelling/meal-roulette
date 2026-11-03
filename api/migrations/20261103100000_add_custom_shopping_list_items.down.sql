-- Deletes the free-text lines: they have no ingredient to revert to.
DELETE FROM shopping_list_item WHERE ingredient_id IS NULL;
ALTER TABLE shopping_list_item DROP CONSTRAINT chk_shopping_list_item_kind;
ALTER TABLE shopping_list_item DROP COLUMN name;
ALTER TABLE shopping_list_item ALTER COLUMN unit_id SET NOT NULL;
ALTER TABLE shopping_list_item ALTER COLUMN ingredient_id SET NOT NULL;
