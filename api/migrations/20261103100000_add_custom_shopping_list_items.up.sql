-- A list line may be free text with no ingredient behind it ("bin bags"). Such a line has a
-- name and nothing else; an ingredient line keeps its ingredient and unit and has no name.
ALTER TABLE shopping_list_item ALTER COLUMN ingredient_id DROP NOT NULL;
ALTER TABLE shopping_list_item ALTER COLUMN unit_id DROP NOT NULL;
ALTER TABLE shopping_list_item ADD COLUMN name TEXT;
ALTER TABLE shopping_list_item ADD CONSTRAINT chk_shopping_list_item_kind CHECK (
    (ingredient_id IS NOT NULL AND unit_id IS NOT NULL AND name IS NULL)
    OR (ingredient_id IS NULL AND unit_id IS NULL AND qty IS NULL AND char_length(btrim(name)) > 0)
);
