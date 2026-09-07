use crate::features::{cuisine, ingredient, protein, recipe_ingredient, unit};
use domain::{
    cuisine::{CuisineEntry, NewCuisine},
    ingredient::{IngredientEntry, InsertIngredient},
    protein::{NewProtein, ProteinEntry},
    recipe::{InsertRecipe, NewRecipe, Recipe, RecipeWithIngredients, UpdateRecipe},
    recipe_ingredient::{NewRecipeIngredient, RecipeIngredient},
};
use sqlx::PgPool;
use uuid::Uuid;

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

    // Inline-created cuisine/protein win; otherwise fall back to the id the client
    // picked from the catalog.
    let cuisine_id = match new_recipe.cuisine {
        CuisineEntry::Existing(id) => id,
        CuisineEntry::New(name) => {
            cuisine::repository::create(
                &mut *tx,
                NewCuisine {
                    household_id: None,
                    name,
                },
            )
            .await?
            .id
        }
    };
    let main_protein_id = match new_recipe.protein {
        ProteinEntry::Existing(id) => id,
        ProteinEntry::New(name) => {
            protein::repository::create(
                &mut *tx,
                NewProtein {
                    household_id: None,
                    name,
                },
            )
            .await?
            .id
        }
    };

    let recipe = create(
        &mut *tx,
        InsertRecipe {
            household_id: None,
            title: new_recipe.title,
            difficulty: new_recipe.difficulty,
            time_mins: new_recipe.time_mins,
            servings: new_recipe.servings,
            healthiness: new_recipe.healthiness,
            main_protein_id: Some(main_protein_id),
            cuisine_id: Some(cuisine_id),
        },
    )
    .await?;

    let mut ingredients: Vec<RecipeIngredient> = Vec::new();
    for entry in recipe_ingredients {
        let ingredient_id = match entry.ingredient {
            IngredientEntry::Existing(id) => id,
            IngredientEntry::New {
                name,
                purchase_size,
                unit,
            } => {
                // Resolve the unit first so its borrow of `tx` ends before the
                // ingredient insert borrows `tx` again.
                let unit_id = unit::repository::resolve(&mut *tx, unit).await?;
                ingredient::repository::create(
                    &mut *tx,
                    InsertIngredient {
                        household_id: None,
                        name,
                        purchase_size,
                        unit_id,
                    },
                )
                .await?
                .id
            }
        };
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
    let recipe = sqlx::query_as!(
        Recipe,
        r#"
        UPDATE recipe
        SET title = COALESCE($1, title),
            difficulty = COALESCE($2, difficulty),
            time_mins = COALESCE($3, time_mins),
            servings = COALESCE($4, servings),
            healthiness = COALESCE($5, healthiness),
            main_protein_id = COALESCE($6, main_protein_id),
            cuisine_id = COALESCE($7, cuisine_id)
        WHERE id = $8
        RETURNING id, household_id, title, difficulty, time_mins, servings,
                  healthiness, main_protein_id, cuisine_id, created_at, updated_at
        "#,
        update_recipe.title,
        update_recipe.difficulty,
        update_recipe.time_mins,
        update_recipe.servings,
        update_recipe.healthiness,
        update_recipe.main_protein_id,
        update_recipe.cuisine_id,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(recipe)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<Recipe, sqlx::Error> {
    let recipe = sqlx::query_as!(
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
    .await?;
    Ok(recipe)
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
