# 4. An ingredient's unit is optional

- Status: Accepted
- Date: 2026-11-15
- Amends: [0003](0003-simple-units.md), whose rule that every ingredient has exactly one unit
  no longer holds. The rest of 0003 stands.

## Context

[0003](0003-simple-units.md) chose simple mode and said every ingredient has exactly one unit.
Some ingredients have no natural unit: eggs are counted ("2 eggs") and salt is not measured at
all. A required unit meant inventing one such as "each", and recipe import had to guess it.

Migration `20261114100000_make_ingredient_unit_optional` made `ingredient.unit_id` nullable.

## Decision

- An ingredient has at most one unit. With none, its recipe amounts are plain counts, or absent.
- Everything else in 0003 holds: every recipe line for an ingredient is in the ingredient's
  unit, the shopping list sums per ingredient and never converts, and
  `shopping_list_item.unit_id` is a snapshot of the ingredient's unit, `NULL` when it has none.
- A unitless ingredient is its own list line. It never merges into another ingredient's total.

## Consequences

- `chk_shopping_list_item_kind` no longer requires a unit on an ingredient line; a free-text
  line still has no ingredient, unit or amount.
- Read models carry the unit name as a string, empty when the ingredient has no unit, so the UI
  shows an amount with no trailing unit.
- Recipe import leaves a count line ("3 eggs") unitless instead of giving it a unit.
- Giving a unitless ingredient a unit later is the same manual cleanup as changing a unit under
  0003: existing recipe amounts keep their numbers.
- Reverting the migration gives every unitless ingredient and list line a global "each" unit,
  so the `NOT NULL` can return.
