use domain::roll::{
    Bound, CountRule, DayLimitPlan, DayLimits, DayLimitsAt, DifficultyBand, DifficultyRule,
    Importance, RollConstraints, RuleFamily, UnmetRule,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Days a fresh roll form asks for, and so the recipes a first week needs.
pub const DEFAULT_NUM_DAYS: usize = 7;

/// The most dinners one roll may ask for; the server refuses more.
pub const MAX_ROLL_DAYS: u32 = 31;

/// A rule added on the page starts as a preference, so a fresh rule cannot be
/// what makes a roll impossible. Rules saved in presets keep what they had.
pub const NEW_RULE_IMPORTANCE: Importance = Importance::Medium;

/// The whole roll form, serialised into a preset's JSON payload. Field values
/// mirror the page's signals so a saved preset round-trips exactly.
#[derive(Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RollSettings {
    #[serde(default)]
    pub num_days: String,
    #[serde(default)]
    pub difficulties: Vec<DifficultyRule>,
    #[serde(default)]
    pub proteins: Vec<LookupRule>,
    #[serde(default)]
    pub cuisines: Vec<LookupRule>,
    #[serde(default)]
    pub ingredients: Vec<LookupRule>,
    #[serde(default)]
    pub day_limits: DayLimitSettings,
    /// Presets saved before difficulty rules existed kept an exact per-level
    /// day mix instead. It is only read, and turned into band rules.
    #[serde(default, skip_serializing)]
    pub difficulty_exact: bool,
    #[serde(default, skip_serializing)]
    pub difficulty_mix: Vec<DifficultyDays>,
}

/// One rule of the form as the rules panel shows it, whichever list it lives in.
#[derive(Clone, PartialEq)]
pub struct RuleLine {
    pub family: RuleFamily,
    /// Position in its family's list, which is also how a roll names the rule
    /// in the preferences it could not meet.
    pub index: usize,
    /// Stable across edits, for keying rows.
    pub key: String,
    pub name: String,
    pub times: u32,
    pub bound: Bound,
    pub importance: Importance,
}

/// Something a rule can be about: a difficulty band, or a protein, cuisine or
/// ingredient picked from the recipe data.
#[derive(Clone, PartialEq)]
pub enum RuleTarget {
    Band(DifficultyBand),
    Lookup {
        family: RuleFamily,
        id: Uuid,
        name: String,
    },
}

impl RuleTarget {
    /// The same key as the `RuleLine` of a rule about this target.
    pub fn key(&self) -> String {
        match self {
            RuleTarget::Band(band) => format!("band-{band:?}"),
            RuleTarget::Lookup { family, id, .. } => format!("{family:?}-{id}"),
        }
    }
}

/// A group of rules that cannot all hold, in words, and the rules to loosen.
#[derive(Clone, PartialEq, Debug)]
pub struct Conflict {
    pub message: String,
    pub rules: Vec<(RuleFamily, usize)>,
}

impl RollSettings {
    /// How many dinners to roll; 0 when the text is not a number.
    pub fn days(&self) -> u32 {
        self.num_days.trim().parse().unwrap_or(0)
    }

    /// The difficulty rules, reading an older preset's day mix as at-least
    /// rules per band (the mix only applied when it was switched on).
    pub fn difficulty_rules(&self) -> Vec<DifficultyRule> {
        if !self.difficulties.is_empty() || !self.difficulty_exact {
            return self.difficulties.clone();
        }
        DifficultyBand::ALL
            .into_iter()
            .filter_map(|band| {
                let (low, high) = band.levels();
                let times: u32 = self
                    .difficulty_mix
                    .iter()
                    .filter(|d| (low..=high).contains(&d.level))
                    .map(|d| d.days)
                    .sum();
                (times > 0).then_some(DifficultyRule {
                    band,
                    times,
                    bound: Bound::AtLeast,
                    importance: Importance::Required,
                })
            })
            .collect()
    }

    /// The settings with an older preset's difficulty mix turned into rules and
    /// dropped, so a loaded preset compares equal to what the page then holds.
    pub fn normalized(mut self) -> Self {
        self.difficulties = self.difficulty_rules();
        self.difficulty_exact = false;
        self.difficulty_mix.clear();
        self
    }

    /// Every rule, difficulty first, then proteins, cuisines and ingredients.
    pub fn lines(&self) -> Vec<RuleLine> {
        let mut lines = Vec::new();
        for (index, rule) in self.difficulties.iter().enumerate() {
            lines.push(RuleLine {
                family: RuleFamily::Difficulty,
                index,
                key: RuleTarget::Band(rule.band).key(),
                name: band_label(rule.band).to_string(),
                times: rule.times,
                bound: rule.bound,
                importance: rule.importance,
            });
        }
        for (family, rules) in [
            (RuleFamily::Protein, &self.proteins),
            (RuleFamily::Cuisine, &self.cuisines),
            (RuleFamily::Ingredient, &self.ingredients),
        ] {
            for (index, rule) in rules.iter().enumerate() {
                lines.push(RuleLine {
                    family,
                    index,
                    key: format!("{family:?}-{}", rule.id),
                    name: rule.name.clone(),
                    times: rule.times,
                    bound: rule.bound,
                    importance: rule.importance,
                });
            }
        }
        lines
    }

    fn lookups_mut(&mut self, family: RuleFamily) -> Option<&mut Vec<LookupRule>> {
        match family {
            RuleFamily::Protein => Some(&mut self.proteins),
            RuleFamily::Cuisine => Some(&mut self.cuisines),
            RuleFamily::Ingredient => Some(&mut self.ingredients),
            RuleFamily::Difficulty => None,
        }
    }

    /// Replaces the count, bound and importance of one rule.
    pub fn set_rule(
        &mut self,
        family: RuleFamily,
        index: usize,
        times: u32,
        bound: Bound,
        importance: Importance,
    ) {
        if family == RuleFamily::Difficulty {
            if let Some(rule) = self.difficulties.get_mut(index) {
                (rule.times, rule.bound, rule.importance) = (times, bound, importance);
            }
        } else if let Some(rule) = self
            .lookups_mut(family)
            .and_then(|rules| rules.get_mut(index))
        {
            (rule.times, rule.bound, rule.importance) = (times, bound, importance);
        }
    }

    pub fn remove_rule(&mut self, family: RuleFamily, index: usize) {
        if family == RuleFamily::Difficulty {
            if index < self.difficulties.len() {
                self.difficulties.remove(index);
            }
        } else if let Some(rules) = self.lookups_mut(family) {
            if index < rules.len() {
                rules.remove(index);
            }
        }
    }

    /// Adds a preference of "at least one" about `target`, unless it already has a rule.
    pub fn add_rule(&mut self, target: &RuleTarget) {
        match target {
            RuleTarget::Band(band) => self.add_band(*band),
            RuleTarget::Lookup { family, id, name } => {
                self.add_lookup(*family, *id, name.clone());
            }
        }
    }

    /// Whether `target` already has a rule.
    pub fn has_rule(&self, target: &RuleTarget) -> bool {
        let key = target.key();
        self.lines().iter().any(|l| l.key == key)
    }

    /// Adds "at least one" of a protein, cuisine or ingredient as a preference,
    /// unless it already has a rule.
    pub fn add_lookup(&mut self, family: RuleFamily, id: Uuid, name: String) {
        let Some(rules) = self.lookups_mut(family) else {
            return;
        };
        if rules.iter().any(|r| r.id == id) {
            return;
        }
        rules.push(LookupRule {
            id,
            name,
            times: 1,
            bound: Bound::AtLeast,
            importance: NEW_RULE_IMPORTANCE,
        });
    }

    /// Adds "at least one" dinner in a difficulty band as a preference, unless
    /// the band already has a rule.
    pub fn add_band(&mut self, band: DifficultyBand) {
        if self.difficulties.iter().any(|r| r.band == band) {
            return;
        }
        self.difficulties.push(DifficultyRule {
            band,
            times: 1,
            bound: Bound::AtLeast,
            importance: NEW_RULE_IMPORTANCE,
        });
    }

    /// Turns the named required rules into ordinary preferences.
    pub fn make_nice_to_have(&mut self, rules: &[(RuleFamily, usize)]) {
        for &(family, index) in rules {
            let Some(line) = self
                .lines()
                .into_iter()
                .find(|l| l.family == family && l.index == index)
            else {
                continue;
            };
            if line.importance == Importance::Required {
                self.set_rule(family, index, line.times, line.bound, NEW_RULE_IMPORTANCE);
            }
        }
    }

    /// What to send the solver for a roll of `days` dinners.
    pub fn constraints(&self, days: usize) -> RollConstraints {
        RollConstraints {
            cuisines: count_rules(&self.cuisines),
            proteins: count_rules(&self.proteins),
            ingredients: count_rules(&self.ingredients),
            difficulties: self.difficulties.clone(),
            day_limits: self.day_limits.plan(days),
        }
    }

    /// The preferences a roll fell short of, each in a sentence.
    pub fn unmet_sentences(&self, unmet: &[UnmetRule]) -> Vec<String> {
        let lines = self.lines();
        unmet
            .iter()
            .map(|u| {
                let name = lines
                    .iter()
                    .find(|l| l.family == u.family && l.index == u.index)
                    .map_or("A rule", |l| l.name.as_str());
                format!(
                    "{name}: wanted {} {}, got {}.",
                    bound_phrase(u.bound),
                    u.times,
                    u.got
                )
            })
            .collect()
    }

    /// One line about the rules, for when the rules panel is folded.
    pub fn rules_summary(&self) -> String {
        let lines = self.lines();
        if lines.is_empty() {
            return "No rules yet".to_string();
        }
        let must = lines
            .iter()
            .filter(|l| l.importance == Importance::Required)
            .count();
        let shown: Vec<&str> = lines.iter().take(3).map(|l| l.name.as_str()).collect();
        let more = lines.len().saturating_sub(3);
        format!(
            "{} rule{}{}: {}{}",
            lines.len(),
            if lines.len() == 1 { "" } else { "s" },
            if must > 0 {
                format!(", {must} Must")
            } else {
                String::new()
            },
            shown.join(", "),
            if more > 0 {
                format!(" +{more} more")
            } else {
                String::new()
            },
        )
    }

    /// Required rules that cannot all hold in a roll of `days` dinners, found
    /// without looking at any recipes: a floor above the number of dinners, a
    /// family whose floors add up to more than the dinners (a dinner has one
    /// protein, one cuisine and one difficulty), and ceilings on every
    /// difficulty band that together allow fewer dinners than rolled.
    pub fn conflicts(&self, days: u32) -> Vec<Conflict> {
        let mut found = Vec::new();
        if days == 0 {
            return found;
        }
        let lines = self.lines();
        let required = |l: &&RuleLine| l.importance == Importance::Required;
        let floor = |l: &&RuleLine| required(l) && l.bound == Bound::AtLeast;

        for line in lines.iter().filter(floor).filter(|l| l.times > days) {
            found.push(Conflict {
                message: format!(
                    "{}: at least {} dinners, but you are rolling {days}.",
                    line.name, line.times
                ),
                rules: vec![(line.family, line.index)],
            });
        }

        for family in [
            RuleFamily::Difficulty,
            RuleFamily::Protein,
            RuleFamily::Cuisine,
        ] {
            let floors: Vec<&RuleLine> = lines
                .iter()
                .filter(|l| l.family == family)
                .filter(floor)
                .filter(|l| l.times <= days)
                .collect();
            let total: u32 = floors.iter().map(|l| l.times).sum();
            if floors.len() > 1 && total > days {
                found.push(Conflict {
                    message: format!(
                        "Your Must {} rules ask for {total} dinners between them, but a dinner has one {} and you are rolling {days}.",
                        family_noun(family),
                        family_noun(family),
                    ),
                    rules: floors.iter().map(|l| (l.family, l.index)).collect(),
                });
            }
        }

        let ceilings: Vec<&RuleLine> = lines
            .iter()
            .filter(|l| l.family == RuleFamily::Difficulty)
            .filter(required)
            .filter(|l| l.bound == Bound::AtMost)
            .collect();
        let total: u32 = ceilings.iter().map(|l| l.times).sum();
        if ceilings.len() == DifficultyBand::ALL.len() && total < days {
            found.push(Conflict {
                message: format!(
                    "Your Must difficulty limits allow {total} dinners in total, but you are rolling {days}."
                ),
                rules: ceilings.iter().map(|l| (l.family, l.index)).collect(),
            });
        }
        found
    }
}

fn count_rules(rules: &[LookupRule]) -> Vec<CountRule> {
    rules
        .iter()
        .map(|rule| CountRule {
            id: rule.id,
            times: rule.times,
            bound: rule.bound,
            importance: rule.importance,
        })
        .collect()
}

/// The limit every day follows, and the days that have their own. Days are
/// numbers from the first rolled day (0), so a preset fits any start date. Off
/// by default, so a plain roll is unlimited.
#[derive(Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DayLimitSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub default: DayLimits,
    #[serde(default)]
    pub overrides: Vec<DayLimitsAt>,
}

impl DayLimitSettings {
    /// The own limits of rolled day `day`, if it has any.
    pub fn override_for(&self, day: usize) -> Option<DayLimits> {
        self.overrides
            .iter()
            .find(|o| usize::from(o.day) == day)
            .map(|o| o.limits)
    }

    /// Sets or, with `None`, clears the own limits of rolled day `day`.
    pub fn set_override(&mut self, day: usize, limits: Option<DayLimits>) {
        let Ok(day) = u16::try_from(day) else {
            return;
        };
        self.overrides.retain(|o| o.day != day);
        if let Some(limits) = limits {
            self.overrides.push(DayLimitsAt { day, limits });
        }
    }

    /// The limits to send with a roll of `num_days`, or `None` when they are
    /// switched off. Overrides past `num_days` are left out.
    pub fn plan(&self, num_days: usize) -> Option<DayLimitPlan> {
        self.enabled.then(|| DayLimitPlan {
            default: self.default,
            overrides: self
                .overrides
                .iter()
                .filter(|o| usize::from(o.day) < num_days)
                .copied()
                .collect(),
        })
    }
}

/// How many days an older preset wanted at a difficulty level (1 = Very easy .. 5 = Ambitious).
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DifficultyDays {
    pub level: i16,
    pub days: u32,
}

fn one() -> u32 {
    1
}

/// A protein, cuisine or must-use ingredient rule, kept with its label so rows
/// render without a lookup. Older presets stored ingredients as `{id, label}`
/// and rules without a bound or importance; those read as at least once, required.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LookupRule {
    pub id: Uuid,
    #[serde(alias = "label")]
    pub name: String,
    #[serde(default = "one")]
    pub times: u32,
    #[serde(default)]
    pub bound: Bound,
    #[serde(default)]
    pub importance: Importance,
}

pub const IMPORTANCE_LEVELS: [Importance; 4] = [
    Importance::Required,
    Importance::High,
    Importance::Medium,
    Importance::Low,
];

pub fn importance_label(importance: Importance) -> &'static str {
    match importance {
        Importance::Required => "Must",
        Importance::High => "Strongly prefer",
        Importance::Medium => "Prefer",
        Importance::Low => "If possible",
    }
}

pub fn bound_phrase(bound: Bound) -> &'static str {
    match bound {
        Bound::AtLeast => "at least",
        Bound::AtMost => "at most",
    }
}

pub fn band_label(band: DifficultyBand) -> &'static str {
    match band {
        DifficultyBand::Easy => "Easy",
        DifficultyBand::Medium => "Medium",
        DifficultyBand::Hard => "Hard",
    }
}

pub fn family_noun(family: RuleFamily) -> &'static str {
    match family {
        RuleFamily::Difficulty => "difficulty",
        RuleFamily::Protein => "protein",
        RuleFamily::Cuisine => "cuisine",
        RuleFamily::Ingredient => "ingredient",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup(name: &str, times: u32, bound: Bound, importance: Importance) -> LookupRule {
        LookupRule {
            id: Uuid::now_v7(),
            name: name.to_string(),
            times,
            bound,
            importance,
        }
    }

    fn band(
        band: DifficultyBand,
        times: u32,
        bound: Bound,
        importance: Importance,
    ) -> DifficultyRule {
        DifficultyRule {
            band,
            times,
            bound,
            importance,
        }
    }

    #[test]
    fn a_preset_saved_before_rules_had_bounds_still_loads() {
        let saved = r#"{
            "num_days": "5",
            "difficulty": "balanced",
            "difficulty_exact": true,
            "difficulty_mix": [{"level": 1, "days": 1}, {"level": 2, "days": 2}, {"level": 5, "days": 1}],
            "proteins": [{"id": "01a04351-39b7-7761-8bfb-58dad6017b5e", "day": null, "name": "Beef", "times": 2}],
            "cuisines": [],
            "ingredients": [{"id": "01a04351-39bb-7cf4-8aee-6c2936ff9f2b", "label": "Spinach"}]
        }"#;
        let settings: RollSettings = serde_json::from_str(saved).expect("old presets parse");
        let beef = &settings.proteins[0];
        assert_eq!(
            (beef.times, beef.bound, beef.importance),
            (2, Bound::AtLeast, Importance::Required)
        );
        let spinach = &settings.ingredients[0];
        assert_eq!((spinach.name.as_str(), spinach.times), ("Spinach", 1));
        let bands: Vec<(DifficultyBand, u32)> = settings
            .difficulty_rules()
            .iter()
            .map(|r| (r.band, r.times))
            .collect();
        assert_eq!(
            bands,
            [(DifficultyBand::Easy, 3), (DifficultyBand::Hard, 1)]
        );
    }

    #[test]
    fn a_preset_without_day_limits_loads_with_them_off() {
        let settings: RollSettings = serde_json::from_str(r#"{"num_days": "5"}"#).expect("parses");
        assert!(!settings.day_limits.enabled);
        assert!(settings.day_limits.plan(5).is_none());
    }

    #[test]
    fn day_limits_round_trip_through_a_preset() {
        let mut limits = DayLimitSettings {
            enabled: true,
            ..DayLimitSettings::default()
        };
        limits.default.max_time_mins = Some(30);
        limits.set_override(
            2,
            Some(DayLimits {
                min_difficulty: Some(DifficultyBand::Hard),
                ..DayLimits::default()
            }),
        );
        let settings = RollSettings {
            day_limits: limits.clone(),
            ..RollSettings::default()
        };
        let json = serde_json::to_value(&settings).expect("serialises");
        let back: RollSettings = serde_json::from_value(json).expect("parses");
        assert!(back.day_limits == limits);
    }

    #[test]
    fn only_the_rolled_days_overrides_are_sent() {
        let mut limits = DayLimitSettings {
            enabled: true,
            ..DayLimitSettings::default()
        };
        limits.set_override(2, Some(DayLimits::default()));
        limits.set_override(9, Some(DayLimits::default()));
        let plan = limits.plan(5).expect("enabled");
        assert_eq!(plan.overrides.len(), 1);
        assert_eq!(plan.overrides[0].day, 2);
        limits.set_override(2, None);
        assert!(limits.override_for(2).is_none());
    }

    #[test]
    fn saving_drops_the_old_difficulty_fields() {
        let json = serde_json::to_string(&RollSettings::default()).expect("settings serialise");
        assert!(!json.contains("difficulty_mix") && !json.contains("difficulty_exact"));
    }

    #[test]
    fn a_loaded_old_preset_matches_what_the_page_then_holds() {
        let old: RollSettings = serde_json::from_str(
            r#"{"num_days": "4", "difficulty_exact": true, "difficulty_mix": [{"level": 5, "days": 1}]}"#,
        )
        .expect("parses");
        let held = old.clone().normalized();
        assert_eq!(held.difficulties.len(), 1);
        assert!(held == held.clone().normalized());
    }

    #[test]
    fn a_new_rule_is_a_preference_but_a_saved_rule_keeps_its_importance() {
        let mut settings = RollSettings::default();
        settings.add_lookup(RuleFamily::Protein, Uuid::now_v7(), "Beef".to_string());
        settings.add_band(DifficultyBand::Hard);
        assert!(settings
            .lines()
            .iter()
            .all(|l| l.importance == Importance::Medium));
        let saved: RollSettings = serde_json::from_str(
            r#"{"proteins": [{"id": "01a04351-39b7-7761-8bfb-58dad6017b5e", "name": "Beef", "times": 2}]}"#,
        )
        .expect("parses");
        assert_eq!(saved.lines()[0].importance, Importance::Required);
    }

    #[test]
    fn adding_the_same_rule_twice_keeps_one() {
        let id = Uuid::now_v7();
        let mut settings = RollSettings::default();
        settings.add_lookup(RuleFamily::Cuisine, id, "Thai".to_string());
        settings.add_lookup(RuleFamily::Cuisine, id, "Thai".to_string());
        settings.add_band(DifficultyBand::Easy);
        settings.add_band(DifficultyBand::Easy);
        assert_eq!(settings.lines().len(), 2);
    }

    #[test]
    fn rules_are_listed_by_family_and_edited_by_position() {
        let mut settings = RollSettings::default();
        settings.add_lookup(
            RuleFamily::Ingredient,
            Uuid::now_v7(),
            "Spinach".to_string(),
        );
        settings.add_lookup(RuleFamily::Protein, Uuid::now_v7(), "Beef".to_string());
        settings.add_band(DifficultyBand::Hard);
        let names: Vec<String> = settings.lines().into_iter().map(|l| l.name).collect();
        assert_eq!(names, ["Hard", "Beef", "Spinach"]);
        settings.set_rule(
            RuleFamily::Protein,
            0,
            3,
            Bound::AtMost,
            Importance::Required,
        );
        let beef = &settings.lines()[1];
        assert_eq!((beef.times, beef.bound), (3, Bound::AtMost));
        settings.remove_rule(RuleFamily::Difficulty, 0);
        assert_eq!(settings.lines().len(), 2);
    }

    #[test]
    fn a_must_floor_above_the_dinners_is_a_conflict_and_a_preference_is_not() {
        let mut settings = RollSettings {
            num_days: "7".to_string(),
            proteins: vec![lookup("Beef", 9, Bound::AtLeast, Importance::Required)],
            ..RollSettings::default()
        };
        let found = settings.conflicts(7);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].rules, [(RuleFamily::Protein, 0)]);
        settings.make_nice_to_have(&found[0].rules);
        assert!(settings.conflicts(7).is_empty());
        assert_eq!(settings.proteins[0].importance, NEW_RULE_IMPORTANCE);
        assert_eq!(settings.proteins[0].times, 9);
    }

    #[test]
    fn floors_in_one_family_that_add_up_past_the_dinners_conflict() {
        let settings = RollSettings {
            proteins: vec![
                lookup("Beef", 4, Bound::AtLeast, Importance::Required),
                lookup("Chicken", 4, Bound::AtLeast, Importance::Required),
                lookup("Fish", 1, Bound::AtLeast, Importance::Medium),
            ],
            ingredients: vec![
                lookup("Spinach", 4, Bound::AtLeast, Importance::Required),
                lookup("Garlic", 4, Bound::AtLeast, Importance::Required),
            ],
            ..RollSettings::default()
        };
        let found = settings.conflicts(7);
        assert_eq!(
            found.len(),
            1,
            "ingredients overlap, so only proteins clash"
        );
        assert_eq!(found[0].rules.len(), 2);
    }

    #[test]
    fn ceilings_on_every_difficulty_band_can_leave_too_few_dinners() {
        let settings = RollSettings {
            difficulties: vec![
                band(DifficultyBand::Easy, 2, Bound::AtMost, Importance::Required),
                band(
                    DifficultyBand::Medium,
                    2,
                    Bound::AtMost,
                    Importance::Required,
                ),
                band(DifficultyBand::Hard, 1, Bound::AtMost, Importance::Required),
            ],
            ..RollSettings::default()
        };
        assert_eq!(settings.conflicts(7).len(), 1);
        assert!(settings.conflicts(5).is_empty());
        assert!(settings.conflicts(0).is_empty());
    }

    #[test]
    fn the_unmet_sentences_name_the_rule() {
        let settings = RollSettings {
            proteins: vec![lookup("Beef", 3, Bound::AtLeast, Importance::Medium)],
            difficulties: vec![band(
                DifficultyBand::Hard,
                1,
                Bound::AtMost,
                Importance::Low,
            )],
            ..RollSettings::default()
        };
        let sentences = settings.unmet_sentences(&[UnmetRule {
            family: RuleFamily::Protein,
            index: 0,
            bound: Bound::AtLeast,
            times: 3,
            got: 1,
        }]);
        assert_eq!(sentences, ["Beef: wanted at least 3, got 1."]);
    }

    #[test]
    fn the_folded_summary_counts_the_rules() {
        let mut settings = RollSettings::default();
        assert_eq!(settings.rules_summary(), "No rules yet");
        settings.proteins = vec![lookup("Beef", 2, Bound::AtLeast, Importance::Required)];
        settings.add_band(DifficultyBand::Easy);
        assert_eq!(settings.rules_summary(), "2 rules, 1 Must: Easy, Beef");
    }
}
