-- A recipe line may leave its amount unsaid ("salt, to taste") and an ingredient may have
-- no fixed pack size. NULL = not given; the positive checks still hold for any value set.
-- A shopping-list line sums its recipes' amounts, so it is NULL when none of them gave one.
ALTER TABLE ingredient ALTER COLUMN purchase_size DROP NOT NULL;
ALTER TABLE recipe_ingredient ALTER COLUMN qty DROP NOT NULL;
ALTER TABLE shopping_list_item ALTER COLUMN qty DROP NOT NULL;
