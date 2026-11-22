ALTER TABLE shopping_list_item DROP CONSTRAINT chk_shopping_list_item_kind;
ALTER TABLE shopping_list_item ADD COLUMN name TEXT;

UPDATE shopping_list_item
SET name = shopping_item.name
FROM shopping_item
WHERE shopping_item.id = shopping_list_item.shopping_item_id;

ALTER TABLE shopping_list_item DROP COLUMN shopping_item_id;
ALTER TABLE shopping_list_item ADD CONSTRAINT chk_shopping_list_item_kind CHECK (
    (ingredient_id IS NOT NULL AND name IS NULL)
    OR (ingredient_id IS NULL AND unit_id IS NULL AND qty IS NULL AND char_length(btrim(name)) > 0)
);

DROP TABLE shopping_item;
ALTER TABLE ingredient DROP COLUMN shopping_category_id;
DROP TABLE shopping_category;
