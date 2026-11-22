-- A line the user put on the list themselves, as opposed to one the plan's recipes need. A
-- rebuild never removes it, and it moves on to the next week's list until it is bought. Lines
-- that are not ingredients were always added by hand.
ALTER TABLE shopping_list_item ADD COLUMN added_by_hand BOOLEAN NOT NULL DEFAULT false;
UPDATE shopping_list_item SET added_by_hand = true WHERE ingredient_id IS NULL;
ALTER TABLE shopping_list_item ADD CONSTRAINT chk_shopping_list_item_item_added_by_hand
    CHECK (ingredient_id IS NOT NULL OR added_by_hand);
