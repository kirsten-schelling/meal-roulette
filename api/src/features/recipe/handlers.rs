use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use super::repository::{create, delete, get, list, update};
use crate::features::cuisine::repository::create as create_cuisine;
use crate::features::protein::repository::create as create_protein;
use crate::features::recipe_ingredient::repository::create as create_recipe_ingredient;
use crate::features::unit::repository::create as create_unit;
use crate::state::AppState;
use crate::{error::AppError, features::ingredient::repository::create as create_ingredient};
use domain::{
    cuisine::{CuisineEntry, NewCuisine},
    ingredient::IngredientEntry,
    protein::{NewProtein, ProteinEntry},
    recipe::{InsertRecipe, NewRecipe, Recipe, RecipeWithIngredients, UpdateRecipe},
    unit::UnitEntry,
};
use domain::{
    ingredient::NewIngredient,
    recipe_ingredient::{NewRecipeIngredient, RecipeIngredient},
    unit::NewUnit,
};

pub async fn get_recipes(State(state): State<AppState>) -> Result<Json<Vec<Recipe>>, AppError> {
    let recipes = list(&state.db).await?;
    Ok(Json(recipes))
}

pub async fn get_recipe(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Recipe>, AppError> {
    let recipe = get(&state.db, id).await?;
    Ok(Json(recipe))
}
pub async fn create_recipe(
    State(state): State<AppState>,
    Json(mut new_recipe): Json<NewRecipe>,
) -> Result<Json<RecipeWithIngredients>, AppError> {
    dbg!(&new_recipe);
    let mut tx = state.db.begin().await?;
    let recipe_ingredients = std::mem::take(&mut new_recipe.ingredients);

    // Inline-created cuisine/protein win; otherwise fall back to the id the client
    // picked from the catalog.
    let cuisine_id = match new_recipe.cuisine {
        CuisineEntry::Existing(id) => id,
        CuisineEntry::New(cuisine) => {
            create_cuisine(
                &mut *tx,
                NewCuisine {
                    household_id: None,
                    name: cuisine,
                },
            )
            .await?
            .id
        }
    };
    let main_protein_id = match new_recipe.protein {
        ProteinEntry::Existing(id) => id,
        ProteinEntry::New(protein) => {
            create_protein(
                &mut *tx,
                NewProtein {
                    household_id: None,
                    name: protein,
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
    for ingredient in recipe_ingredients {
        let ingredient_id = match ingredient.ingredient {
            IngredientEntry::Existing(id) => id,
            IngredientEntry::New {
                name,
                purchase_size,
                unit,
            } => {
                // Resolve the unit first so its borrow of `tx` ends before the
                // ingredient insert borrows `tx` again.
                let unit_id = match unit {
                    UnitEntry::Existing(id) => id,
                    UnitEntry::New(unit_name) => {
                        create_unit(
                            &mut *tx,
                            NewUnit {
                                household_id: None,
                                name: unit_name,
                                description: None,
                            },
                        )
                        .await?
                        .id
                    }
                };
                create_ingredient(
                    &mut *tx,
                    NewIngredient {
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
        let ingredient = create_recipe_ingredient(
            &mut *tx,
            recipe.id,
            NewRecipeIngredient {
                ingredient_id,
                qty: ingredient.qty,
                note: ingredient.note,
                is_optional: ingredient.is_optional,
            },
        )
        .await?;
        ingredients.push(ingredient);
    }

    tx.commit().await?;
    dbg!(&recipe);
    Ok(Json(RecipeWithIngredients {
        ingredients: Some(ingredients),
        recipe,
    }))
}

pub async fn update_recipe(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update_recipe): Json<UpdateRecipe>,
) -> Result<Json<Recipe>, AppError> {
    let recipe = update(&state.db, id, update_recipe).await?;
    Ok(Json(recipe))
}

pub async fn delete_recipe(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Recipe>, AppError> {
    let recipe = delete(&state.db, id).await?;
    Ok(Json(recipe))
}
