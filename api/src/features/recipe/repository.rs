use crate::features::{cuisine, ingredient, protein, recipe_ingredient, recipe_tag, tag};
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
async fn resolve_lookups(
    conn: &mut PgConnection,
    cuisine: CuisineEntry,
    protein: ProteinEntry,
) -> Result<(Uuid, Uuid), sqlx::Error> {
    let cuisine_id = match cuisine {
        CuisineEntry::Existing(id) => id,
        CuisineEntry::New(name) => {
            cuisine::repository::create(
                &mut *conn,
                NewCuisine {
                    household_id: None,
                    name: name.trim().to_string(),
                },
            )
            .await?
            .id
        }
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
               healthiness, main_protein_id, cuisine_id, created_at, updated_at
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
             healthiness, main_protein_id, cuisine_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, household_id, title, difficulty, time_mins, servings,
                  healthiness, main_protein_id, cuisine_id, created_at, updated_at
        "#,
        insert.household_id,
        insert.title,
        insert.difficulty,
        insert.time_mins,
        insert.servings,
        insert.healthiness,
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
            main_protein_id: Some(main_protein_id),
            cuisine_id: Some(cuisine_id),
        },
    )
    .await?;

    set_tags(&mut tx, recipe.id, tags).await?;

    let mut ingredients: Vec<RecipeIngredient> = Vec::new();
    for entry in recipe_ingredients {
        let ingredient_id = ingredient::repository::resolve(&mut tx, entry.ingredient).await?;
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
               healthiness, main_protein_id, cuisine_id, created_at, updated_at
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
            main_protein_id = $6,
            cuisine_id = $7
        WHERE id = $8
        RETURNING id, household_id, title, difficulty, time_mins, servings,
                  healthiness, main_protein_id, cuisine_id, created_at, updated_at
        "#,
        update_recipe.title.trim(),
        update_recipe.difficulty,
        update_recipe.time_mins,
        update_recipe.servings,
        update_recipe.healthiness,
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

/// Ingredient lines and tags go with the recipe; meal plans and the cooking log keep it, so
/// the database refuses the delete while either still points at it.
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<Recipe, sqlx::Error> {
    sqlx::query_as!(
        Recipe,
        r#"
        DELETE FROM recipe
        WHERE id = $1
        RETURNING id, household_id, title, difficulty, time_mins, servings,
                  healthiness, main_protein_id, cuisine_id, created_at, updated_at
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
               healthiness, main_protein_id, cuisine_id, created_at, updated_at
        FROM recipe
        WHERE cuisine_id = $1
        "#,
        cuisine_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(recipes)
}
