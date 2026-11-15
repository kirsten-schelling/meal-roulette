-- An ingredient may have no unit ("2 eggs", "salt"). Its shopping list lines then have none
-- either, so the ingredient branch of the line check no longer asks for a unit.
ALTER TABLE ingredient ALTER COLUMN unit_id DROP NOT NULL;
ALTER TABLE shopping_list_item DROP CONSTRAINT chk_shopping_list_item_kind;
ALTER TABLE shopping_list_item ADD CONSTRAINT chk_shopping_list_item_kind CHECK (
    (ingredient_id IS NOT NULL AND name IS NULL)
    OR (ingredient_id IS NULL AND unit_id IS NULL AND qty IS NULL AND char_length(btrim(name)) > 0)
);
