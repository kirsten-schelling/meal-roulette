use crate::features::unit;
use crate::references::{to_references, ReferenceRow};
use domain::ingredient::{
    BulkIngredientUpdate, FieldChange, Ingredient, IngredientEntry, InsertIngredient,
    NewIngredient, UpdateIngredient,
};
use domain::usage::{Reference, ReferenceKind};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<Ingredient>, sqlx::Error> {
    let ingredients = sqlx::query_as!(
        Ingredient,
        r#"
        SELECT ingredient.id, ingredient.household_id, ingredient.name, ingredient.purchase_size, COALESCE(unit.name, '') as "unit!", ingredient.unit_id, ingredient.shopping_category_id, ingredient.created_at, ingredient.updated_at
        FROM ingredient
        LEFT JOIN unit ON ingredient.unit_id = unit.id
        ORDER BY ingredient.created_at DESC
    "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(ingredients)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Ingredient, sqlx::Error> {
    let ingredient = sqlx::query_as!(
        Ingredient,
        r#"
        SELECT ingredient.id, ingredient.household_id, ingredient.name, ingredient.purchase_size, COALESCE(unit.name, '') as "unit!", ingredient.unit_id, ingredient.shopping_category_id, ingredient.created_at, ingredient.updated_at
        FROM ingredient
        LEFT JOIN unit ON ingredient.unit_id = unit.id
        WHERE ingredient.id = $1
    "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(ingredient)
}

pub async fn create<'e, E>(
    executor: E,
    ingredient: InsertIngredient,
) -> Result<Ingredient, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    let ingredient = sqlx::query_as!(
        Ingredient,
        r#"
        with new_ingredient as (
            INSERT INTO ingredient (household_id, name, purchase_size, unit_id, shopping_category_id) VALUES ($1, $2, $3, $4, $5) RETURNING id, household_id, name, purchase_size, unit_id, shopping_category_id, created_at, updated_at
        )
        SELECT new_ingredient.id, new_ingredient.household_id, new_ingredient.name, new_ingredient.purchase_size, COALESCE(unit.name, '') as "unit!", new_ingredient.unit_id, new_ingredient.shopping_category_id, new_ingredient.created_at, new_ingredient.updated_at
        FROM new_ingredient
        LEFT JOIN unit ON new_ingredient.unit_id = unit.id
        "#,
        ingredient.household_id,
        ingredient.name,
        ingredient.purchase_size,
        ingredient.unit_id,
        ingredient.shopping_category_id,
        )
        .fetch_one(executor)
        .await?;
    Ok(ingredient)
}

/// Like units, an ingredient typed as new reuses one that already has that name.
pub async fn resolve(conn: &mut PgConnection, entry: IngredientEntry) -> Result<Uuid, sqlx::Error> {
    let (name, purchase_size, unit) = match entry {
        IngredientEntry::Existing(id) => return Ok(id),
        IngredientEntry::New {
            name,
            purchase_size,
            unit,
        } => (name.trim().to_string(), purchase_size, unit),
    };
    let existing = sqlx::query_scalar!(
        r#"
        SELECT id FROM ingredient
        WHERE household_id IS NULL AND lower(name) = lower($1)
        "#,
        name,
    )
    .fetch_optional(&mut *conn)
    .await?;
    if let Some(id) = existing {
        return Ok(id);
    }
    let unit_id = unit::repository::resolve(&mut *conn, unit).await?;
    let ingredient = create(
        conn,
        InsertIngredient {
            household_id: None,
            name,
            purchase_size,
            unit_id,
            shopping_category_id: None,
        },
    )
    .await?;
    Ok(ingredient.id)
}

pub async fn create_with_unit(
    pool: &PgPool,
    new_ingredient: NewIngredient,
) -> Result<Ingredient, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let unit_id = unit::repository::resolve(&mut tx, new_ingredient.unit).await?;
    let ingredient = create(
        &mut *tx,
        InsertIngredient {
            household_id: new_ingredient.household_id,
            name: new_ingredient.name,
            purchase_size: new_ingredient.purchase_size,
            unit_id,
            shopping_category_id: new_ingredient.shopping_category_id,
        },
    )
    .await?;
    tx.commit().await?;
    Ok(ingredient)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    ingredient: UpdateIngredient,
) -> Result<Ingredient, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let unit_id = unit::repository::resolve(&mut tx, ingredient.unit).await?;
    let updated = sqlx::query_as!(
        Ingredient,
        r#"
        with updated_ingredient as (
            UPDATE ingredient SET name = $1, purchase_size = $2, unit_id = $3, shopping_category_id = $5 WHERE id = $4 RETURNING id, household_id, name, purchase_size, unit_id, shopping_category_id, created_at, updated_at
        )
        SELECT updated_ingredient.id, updated_ingredient.household_id, updated_ingredient.name, updated_ingredient.purchase_size, COALESCE(unit.name, '') as "unit!", updated_ingredient.unit_id, updated_ingredient.shopping_category_id, updated_ingredient.created_at, updated_ingredient.updated_at
        FROM updated_ingredient
        LEFT JOIN unit ON updated_ingredient.unit_id = unit.id
        "#,
        ingredient.name.trim(),
        ingredient.purchase_size,
        unit_id,
        id,
        ingredient.shopping_category_id,
    )
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(updated)
}

/// Apply the same changes to every ingredient in `ids`; returns how many were changed.
pub async fn update_many(
    pool: &PgPool,
    ids: &[Uuid],
    changes: BulkIngredientUpdate,
) -> Result<u64, sqlx::Error> {
    let (set_unit, unit_id) = match changes.unit_id {
        FieldChange::Keep => (false, None),
        FieldChange::Set(id) => (true, id),
    };
    let (set_size, purchase_size) = match changes.purchase_size {
        FieldChange::Keep => (false, None),
        FieldChange::Set(size) => (true, Some(size)),
    };
    let (set_category, category_id) = match changes.shopping_category_id {
        FieldChange::Keep => (false, None),
        FieldChange::Set(id) => (true, id),
    };
    let changed = sqlx::query_scalar!(
        r#"
        UPDATE ingredient
        SET unit_id = CASE WHEN $2::bool THEN $3::uuid ELSE unit_id END,
            purchase_size = CASE WHEN $4::bool THEN $5::int ELSE purchase_size END,
            shopping_category_id = CASE WHEN $6::bool THEN $7::uuid ELSE shopping_category_id END
        WHERE id = ANY($1)
        RETURNING id
        "#,
        ids,
        set_unit,
        unit_id,
        set_size,
        purchase_size,
        set_category,
        category_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(changed.len() as u64)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<Ingredient, sqlx::Error> {
    let ingredient = sqlx::query_as!(
        Ingredient,
        r#"
        with deleted_ingredient as (
            DELETE FROM ingredient WHERE id = $1 RETURNING id, household_id, name, purchase_size, unit_id, shopping_category_id, created_at, updated_at
        )
        SELECT deleted_ingredient.id, deleted_ingredient.household_id, deleted_ingredient.name, deleted_ingredient.purchase_size, COALESCE(unit.name, '') as "unit!", deleted_ingredient.unit_id, deleted_ingredient.shopping_category_id, deleted_ingredient.created_at, deleted_ingredient.updated_at
        FROM deleted_ingredient
        LEFT JOIN unit ON deleted_ingredient.unit_id = unit.id
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(ingredient)
}

pub async fn references(pool: &PgPool, id: Uuid) -> Result<Vec<Reference>, sqlx::Error> {
    let recipes = sqlx::query_as!(
        ReferenceRow,
        r#"
        SELECT recipe.id, recipe.title AS name
        FROM recipe
        WHERE EXISTS (
            SELECT 1 FROM recipe_ingredient
            WHERE recipe_ingredient.recipe_id = recipe.id
              AND recipe_ingredient.ingredient_id = $1
        )
        ORDER BY recipe.title, recipe.id
        "#,
        id,
    )
    .fetch_all(pool)
    .await?;
    let lists = sqlx::query_as!(
        ReferenceRow,
        r#"
        SELECT shopping_list.id, COALESCE(shopping_list.name, 'Shopping list') AS "name!"
        FROM shopping_list
        WHERE EXISTS (
            SELECT 1 FROM shopping_list_item
            WHERE shopping_list_item.shopping_list_id = shopping_list.id
              AND shopping_list_item.ingredient_id = $1
        )
        ORDER BY 2, shopping_list.id
        "#,
        id,
    )
    .fetch_all(pool)
    .await?;
    let mut found = to_references(recipes, ReferenceKind::Recipe);
    found.extend(to_references(lists, ReferenceKind::ShoppingList));
    Ok(found)
}

/// Removes the ingredient from every recipe and shopping list that uses it, then deletes it,
/// in one transaction. Only reachable through an explicit cascade request.
pub async fn delete_with_references(pool: &PgPool, id: Uuid) -> Result<Ingredient, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query_scalar!(
        "DELETE FROM recipe_ingredient WHERE ingredient_id = $1 RETURNING id",
        id,
    )
    .fetch_all(&mut *tx)
    .await?;
    sqlx::query_scalar!(
        "DELETE FROM shopping_list_item WHERE ingredient_id = $1 RETURNING id",
        id,
    )
    .fetch_all(&mut *tx)
    .await?;
    let ingredient = sqlx::query_as!(
        Ingredient,
        r#"
        with deleted_ingredient as (
            DELETE FROM ingredient WHERE id = $1 RETURNING id, household_id, name, purchase_size, unit_id, shopping_category_id, created_at, updated_at
        )
        SELECT deleted_ingredient.id, deleted_ingredient.household_id, deleted_ingredient.name, deleted_ingredient.purchase_size, COALESCE(unit.name, '') as "unit!", deleted_ingredient.unit_id, deleted_ingredient.shopping_category_id, deleted_ingredient.created_at, deleted_ingredient.updated_at
        FROM deleted_ingredient
        LEFT JOIN unit ON deleted_ingredient.unit_id = unit.id
        "#,
        id,
    )
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(ingredient)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::unit::UnitEntry;

    fn new_ingredient(unit: Option<UnitEntry>) -> NewIngredient {
        NewIngredient {
            household_id: None,
            name: "Eggs".to_string(),
            purchase_size: None,
            unit,
            shopping_category_id: None,
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn a_bulk_edit_changes_only_the_fields_it_sets(pool: PgPool) {
        let mut ids = Vec::new();
        for name in ["Flour", "Sugar"] {
            let ingredient = create_with_unit(
                &pool,
                NewIngredient {
                    household_id: None,
                    name: name.to_string(),
                    purchase_size: Some(500),
                    unit: Some(UnitEntry::New("g".to_string())),
                    shopping_category_id: None,
                },
            )
            .await
            .unwrap();
            ids.push(ingredient.id);
        }
        let baking = crate::features::shopping_category::repository::create(&pool, "Baking")
            .await
            .unwrap();

        let changed = update_many(
            &pool,
            &ids,
            BulkIngredientUpdate {
                unit_id: FieldChange::Set(None),
                purchase_size: FieldChange::Keep,
                shopping_category_id: FieldChange::Set(Some(baking.id)),
            },
        )
        .await
        .unwrap();

        assert_eq!(changed, 2);
        for id in ids {
            let ingredient = get(&pool, id).await.unwrap();
            assert_eq!(ingredient.unit_id, None);
            assert_eq!(ingredient.purchase_size, Some(500));
            assert_eq!(ingredient.shopping_category_id, Some(baking.id));
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn an_ingredient_can_be_saved_without_a_unit(pool: PgPool) {
        let created = create_with_unit(&pool, new_ingredient(None)).await.unwrap();
        assert_eq!((created.unit.as_str(), created.unit_id), ("", None));
        assert_eq!(get(&pool, created.id).await.unwrap(), created);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn a_blank_unit_name_means_no_unit_and_creates_none(pool: PgPool) {
        let blank = Some(UnitEntry::New("  ".to_string()));
        let created = create_with_unit(&pool, new_ingredient(blank))
            .await
            .unwrap();
        assert_eq!(created.unit_id, None);
        let units = sqlx::query_scalar!("SELECT count(*) AS \"n!\" FROM unit")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(units, 0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn a_unit_can_be_added_then_removed(pool: PgPool) {
        let created = create_with_unit(&pool, new_ingredient(None)).await.unwrap();
        let with_unit = UpdateIngredient {
            name: "Eggs".to_string(),
            purchase_size: None,
            unit: Some(UnitEntry::New("each".to_string())),
            shopping_category_id: None,
        };
        let updated = update(&pool, created.id, with_unit).await.unwrap();
        assert_eq!(updated.unit, "each");
        let without = UpdateIngredient {
            name: "Eggs".to_string(),
            purchase_size: None,
            unit: None,
            shopping_category_id: None,
        };
        let updated = update(&pool, created.id, without).await.unwrap();
        assert_eq!((updated.unit.as_str(), updated.unit_id), ("", None));
    }
}
