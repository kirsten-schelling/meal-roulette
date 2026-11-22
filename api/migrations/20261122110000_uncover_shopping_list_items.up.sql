-- The pantry is switched off, so no line stays hidden as covered by it.
UPDATE shopping_list_item SET is_covered = false WHERE is_covered;
