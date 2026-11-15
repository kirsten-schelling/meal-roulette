use crate::features::{
    cuisine, ingredient, ingredient_alias, protein, recipe_ingredient, recipe_tag, tag,
};
use domain::{
    cuisine::{CuisineEntry, NewCuisine},
    protein::{NewProtein, ProteinEntry},
    recipe::{
        InsertRecipe, NewRecipe, Recipe, RecipeDetail, RecipeOverview, RecipeWithIngredients,
        UpdateRecipe,
    },
    recipe_ingredient::{NewRecipeIngredient, RecipeIngredient},
    tag::{TagEntry, TagRef},
};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

struct OverviewRow {
    id: Uuid,
    household_id: Option<Uuid>,
    title: String,
    difficulty: i16,
    time_mins: i16,
    servings: i16,
    healthiness: Option<i16>,
    min_gap_weeks: Option<i16>,
    max_gap_weeks: Option<i16>,
    main_protein_id: Option<Uuid>,
    cuisine_id: Option<Uuid>,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
    cuisine: Option<String>,
    protein: Option<String>,
    tag_ids: Vec<Uuid>,
    tag_names: Vec<String>,
}

impl From<OverviewRow> for RecipeOverview {
    fn from(row: OverviewRow) -> Self {
        RecipeOverview {
            recipe: Recipe {
                id: row.id,
                household_id: row.household_id,
                title: row.title,
                difficulty: row.difficulty,
                time_mins: row.time_mins,
                servings: row.servings,
                healthiness: row.healthiness,
                min_gap_weeks: row.min_gap_weeks,
                max_gap_weeks: row.max_gap_weeks,
                main_protein_id: row.main_protein_id,
                cuisine_id: row.cuisine_id,
                created_at: row.created_at,
                updated_at: row.updated_at,
            },
            cuisine: row.cuisine,
            protein: row.protein,
            tags: row
                .tag_ids
                .into_iter()
                .zip(row.tag_names)
                .map(|(id, name)| TagRef { id, name })
                .collect(),
        }
    }
}

/// Every recipe, or just `only`, with its cuisine, protein and tags named.
async fn overviews(pool: &PgPool, only: Option<Uuid>) -> Result<Vec<RecipeOverview>, sqlx::Error> {
    let rows = sqlx::query_as!(
        OverviewRow,
        r#"
        SELECT recipe.id, recipe.household_id, recipe.title, recipe.difficulty,
               recipe.time_mins, recipe.servings, recipe.healthiness,
               recipe.min_gap_weeks, recipe.max_gap_weeks,
               recipe.main_protein_id, recipe.cuisine_id, recipe.created_at, recipe.updated_at,
               cuisine.name AS "cuisine?", protein.name AS "protein?",
               COALESCE(array_agg(tag.id ORDER BY tag.name, tag.id)
                        FILTER (WHERE tag.id IS NOT NULL), '{}') AS "tag_ids!",
               COALESCE(array_agg(tag.name ORDER BY tag.name, tag.id)
                        FILTER (WHERE tag.id IS NOT NULL), '{}') AS "tag_names!"
        FROM recipe
        LEFT JOIN cuisine ON cuisine.id = recipe.cuisine_id
        LEFT JOIN protein ON protein.id = recipe.main_protein_id
        LEFT JOIN recipe_tag ON recipe_tag.recipe_id = recipe.id
        LEFT JOIN tag ON tag.id = recipe_tag.tag_id
        WHERE $1::uuid IS NULL OR recipe.id = $1
        GROUP BY recipe.id, cuisine.name, protein.name
        ORDER BY recipe.created_at DESC
        "#,
        only,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn list_overview(pool: &PgPool) -> Result<Vec<RecipeOverview>, sqlx::Error> {
    overviews(pool, None).await
}

pub async fn get_detail(pool: &PgPool, id: Uuid) -> Result<RecipeDetail, sqlx::Error> {
    let overview = overviews(pool, Some(id))
        .await?
        .pop()
        .ok_or(sqlx::Error::RowNotFound)?;
    let ingredients = recipe_ingredient::repository::list_for_recipe(pool, id).await?;
    Ok(RecipeDetail {
        overview,
        ingredients,
    })
}

/// Inline-created cuisine/protein win; otherwise the id the client picked from the catalog.
/// A recipe may have no cuisine.
async fn resolve_lookups(
    conn: &mut PgConnection,
    cuisine: Option<CuisineEntry>,
    protein: ProteinEntry,
) -> Result<(Option<Uuid>, Uuid), sqlx::Error> {
    let cuisine_id = match cuisine {
        None => None,
        Some(CuisineEntry::Existing(id)) => Some(id),
        Some(CuisineEntry::New(name)) => Some(
            cuisine::repository::create(
                &mut *conn,
                NewCuisine {
                    household_id: None,
                    name: name.trim().to_string(),
                },
            )
            .await?
            .id,
        ),
    };
    let protein_id = match protein {
        ProteinEntry::Existing(id) => id,
        ProteinEntry::New(name) => {
            protein::repository::create(
                &mut *conn,
                NewProtein {
                    household_id: None,
                    name: name.trim().to_string(),
                },
            )
            .await?
            .id
        }
    };
    Ok((cuisine_id, protein_id))
}

async fn set_tags(
    conn: &mut PgConnection,
    recipe_id: Uuid,
    entries: Vec<TagEntry>,
) -> Result<(), sqlx::Error> {
    let mut tag_ids = Vec::with_capacity(entries.len());
    for entry in entries {
        let id = tag::repository::resolve(&mut *conn, entry).await?;
        if !tag_ids.contains(&id) {
            tag_ids.push(id);
        }
    }
    recipe_tag::repository::replace_for_recipe(conn, recipe_id, &tag_ids).await?;
    Ok(())
}

pub async fn list(pool: &PgPool) -> Result<Vec<Recipe>, sqlx::Error> {
    let recipes = sqlx::query_as!(
        Recipe,
        r#"
        SELECT id, household_id, title, difficulty, time_mins, servings,
               healthiness, min_gap_weeks, max_gap_weeks, main_protein_id, cuisine_id,
               created_at, updated_at
        FROM recipe
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(recipes)
}

pub async fn create<'e, E>(executor: E, insert: InsertRecipe) -> Result<Recipe, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    let recipe = sqlx::query_as!(
        Recipe,
        r#"
        INSERT INTO recipe
            (household_id, title, difficulty, time_mins, servings,
             healthiness, min_gap_weeks, max_gap_weeks, main_protein_id, cuisine_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, household_id, title, difficulty, time_mins, servings,
                  healthiness, min_gap_weeks, max_gap_weeks, main_protein_id, cuisine_id,
                  created_at, updated_at
        "#,
        insert.household_id,
        insert.title,
        insert.difficulty,
        insert.time_mins,
        insert.servings,
        insert.healthiness,
        insert.min_gap_weeks,
        insert.max_gap_weeks,
        insert.main_protein_id,
        insert.cuisine_id,
    )
    .fetch_one(executor)
    .await?;
    Ok(recipe)
}

pub async fn create_with_ingredients(
    pool: &PgPool,
    mut new_recipe: NewRecipe,
) -> Result<RecipeWithIngredients, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let recipe_ingredients = std::mem::take(&mut new_recipe.ingredients);
    let tags = std::mem::take(&mut new_recipe.tags);
    let (cuisine_id, main_protein_id) =
        resolve_lookups(&mut tx, new_recipe.cuisine, new_recipe.protein).await?;

    let recipe = create(
        &mut *tx,
        InsertRecipe {
            household_id: None,
            title: new_recipe.title.trim().to_string(),
            difficulty: new_recipe.difficulty,
            time_mins: new_recipe.time_mins,
            servings: new_recipe.servings,
            healthiness: new_recipe.healthiness,
            min_gap_weeks: new_recipe.min_gap_weeks,
            max_gap_weeks: new_recipe.max_gap_weeks,
            main_protein_id: Some(main_protein_id),
            cuisine_id,
        },
    )
    .await?;

    set_tags(&mut tx, recipe.id, tags).await?;

    let mut ingredients: Vec<RecipeIngredient> = Vec::new();
    for entry in recipe_ingredients {
        let ingredient_id = ingredient::repository::resolve(&mut tx, entry.ingredient).await?;
        if let Some(phrase) = entry.import_phrase.as_deref() {
            ingredient_alias::repository::remember(&mut tx, phrase, ingredient_id).await?;
        }
        let ingredient = recipe_ingredient::repository::create(
            &mut *tx,
            recipe.id,
            NewRecipeIngredient {
                ingredient_id,
                qty: entry.qty,
                note: entry.note,
                is_optional: entry.is_optional,
            },
        )
        .await?;
        ingredients.push(ingredient);
    }

    tx.commit().await?;
    Ok(RecipeWithIngredients {
        ingredients: Some(ingredients),
        recipe,
    })
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Recipe, sqlx::Error> {
    let recipe = sqlx::query_as!(
        Recipe,
        r#"
        SELECT id, household_id, title, difficulty, time_mins, servings,
               healthiness, min_gap_weeks, max_gap_weeks, main_protein_id, cuisine_id,
               created_at, updated_at
        FROM recipe
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(recipe)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    update_recipe: UpdateRecipe,
) -> Result<Recipe, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let (cuisine_id, protein_id) =
        resolve_lookups(&mut tx, update_recipe.cuisine, update_recipe.protein).await?;
    let recipe = sqlx::query_as!(
        Recipe,
        r#"
        UPDATE recipe
        SET title = $1,
            difficulty = $2,
            time_mins = $3,
            servings = $4,
            healthiness = $5,
            min_gap_weeks = $6,
            max_gap_weeks = $7,
            main_protein_id = $8,
            cuisine_id = $9
        WHERE id = $10
        RETURNING id, household_id, title, difficulty, time_mins, servings,
                  healthiness, min_gap_weeks, max_gap_weeks, main_protein_id, cuisine_id,
                  created_at, updated_at
        "#,
        update_recipe.title.trim(),
        update_recipe.difficulty,
        update_recipe.time_mins,
        update_recipe.servings,
        update_recipe.healthiness,
        update_recipe.min_gap_weeks,
        update_recipe.max_gap_weeks,
        protein_id,
        cuisine_id,
        id,
    )
    .fetch_one(&mut *tx)
    .await?;
    set_tags(&mut tx, id, update_recipe.tags).await?;
    tx.commit().await?;
    Ok(recipe)
}

/// A new recipe under `title` with the same details, tags and ingredient lines as `id`.
pub async fn copy(pool: &PgPool, id: Uuid, title: &str) -> Result<Recipe, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let recipe = sqlx::query_as!(
        Recipe,
        r#"
        INSERT INTO recipe
            (household_id, title, difficulty, time_mins, servings,
             healthiness, min_gap_weeks, max_gap_weeks, main_protein_id, cuisine_id)
        SELECT household_id, $2, difficulty, time_mins, servings,
               healthiness, min_gap_weeks, max_gap_weeks, main_protein_id, cuisine_id
        FROM recipe
        WHERE id = $1
        RETURNING id, household_id, title, difficulty, time_mins, servings,
                  healthiness, min_gap_weeks, max_gap_weeks, main_protein_id, cuisine_id,
                  created_at, updated_at
        "#,
        id,
        title.trim(),
    )
    .fetch_one(&mut *tx)
    .await?;
    recipe_tag::repository::copy_tags(&mut *tx, id, recipe.id).await?;
    recipe_ingredient::repository::copy_lines(&mut *tx, id, recipe.id).await?;
    tx.commit().await?;
    Ok(recipe)
}

/// Ingredient lines and tags go with the recipe; meal plans and the cooking log keep it, so
/// the database refuses the delete while either still points at it.
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<Recipe, sqlx::Error> {
    sqlx::query_as!(
        Recipe,
        r#"
        DELETE FROM recipe
        WHERE id = $1
        RETURNING id, household_id, title, difficulty, time_mins, servings,
                  healthiness, min_gap_weeks, max_gap_weeks, main_protein_id, cuisine_id,
                  created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await
}

pub async fn list_by_cuisine(pool: &PgPool, cuisine_id: Uuid) -> Result<Vec<Recipe>, sqlx::Error> {
    let recipes = sqlx::query_as!(
        Recipe,
        r#"
        SELECT id, household_id, title, difficulty, time_mins, servings,
               healthiness, min_gap_weeks, max_gap_weeks, main_protein_id, cuisine_id,
               created_at, updated_at
        FROM recipe
        WHERE cuisine_id = $1
        "#,
        cuisine_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(recipes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;

    fn new_recipe(min_gap_weeks: Option<i16>, max_gap_weeks: Option<i16>) -> NewRecipe {
        NewRecipe {
            household_id: None,
            title: "Pasta".to_string(),
            difficulty: 2,
            time_mins: 20,
            servings: 2,
            healthiness: None,
            min_gap_weeks,
            max_gap_weeks,
            protein: ProteinEntry::New("Beef".to_string()),
            cuisine: None,
            ingredients: Vec::new(),
            tags: Vec::new(),
        }
    }

    fn update_to(
        recipe: &Recipe,
        min_gap_weeks: Option<i16>,
        max_gap_weeks: Option<i16>,
    ) -> UpdateRecipe {
        UpdateRecipe {
            title: "Pasta".to_string(),
            difficulty: 2,
            time_mins: 20,
            servings: 2,
            healthiness: None,
            min_gap_weeks,
            max_gap_weeks,
            protein: ProteinEntry::Existing(recipe.main_protein_id.unwrap()),
            cuisine: None,
            tags: Vec::new(),
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn gaps_survive_create_update_and_copy(pool: PgPool) {
        let created = create_with_ingredients(&pool, new_recipe(Some(2), Some(6)))
            .await
            .unwrap()
            .recipe;
        assert_eq!(
            (created.min_gap_weeks, created.max_gap_weeks),
            (Some(2), Some(6))
        );

        let updated = update(&pool, created.id, update_to(&created, None, Some(12)))
            .await
            .unwrap();
        assert_eq!(
            (updated.min_gap_weeks, updated.max_gap_weeks),
            (None, Some(12))
        );

        let copied = copy(&pool, created.id, "Pasta again").await.unwrap();
        assert_eq!(
            (copied.min_gap_weeks, copied.max_gap_weeks),
            (None, Some(12))
        );
        let read = get(&pool, copied.id).await.unwrap();
        assert_eq!((read.min_gap_weeks, read.max_gap_weeks), (None, Some(12)));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn a_shortest_gap_longer_than_the_longest_is_refused_until_it_is_fixed(pool: PgPool) {
        let created = create_with_ingredients(&pool, new_recipe(None, None))
            .await
            .unwrap()
            .recipe;

        let refused = update(&pool, created.id, update_to(&created, Some(8), Some(2)))
            .await
            .unwrap_err();
        assert_eq!(
            AppError::from(refused).to_string(),
            "The shortest gap can't be longer than the longest."
        );

        let fixed = update(&pool, created.id, update_to(&created, Some(2), Some(8)))
            .await
            .unwrap();
        assert_eq!(
            (fixed.min_gap_weeks, fixed.max_gap_weeks),
            (Some(2), Some(8))
        );
    }
}
