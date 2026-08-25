-- One aggregated line on a list — the summed union of the week's ingredients, one row
-- per ingredient. unit_id is a snapshot taken at generation time, so a past list stays
-- correct even if the ingredient's unit later changes.
CREATE TABLE shopping_list_item (
    id               UUID DEFAULT uuidv7() CONSTRAINT pk_shopping_list_item PRIMARY KEY,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    shopping_list_id UUID NOT NULL CONSTRAINT fk_shopping_list_item_shopping_list REFERENCES shopping_list(id) ON DELETE CASCADE,
    ingredient_id    UUID NOT NULL CONSTRAINT fk_shopping_list_item_ingredient REFERENCES ingredient(id) ON DELETE RESTRICT,
    unit_id          UUID NOT NULL CONSTRAINT fk_shopping_list_item_unit REFERENCES unit(id) ON DELETE RESTRICT,
    qty              NUMERIC(10, 2) NOT NULL CONSTRAINT chk_shopping_list_item_qty_positive CHECK (qty > 0),
    is_purchased     BOOLEAN NOT NULL DEFAULT false,
    note             TEXT,
    CONSTRAINT uq_shopping_list_item UNIQUE (shopping_list_id, ingredient_id)
);

CREATE TRIGGER shopping_list_item_set_updated_at
  BEFORE UPDATE ON shopping_list_item
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
