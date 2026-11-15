-- Unitless ingredients and lines get a global "each" so NOT NULL can come back.
INSERT INTO unit (name) VALUES ('each') ON CONFLICT ON CONSTRAINT uq_unit_household_name DO NOTHING;
UPDATE ingredient
SET unit_id = (SELECT id FROM unit WHERE household_id IS NULL AND name = 'each')
WHERE unit_id IS NULL;
UPDATE shopping_list_item
SET unit_id = (SELECT id FROM unit WHERE household_id IS NULL AND name = 'each')
WHERE ingredient_id IS NOT NULL AND unit_id IS NULL;
ALTER TABLE shopping_list_item DROP CONSTRAINT chk_shopping_list_item_kind;
ALTER TABLE shopping_list_item ADD CONSTRAINT chk_shopping_list_item_kind CHECK (
    (ingredient_id IS NOT NULL AND unit_id IS NOT NULL AND name IS NULL)
    OR (ingredient_id IS NULL AND unit_id IS NULL AND qty IS NULL AND char_length(btrim(name)) > 0)
);
ALTER TABLE ingredient ALTER COLUMN unit_id SET NOT NULL;
