use crate::features::household;
use domain::cook_log::{CookLog, CookLogEntry, NewCookLog, UpdateCookLog};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool) -> Result<Vec<CookLog>, sqlx::Error> {
    let logs = sqlx::query_as!(
        CookLog,
        r#"
        SELECT id, household_id, recipe_id, planned_meal_id, cooked_on, created_at, updated_at
        FROM cook_log
        ORDER BY cooked_on DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(logs)
}

/// Every cook with its recipe's title, newest first; same-day cooks in the order logged,
/// latest first.
pub async fn list_entries(pool: &PgPool) -> Result<Vec<CookLogEntry>, sqlx::Error> {
    let entries = sqlx::query_as!(
        CookLogEntry,
        r#"
        SELECT cook_log.id, cook_log.recipe_id, recipe.title, cook_log.cooked_on
        FROM cook_log
        JOIN recipe ON recipe.id = cook_log.recipe_id
        ORDER BY cook_log.cooked_on DESC, cook_log.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(entries)
}

/// The cook that fulfilled a planned dinner, if it has been cooked.
pub async fn find_for_planned_meal(
    pool: &PgPool,
    planned_meal_id: Uuid,
) -> Result<Option<CookLog>, sqlx::Error> {
    let log = sqlx::query_as!(
        CookLog,
        r#"
        SELECT id, household_id, recipe_id, planned_meal_id, cooked_on, created_at, updated_at
        FROM cook_log
        WHERE planned_meal_id = $1
        "#,
        planned_meal_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(log)
}

pub struct LastCook {
    pub recipe_id: Uuid,
    pub cooked_on: time::Date,
}

/// The day each recipe was last cooked; a recipe never cooked has no entry.
pub async fn last_cooks(pool: &PgPool) -> Result<Vec<LastCook>, sqlx::Error> {
    let cooks = sqlx::query_as!(
        LastCook,
        r#"
        SELECT recipe_id, MAX(cooked_on) AS "cooked_on!"
        FROM cook_log
        GROUP BY recipe_id
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(cooks)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<CookLog, sqlx::Error> {
    let log = sqlx::query_as!(
        CookLog,
        r#"
        SELECT id, household_id, recipe_id, planned_meal_id, cooked_on, created_at, updated_at
        FROM cook_log
        WHERE id = $1
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(log)
}

pub async fn create(pool: &PgPool, new_log: NewCookLog) -> Result<CookLog, sqlx::Error> {
    let log = sqlx::query_as!(
        CookLog,
        r#"
        INSERT INTO cook_log (household_id, recipe_id, planned_meal_id, cooked_on)
        VALUES ($1, $2, $3, $4)
        RETURNING id, household_id, recipe_id, planned_meal_id, cooked_on, created_at, updated_at
        "#,
        new_log.household_id,
        new_log.recipe_id,
        new_log.planned_meal_id,
        new_log.cooked_on,
    )
    .fetch_one(pool)
    .await?;
    Ok(log)
}

/// Records a cook for the household every unscoped request acts for.
pub async fn log(
    pool: &PgPool,
    recipe_id: Uuid,
    planned_meal_id: Option<Uuid>,
    cooked_on: time::Date,
) -> Result<CookLog, sqlx::Error> {
    let household_id = household::repository::default_household(pool).await?.id;
    create(
        pool,
        NewCookLog {
            household_id,
            recipe_id,
            planned_meal_id,
            cooked_on,
        },
    )
    .await
}

pub async fn update(pool: &PgPool, id: Uuid, log: UpdateCookLog) -> Result<CookLog, sqlx::Error> {
    let log = sqlx::query_as!(
        CookLog,
        r#"
        UPDATE cook_log
        SET recipe_id = COALESCE($1, recipe_id),
            planned_meal_id = COALESCE($2, planned_meal_id),
            cooked_on = COALESCE($3, cooked_on)
        WHERE id = $4
        RETURNING id, household_id, recipe_id, planned_meal_id, cooked_on, created_at, updated_at
        "#,
        log.recipe_id,
        log.planned_meal_id,
        log.cooked_on,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(log)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<CookLog, sqlx::Error> {
    let log = sqlx::query_as!(
        CookLog,
        r#"
        DELETE FROM cook_log
        WHERE id = $1
        RETURNING id, household_id, recipe_id, planned_meal_id, cooked_on, created_at, updated_at
        "#,
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(log)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;
    use time::macros::date;

    async fn recipe_titled(pool: &PgPool, title: &str) -> Uuid {
        sqlx::query_scalar!(
            "INSERT INTO recipe (title, difficulty, time_mins, servings) VALUES ($1, 1, 20, 2) RETURNING id",
            title,
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn planned_meal_of(pool: &PgPool, recipe_id: Uuid) -> Uuid {
        let household_id = household::repository::default_household(pool)
            .await
            .unwrap()
            .id;
        let plan_id = sqlx::query_scalar!(
            "INSERT INTO meal_plan (household_id) VALUES ($1) RETURNING id",
            household_id,
        )
        .fetch_one(pool)
        .await
        .unwrap();
        sqlx::query_scalar!(
            "INSERT INTO planned_meal (meal_plan_id, recipe_id, day_index, scheduled_date)
             VALUES ($1, $2, 1, '2026-11-14')
             RETURNING id",
            plan_id,
            recipe_id,
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn the_log_lists_cooks_newest_first_with_their_titles(pool: PgPool) {
        let pasta = recipe_titled(&pool, "Pasta").await;
        let curry = recipe_titled(&pool, "Curry").await;
        log(&pool, pasta, None, date!(2026 - 11 - 10))
            .await
            .unwrap();
        log(&pool, curry, None, date!(2026 - 11 - 12))
            .await
            .unwrap();
        log(&pool, pasta, None, date!(2026 - 11 - 12))
            .await
            .unwrap();

        let listed: Vec<(String, time::Date)> = list_entries(&pool)
            .await
            .unwrap()
            .into_iter()
            .map(|entry| (entry.title, entry.cooked_on))
            .collect();
        assert_eq!(
            listed,
            vec![
                ("Pasta".to_string(), date!(2026 - 11 - 12)),
                ("Curry".to_string(), date!(2026 - 11 - 12)),
                ("Pasta".to_string(), date!(2026 - 11 - 10)),
            ]
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn a_cooked_dinner_is_found_by_its_plan_slot_until_the_cook_is_undone(pool: PgPool) {
        let pasta = recipe_titled(&pool, "Pasta").await;
        let dinner = planned_meal_of(&pool, pasta).await;
        assert_eq!(find_for_planned_meal(&pool, dinner).await.unwrap(), None);

        let cooked = log(&pool, pasta, Some(dinner), date!(2026 - 11 - 14))
            .await
            .unwrap();
        assert_eq!(
            find_for_planned_meal(&pool, dinner).await.unwrap(),
            Some(cooked.clone())
        );

        delete(&pool, cooked.id).await.unwrap();
        assert_eq!(find_for_planned_meal(&pool, dinner).await.unwrap(), None);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn a_planned_dinner_is_logged_once_but_unplanned_cooks_repeat(pool: PgPool) {
        let pasta = recipe_titled(&pool, "Pasta").await;
        let dinner = planned_meal_of(&pool, pasta).await;
        log(&pool, pasta, Some(dinner), date!(2026 - 11 - 14))
            .await
            .unwrap();

        let again = log(&pool, pasta, Some(dinner), date!(2026 - 11 - 14))
            .await
            .unwrap_err();
        assert_eq!(
            AppError::from(again).to_string(),
            "That dinner is already in the cooking log."
        );

        log(&pool, pasta, None, date!(2026 - 11 - 14))
            .await
            .unwrap();
        log(&pool, pasta, None, date!(2026 - 11 - 14))
            .await
            .unwrap();
        assert_eq!(list_entries(&pool).await.unwrap().len(), 3);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn last_cooks_gives_each_recipe_its_latest_cook(pool: PgPool) {
        let pasta = recipe_titled(&pool, "Pasta").await;
        let curry = recipe_titled(&pool, "Curry").await;
        let never = recipe_titled(&pool, "Stew").await;
        log(&pool, pasta, None, date!(2026 - 11 - 10))
            .await
            .unwrap();
        log(&pool, pasta, None, date!(2026 - 11 - 12))
            .await
            .unwrap();
        log(&pool, curry, None, date!(2026 - 11 - 11))
            .await
            .unwrap();

        let last: std::collections::HashMap<Uuid, time::Date> = last_cooks(&pool)
            .await
            .unwrap()
            .into_iter()
            .map(|cook| (cook.recipe_id, cook.cooked_on))
            .collect();
        assert_eq!(last.get(&pasta), Some(&date!(2026 - 11 - 12)));
        assert_eq!(last.get(&curry), Some(&date!(2026 - 11 - 11)));
        assert_eq!(last.get(&never), None);
    }
}
