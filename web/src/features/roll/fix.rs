//! The ways out of a roll that its rules make hard or impossible. Each fix is
//! applied in one place, the roll page's `apply_fix`.

use crate::components::ui::ComboOption;
use crate::features::recipe::components::recipe_seed::RecipeSeed;
use crate::features::recipe::components::tag_picker::TagChoice;
use crate::features::roll::settings::RollSettings;
use domain::ingredient::Ingredient;
use domain::roll::{Importance, RuleFamily, UnmetRule};

#[derive(Clone, PartialEq, Debug)]
pub enum Fix {
    /// Turn the named required rules into preferences.
    MakeNiceToHave(Vec<(RuleFamily, usize)>),
    /// Drop the cook-time and difficulty caps every dinner follows.
    ClearCaps,
    /// Roll this many dinners instead.
    RollDays(u32),
    /// Open the add-recipe form, filled in from the rule that wants it.
    AddRecipe(Box<RecipeSeed>),
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
        fixes.push(Fix::AddRecipe(Box::default()));
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
        fixes.push(Fix::AddRecipe(Box::default()));
        fixes
    }

    /// The preferences a roll fell short of, each in a sentence. A shortfall
    /// of a minimum (at least, or exactly) can be met by a new recipe; an
    /// excess of a maximum only by swapping a dinner.
    pub fn unmet_notes(&self, unmet: &[UnmetRule], ingredients: &[Ingredient]) -> Vec<UnmetNote> {
        let sentences = self.unmet_sentences(unmet);
        unmet
            .iter()
            .zip(sentences)
            .map(|(u, sentence)| UnmetNote {
                sentence,
                fix: (u.bound.has_floor() && u.got < u.times).then(|| {
                    Fix::AddRecipe(Box::new(self.recipe_seed(u.family, u.index, ingredients)))
                }),
            })
            .collect()
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
            RuleFamily::Tag => {
                seed.tag = self.tags.get(index).map(|rule| TagChoice {
                    id: Some(rule.id),
                    name: rule.name.clone(),
                });
            }
            RuleFamily::Difficulty => {
                seed.difficulty = self
                    .difficulties
                    .get(index)
                    .map(|rule| rule.band.levels().0);
            }
        }
        seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::roll::settings::{EveryDinner, LookupRule};
    use domain::roll::{Bound, DifficultyBand, DifficultyRule};
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
                Fix::AddRecipe(Box::default()),
            ]
        );
    }

    #[test]
    fn caps_are_offered_for_removal_only_when_set() {
        let mut settings = RollSettings::default();
        assert_eq!(settings.shortage_fixes(0), [Fix::AddRecipe(Box::default())]);
        settings.every_dinner = EveryDinner {
            max_time_mins: Some(20),
            max_difficulty: None,
        };
        assert_eq!(
            settings.shortage_fixes(3),
            [
                Fix::RollDays(3),
                Fix::ClearCaps,
                Fix::AddRecipe(Box::default()),
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
        let notes = settings.unmet_notes(&[unmet(RuleFamily::Protein, Bound::AtLeast)], &[]);
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
        let notes = settings.unmet_notes(&[unmet(RuleFamily::Protein, Bound::AtMost)], &[]);
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
        let notes = settings.unmet_notes(&[unmet(RuleFamily::Difficulty, Bound::AtLeast)], &[]);
        let Some(Fix::AddRecipe(seed)) = &notes[0].fix else {
            panic!("expected an add-recipe fix");
        };
        assert_eq!(seed.difficulty, Some(4));
    }

    #[test]
    fn a_missed_tag_minimum_seeds_the_tag() {
        let vegan = lookup("Vegan", Importance::Medium);
        let settings = RollSettings {
            tags: vec![vegan.clone()],
            ..RollSettings::default()
        };
        let notes = settings.unmet_notes(&[unmet(RuleFamily::Tag, Bound::AtLeast)], &[]);
        let Some(Fix::AddRecipe(seed)) = &notes[0].fix else {
            panic!("expected an add-recipe fix");
        };
        assert_eq!(
            seed.tag,
            Some(TagChoice {
                id: Some(vegan.id),
                name: "Vegan".to_string()
            })
        );
        assert_eq!(seed.protein, None);
    }

    #[test]
    fn an_exactly_rule_short_of_its_count_offers_a_recipe() {
        let settings = RollSettings {
            proteins: vec![lookup("Beef", Importance::Medium)],
            ..RollSettings::default()
        };
        let short = unmet(RuleFamily::Protein, Bound::Exactly);
        let notes = settings.unmet_notes(&[short], &[]);
        assert!(matches!(notes[0].fix, Some(Fix::AddRecipe(_))));
    }

    #[test]
    fn an_exactly_rule_over_its_count_offers_no_recipe() {
        let settings = RollSettings {
            proteins: vec![lookup("Beef", Importance::Medium)],
            ..RollSettings::default()
        };
        let over = UnmetRule {
            got: 3,
            ..unmet(RuleFamily::Protein, Bound::Exactly)
        };
        let notes = settings.unmet_notes(&[over], &[]);
        assert_eq!(notes[0].fix, None);
    }

    #[test]
    fn a_failed_roll_offers_to_loosen_required_tag_rules() {
        let settings = RollSettings {
            tags: vec![lookup("Vegan", Importance::Required)],
            ..RollSettings::default()
        };
        assert_eq!(
            settings.failure_fixes()[0],
            Fix::MakeNiceToHave(vec![(RuleFamily::Tag, 0)])
        );
    }

    #[test]
    fn a_missed_tag_rule_is_named_in_its_sentence() {
        let settings = RollSettings {
            tags: vec![lookup("Vegan", Importance::Medium)],
            ..RollSettings::default()
        };
        let notes = settings.unmet_notes(&[unmet(RuleFamily::Tag, Bound::AtLeast)], &[]);
        assert_eq!(notes[0].sentence, "Vegan: wanted at least 2, got 1.");
    }
}
