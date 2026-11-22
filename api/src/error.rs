use axum::{http::StatusCode, response::Response};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Db(sqlx::Error),
    NotFound(String),
    Conflict(String),
    BadRequest(String),
    /// Well-formed, but cannot be carried out against the current data.
    Unprocessable(String),
    Internal(String),
}

impl AppError {
    pub fn status(&self) -> StatusCode {
        match self {
            AppError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unprocessable(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Db(error) => write!(f, "{error}"),
            AppError::NotFound(message)
            | AppError::Conflict(message)
            | AppError::BadRequest(message)
            | AppError::Unprocessable(message)
            | AppError::Internal(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for AppError {}

/// sqlx only recognises 23503, the code for a `NO ACTION` refusal. This schema's foreign keys
/// refuse deletes with `ON DELETE RESTRICT`, which Postgres reports as 23001 instead.
const RESTRICT_VIOLATION: &str = "23001";

pub fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(|db| db.is_unique_violation())
}

/// What a refused write means to the person who made it, by the constraint that refused it.
/// The foreign keys listed only ever refuse a delete (they are all `ON DELETE RESTRICT`).
fn constraint_message(constraint: &str) -> Option<AppError> {
    let conflict = |message: &str| Some(AppError::Conflict(message.to_string()));
    let invalid = |message: &str| Some(AppError::BadRequest(message.to_string()));
    match constraint {
        "uq_recipe_household_title" => conflict("You already have a recipe with that name."),
        "uq_recipe_ingredient" => conflict("That ingredient is already in the recipe."),
        "uq_recipe_tag" => conflict("The recipe already has that tag."),
        "uq_ingredient_household_name" => conflict("An ingredient with that name already exists."),
        "uq_unit_household_name" => conflict("A unit with that name already exists."),
        "uq_cuisine_household_name" => conflict("A cuisine with that name already exists."),
        "uq_protein_household_name" => conflict("A protein with that name already exists."),
        "uq_tag_household_name" => conflict("A tag with that name already exists."),
        "uq_roll_preset_household_name" => conflict("A preset with that name already exists."),
        "uq_planned_meal" => conflict("Another dinner in the plan is already on that date."),
        "uq_planned_meal_recipe" => conflict("That recipe is already on another day of the plan."),
        "uq_cook_log_planned_meal" => conflict("That dinner is already in the cooking log."),
        "fk_recipe_cuisine" => conflict("This cuisine is still used by a recipe."),
        "fk_recipe_protein" => conflict("This protein is still used by a recipe."),
        "fk_recipe_tag_tag" => conflict("This tag is still used by a recipe."),
        "fk_ingredient_unit" => conflict("This unit is still used by an ingredient."),
        "fk_recipe_ingredient_ingredient" => conflict("This ingredient is still used by a recipe."),
        "fk_shopping_list_item_ingredient" => {
            conflict("This ingredient is still on a shopping list.")
        }
        "fk_shopping_list_item_unit" => conflict("This unit is still used on a shopping list."),
        "fk_planned_meal_recipe" => conflict(
            "This recipe is on a meal plan. Delete that plan from History first, or swap the recipe out of it.",
        ),
        "fk_cook_log_recipe" => conflict("This recipe is in the cooking log."),
        "uq_shopping_item_name" => conflict("An item with that name already exists."),
        "uq_shopping_category_name" => conflict("A category with that name already exists."),
        "chk_shopping_category_name" => invalid("Give the category a name."),
        "chk_shopping_list_item_kind" | "chk_shopping_item_name" => {
            invalid("Say what to add to the list.")
        }
        "chk_pantry_item_qty_not_negative" => invalid("A pantry count cannot be below 0."),
        "chk_recipe_difficulty_range" => invalid("Difficulty must be between 1 and 5."),
        "chk_recipe_healthiness_range" => invalid("Healthiness must be between 1 and 5."),
        "chk_recipe_min_gap_weeks_range" | "chk_recipe_max_gap_weeks_range" => {
            invalid("A gap must be between 1 and 52 weeks.")
        }
        "chk_recipe_gap_order" => invalid("The shortest gap can't be longer than the longest."),
        "chk_recipe_time_mins_positive" => invalid("Time must be at least 1 minute."),
        "chk_recipe_servings_positive" => invalid("Servings must be at least 1."),
        "chk_recipe_ingredient_qty_positive" => invalid("Each quantity must be more than 0."),
        "chk_ingredient_purchase_size_positive" => invalid("Purchase size must be more than 0."),
        _ => None,
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        if matches!(error, sqlx::Error::RowNotFound) {
            return AppError::NotFound("not found".to_string());
        }
        let Some(db) = error.as_database_error() else {
            return AppError::Db(error);
        };
        let restrict = db.code().as_deref() == Some(RESTRICT_VIOLATION);
        if db.is_unique_violation() || db.is_check_violation() || restrict {
            if let Some(mapped) = db.constraint().and_then(constraint_message) {
                return mapped;
            }
        }
        if db.is_unique_violation() {
            return AppError::Conflict("Already exists".to_string());
        }
        if restrict {
            return AppError::Conflict("This is still in use, so it can't be deleted.".to_string());
        }
        if db.is_foreign_key_violation() {
            return AppError::Conflict(
                "That refers to something that no longer exists.".to_string(),
            );
        }
        if db.is_check_violation() {
            return AppError::BadRequest("One of the values isn't allowed.".to_string());
        }
        AppError::Db(error)
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        eprintln!("[api] {status}: {self}");
        (status, self.to_string()).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_blocking_foreign_keys_have_a_friendly_conflict_message() {
        for constraint in [
            "fk_recipe_cuisine",
            "fk_recipe_protein",
            "fk_ingredient_unit",
            "fk_shopping_list_item_unit",
            "fk_recipe_ingredient_ingredient",
            "fk_shopping_list_item_ingredient",
        ] {
            let error = constraint_message(constraint).expect(constraint);
            assert_eq!(error.status(), StatusCode::CONFLICT, "{constraint}");
            assert!(error.to_string().contains("still"), "{constraint}");
        }
    }
}
