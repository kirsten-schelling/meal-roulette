-- Sections the shopping list is grouped by (produce, dairy...), in the order the user walks the
-- shop. An ingredient or other shopping item may sit in one; deleting a category leaves its
-- members uncategorised.
CREATE TABLE shopping_category (
    id         UUID DEFAULT uuidv7() CONSTRAINT pk_shopping_category PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    name       TEXT NOT NULL CONSTRAINT chk_shopping_category_name CHECK (char_length(btrim(name)) > 0),
    position   INTEGER NOT NULL,
    CONSTRAINT uq_shopping_category_name UNIQUE (name)
);

CREATE TRIGGER shopping_category_set_updated_at
  BEFORE UPDATE ON shopping_category
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

ALTER TABLE ingredient
    ADD COLUMN shopping_category_id UUID
        CONSTRAINT fk_ingredient_shopping_category REFERENCES shopping_category(id) ON DELETE SET NULL;

-- Things bought that are not recipe ingredients (bin bags, candles). A free-text list line now
-- points at one, so its category is remembered the next time it is added. Names match without
-- regard to case.
CREATE TABLE shopping_item (
    id                   UUID DEFAULT uuidv7() CONSTRAINT pk_shopping_item PRIMARY KEY,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    name                 TEXT NOT NULL CONSTRAINT chk_shopping_item_name CHECK (char_length(btrim(name)) > 0),
    shopping_category_id UUID
        CONSTRAINT fk_shopping_item_shopping_category REFERENCES shopping_category(id) ON DELETE SET NULL
);

CREATE UNIQUE INDEX uq_shopping_item_name ON shopping_item (lower(name));

CREATE TRIGGER shopping_item_set_updated_at
  BEFORE UPDATE ON shopping_item
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

ALTER TABLE shopping_list_item
    ADD COLUMN shopping_item_id UUID
        CONSTRAINT fk_shopping_list_item_shopping_item REFERENCES shopping_item(id) ON DELETE CASCADE;

INSERT INTO shopping_item (name)
SELECT DISTINCT ON (lower(btrim(name))) btrim(name)
FROM shopping_list_item
WHERE name IS NOT NULL
ORDER BY lower(btrim(name)), created_at;

UPDATE shopping_list_item
SET shopping_item_id = shopping_item.id
FROM shopping_item
WHERE lower(btrim(shopping_list_item.name)) = lower(shopping_item.name);

ALTER TABLE shopping_list_item DROP CONSTRAINT chk_shopping_list_item_kind;
ALTER TABLE shopping_list_item DROP COLUMN name;
ALTER TABLE shopping_list_item ADD CONSTRAINT chk_shopping_list_item_kind CHECK (
    (ingredient_id IS NOT NULL AND shopping_item_id IS NULL)
    OR (ingredient_id IS NULL AND shopping_item_id IS NOT NULL AND unit_id IS NULL AND qty IS NULL)
);
