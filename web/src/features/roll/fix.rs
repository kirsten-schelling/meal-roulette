//! The ways out of a roll that its rules make hard or impossible. Each fix is
//! applied in one place, the roll page's `apply_fix`.

use crate::components::ui::ComboOption;
use crate::features::recipe::components::recipe_seed::RecipeSeed;
use crate::features::roll::settings::RollSettings;
use domain::ingredient::Ingredient;
use domain::roll::{Bound, Importance, PreviewMeal, RuleFamily, UnmetRule};

#[derive(Clone, PartialEq, Debug)]
pub enum Fix {
    /// Turn the named required rules into preferences.
    MakeNiceToHave(Vec<(RuleFamily, usize)>),
    /// Drop the cook-time and difficulty caps every dinner follows.
    ClearCaps,
    /// Roll this many dinners instead.
    RollDays(u32),
    /// Open the add-recipe form, filled in from the rule that wants it.
    AddRecipe(RecipeSeed),
}

impl Fix {
    pub fn label(&self) -> String {
        match self {
            Fix::MakeNiceToHave(_) => "Turn off Must for all rules".to_string(),
            Fix::ClearCaps => "Remove the every-dinner caps".to_string(),
            Fix::RollDays(1) => "Roll 1 dinner".to_string(),
            Fix::RollDays(days) => format!("Roll {days} dinners"),
            Fix::AddRecipe(_) => "Add a recipe".to_string(),
        }
    }
}

/// A preference the roll fell short of, in words, with the fix that could
/// meet it.
#[derive(Clone, PartialEq, Debug)]
pub struct UnmetNote {
    pub sentence: String,
    pub fix: Option<Fix>,
}

impl RollSettings {
    /// What to offer when a roll could not satisfy its required rules.
    pub fn failure_fixes(&self) -> Vec<Fix> {
        let must: Vec<(RuleFamily, usize)> = self
            .lines()
            .iter()
            .filter(|l| l.importance == Importance::Required)
            .map(|l| (l.family, l.index))
            .collect();
        let mut fixes = Vec::new();
        if !must.is_empty() {
            fixes.push(Fix::MakeNiceToHave(must));
        }
        fixes.extend(self.clear_caps());
        fixes.push(Fix::AddRecipe(RecipeSeed::default()));
        fixes
    }

    /// What to offer when only `fitting` recipes fit the rules but more
    /// dinners are being rolled.
    pub fn shortage_fixes(&self, fitting: u32) -> Vec<Fix> {
        let mut fixes = Vec::new();
        if fitting > 0 {
            fixes.push(Fix::RollDays(fitting));
        }
        fixes.extend(self.clear_caps());
        fixes.push(Fix::AddRecipe(RecipeSeed::default()));
        fixes
    }

    /// The preferences a roll fell short of, each in a sentence, then each
    /// dinner cooked more recently than its gap allows. A shortfall of a
    /// minimum can be met by a new recipe; an excess of a maximum only by
    /// swapping a dinner.
    pub fn unmet_notes(
        &self,
        unmet: &[UnmetRule],
        meals: &[PreviewMeal],
        ingredients: &[Ingredient],
    ) -> Vec<UnmetNote> {
        let sentences = self.unmet_sentences(unmet);
        let mut notes: Vec<UnmetNote> = unmet
            .iter()
            .zip(sentences)
            .map(|(u, sentence)| UnmetNote {
                sentence,
                fix: (u.bound == Bound::AtLeast)
                    .then(|| Fix::AddRecipe(self.recipe_seed(u.family, u.index, ingredients))),
            })
            .collect();
        notes.extend(
            meals
                .iter()
                .filter(|meal| meal.within_min_gap)
                .map(|meal| UnmetNote {
                    sentence: format!(
                        "{} was cooked recently, inside the gap set for it.",
                        meal.title
                    ),
                    fix: None,
                }),
        );
        notes
    }

    fn clear_caps(&self) -> Option<Fix> {
        (self.every_dinner.count() > 0).then_some(Fix::ClearCaps)
    }

    fn recipe_seed(
        &self,
        family: RuleFamily,
        index: usize,
        ingredients: &[Ingredient],
    ) -> RecipeSeed {
        let mut seed = RecipeSeed::default();
        match family {
            RuleFamily::Cuisine => {
                seed.cuisine = self
                    .cuisines
                    .get(index)
                    .map(|rule| ComboOption::new(rule.id, rule.name.clone()));
            }
            RuleFamily::Protein => {
                seed.protein = self
                    .proteins
                    .get(index)
                    .map(|rule| ComboOption::new(rule.id, rule.name.clone()));
            }
            RuleFamily::Ingredient => {
                seed.ingredient = self.ingredients.get(index).and_then(|rule| {
                    let ingredient = ingredients.iter().find(|i| i.id == rule.id)?;
                    Some(ComboOption::new(rule.id, rule.name.clone()).with_hint(&ingredient.unit))
                });
            }
            RuleFamily::Difficulty => {
                seed.difficulty = self
                    .difficulties
                    .get(index)
                    .map(|rule| rule.band.levels().0);
            }
            RuleFamily::Tag => {}
        }
        seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::roll::settings::{EveryDinner, LookupRule};
    use domain::roll::{DifficultyBand, DifficultyRule};
    use uuid::Uuid;

    fn lookup(name: &str, importance: Importance) -> LookupRule {
        LookupRule {
            id: Uuid::now_v7(),
            name: name.to_string(),
            times: 2,
            bound: Bound::AtLeast,
            importance,
        }
    }

    fn unmet(family: RuleFamily, bound: Bound) -> UnmetRule {
        UnmetRule {
            family,
            index: 0,
            bound,
            times: 2,
            got: 1,
        }
    }

    #[test]
    fn a_failed_roll_offers_to_loosen_every_must_rule_and_to_add_a_recipe() {
        let settings = RollSettings {
            proteins: vec![
                lookup("Beef", Importance::Required),
                lookup("Fish", Importance::Medium),
            ],
            ..RollSettings::default()
        };
        assert_eq!(
            settings.failure_fixes(),
            [
                Fix::MakeNiceToHave(vec![(RuleFamily::Protein, 0)]),
                Fix::AddRecipe(RecipeSeed::default()),
            ]
        );
    }

    #[test]
    fn caps_are_offered_for_removal_only_when_set() {
        let mut settings = RollSettings::default();
        assert_eq!(
            settings.shortage_fixes(0),
            [Fix::AddRecipe(RecipeSeed::default())]
        );
        settings.every_dinner = EveryDinner {
            max_time_mins: Some(20),
            max_difficulty: None,
        };
        assert_eq!(
            settings.shortage_fixes(3),
            [
                Fix::RollDays(3),
                Fix::ClearCaps,
                Fix::AddRecipe(RecipeSeed::default()),
            ]
        );
    }

    #[test]
    fn a_missed_minimum_offers_a_recipe_seeded_from_its_rule() {
        let beef = lookup("Beef", Importance::Medium);
        let settings = RollSettings {
            proteins: vec![beef.clone()],
            ..RollSettings::default()
        };
        let notes = settings.unmet_notes(&[unmet(RuleFamily::Protein, Bound::AtLeast)], &[], &[]);
        let Some(Fix::AddRecipe(seed)) = &notes[0].fix else {
            panic!("expected an add-recipe fix");
        };
        assert_eq!(seed.protein, Some(ComboOption::new(beef.id, "Beef")));
        assert_eq!(seed.cuisine, None);
    }

    #[test]
    fn an_excess_over_a_maximum_offers_no_new_recipe() {
        let settings = RollSettings {
            proteins: vec![lookup("Beef", Importance::Medium)],
            ..RollSettings::default()
        };
        let notes = settings.unmet_notes(&[unmet(RuleFamily::Protein, Bound::AtMost)], &[], &[]);
        assert_eq!(notes[0].fix, None);
    }

    #[test]
    fn a_missed_difficulty_minimum_seeds_the_bands_easiest_level() {
        let settings = RollSettings {
            difficulties: vec![DifficultyRule {
                band: DifficultyBand::Hard,
                times: 2,
                bound: Bound::AtLeast,
                importance: Importance::Medium,
            }],
            ..RollSettings::default()
        };
        let notes =
            settings.unmet_notes(&[unmet(RuleFamily::Difficulty, Bound::AtLeast)], &[], &[]);
        let Some(Fix::AddRecipe(seed)) = &notes[0].fix else {
            panic!("expected an add-recipe fix");
        };
        assert_eq!(seed.difficulty, Some(4));
    }

    #[test]
    fn a_dinner_cooked_inside_its_gap_gets_a_note_without_a_fix() {
        let meal = |title: &str, within_min_gap| PreviewMeal {
            recipe_id: Uuid::now_v7(),
            title: title.to_string(),
            difficulty: 1,
            time_mins: 20,
            cuisine: None,
            protein: None,
            within_min_gap,
        };
        let meals = [meal("Pasta", true), meal("Curry", false)];
        let notes = RollSettings::default().unmet_notes(&[], &meals, &[]);
        assert_eq!(notes.len(), 1);
        assert!(notes[0].sentence.starts_with("Pasta"));
        assert_eq!(notes[0].fix, None);
    }
}
