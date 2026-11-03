use domain::roll::{
    Bound, CountRule, DayLimitPlan, DayLimits, DifficultyBand, DifficultyRule, Importance,
    RollConstraints, RuleFamily, UnmetRule,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Days a fresh roll form asks for, and so the recipes a first week needs.
pub const DEFAULT_NUM_DAYS: usize = 7;

/// The most dinners one roll may ask for; the server refuses more.
pub const MAX_ROLL_DAYS: u32 = 31;

/// What a rule with Must off is sent as. The page shows only Must on or off, so
/// a fresh rule cannot be what makes a roll impossible.
pub const NEW_RULE_IMPORTANCE: Importance = Importance::Medium;

/// The importance a rule is sent with for a Must toggle.
pub fn importance_for(must: bool) -> Importance {
    if must {
        Importance::Required
    } else {
        NEW_RULE_IMPORTANCE
    }
}

/// The cook-time caps the "Every dinner" rule offers, in minutes.
pub const COOK_TIME_CHOICES: [u32; 7] = [15, 20, 30, 45, 60, 90, 120];

/// The cook-time cap a new "Every dinner" rule starts with.
pub const DEFAULT_COOK_TIME_CAP: u32 = 45;

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
    pub every_dinner: EveryDinner,
    /// Presets saved before "Every dinner" kept per-day limits instead. Only
    /// read, and turned into the every-dinner caps.
    #[serde(default, skip_serializing)]
    pub day_limits: OldDayLimits,
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

    pub fn family(&self) -> RuleFamily {
        match self {
            RuleTarget::Band(_) => RuleFamily::Difficulty,
            RuleTarget::Lookup { family, .. } => *family,
        }
    }

    pub fn name(&self) -> String {
        match self {
            RuleTarget::Band(band) => band_label(*band).to_string(),
            RuleTarget::Lookup { name, .. } => name.clone(),
        }
    }
}

/// Caps every rolled dinner follows. Sent to the solver as one set of day
/// limits that applies to every day.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct EveryDinner {
    #[serde(default)]
    pub max_time_mins: Option<u32>,
    #[serde(default)]
    pub max_difficulty: Option<DifficultyBand>,
}

impl EveryDinner {
    pub fn plan(self) -> Option<DayLimitPlan> {
        let limits = DayLimits {
            min_difficulty: None,
            max_difficulty: self.max_difficulty,
            max_time_mins: self.max_time_mins,
        };
        limits.is_limited().then(|| DayLimitPlan {
            default: limits,
            overrides: Vec::new(),
        })
    }

    pub fn count(self) -> usize {
        usize::from(self.max_time_mins.is_some()) + usize::from(self.max_difficulty.is_some())
    }
}

/// The per-day limits of an older preset. Only the limit for all days is kept;
/// a day's own limits and the easiest band are dropped.
#[derive(Clone, PartialEq, Deserialize, Default)]
pub struct OldDayLimits {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub default: DayLimits,
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

    /// The settings as the page holds them, so a loaded preset compares equal
    /// to what the page then shows: an older preset's difficulty mix turned
    /// into rules, its day limits into every-dinner caps, and any importance
    /// other than Must read as Must off.
    pub fn normalized(mut self) -> Self {
        self.difficulties = self.difficulty_rules();
        self.difficulty_exact = false;
        self.difficulty_mix.clear();
        let old = std::mem::take(&mut self.day_limits);
        if old.enabled && self.every_dinner == EveryDinner::default() {
            self.every_dinner = EveryDinner {
                max_time_mins: old.default.max_time_mins.filter(|m| *m > 0),
                max_difficulty: old.default.max_difficulty,
            };
        }
        if self.every_dinner.max_difficulty == Some(DifficultyBand::Hard) {
            self.every_dinner.max_difficulty = None;
        }
        let must_or_not = |i: &mut Importance| *i = importance_for(*i == Importance::Required);
        self.difficulties
            .iter_mut()
            .for_each(|r| must_or_not(&mut r.importance));
        for rules in [
            &mut self.proteins,
            &mut self.cuisines,
            &mut self.ingredients,
        ] {
            rules
                .iter_mut()
                .for_each(|r| must_or_not(&mut r.importance));
        }
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

    /// Adds a rule about `target`, unless it already has one.
    pub fn add_rule(
        &mut self,
        target: &RuleTarget,
        times: u32,
        bound: Bound,
        importance: Importance,
    ) {
        if self.has_rule(target) {
            return;
        }
        match target {
            RuleTarget::Band(band) => self.difficulties.push(DifficultyRule {
                band: *band,
                times,
                bound,
                importance,
            }),
            RuleTarget::Lookup { family, id, name } => {
                if let Some(rules) = self.lookups_mut(*family) {
                    rules.push(LookupRule {
                        id: *id,
                        name: name.clone(),
                        times,
                        bound,
                        importance,
                    });
                }
            }
        }
    }

    /// Whether `target` already has a rule.
    pub fn has_rule(&self, target: &RuleTarget) -> bool {
        let key = target.key();
        self.lines().iter().any(|l| l.key == key)
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

    /// What to send the solver.
    pub fn constraints(&self) -> RollConstraints {
        RollConstraints {
            cuisines: count_rules(&self.cuisines),
            proteins: count_rules(&self.proteins),
            ingredients: count_rules(&self.ingredients),
            difficulties: self.difficulties.clone(),
            day_limits: self.every_dinner.plan(),
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
        let mut names: Vec<String> = lines.iter().map(|l| l.name.clone()).collect();
        if let Some(mins) = self.every_dinner.max_time_mins {
            names.push(format!("max {mins} min"));
        }
        if let Some(band) = self.every_dinner.max_difficulty {
            names.push(format!("max {}", band_label(band)));
        }
        if names.is_empty() {
            return "No rules yet".to_string();
        }
        let must = lines
            .iter()
            .filter(|l| l.importance == Importance::Required)
            .count();
        let more = names.len().saturating_sub(3);
        format!(
            "{} rule{}{}: {}{}",
            names.len(),
            if names.len() == 1 { "" } else { "s" },
            if must > 0 {
                format!(", {must} Must")
            } else {
                String::new()
            },
            names[..names.len().min(3)].join(", "),
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

        if let Some(cap) = self.every_dinner.max_difficulty {
            for (index, rule) in self.difficulties.iter().enumerate() {
                let above_cap = rule.band.levels().0 > cap.levels().1;
                if above_cap
                    && rule.importance == Importance::Required
                    && rule.bound == Bound::AtLeast
                    && rule.times > 0
                {
                    found.push(Conflict {
                        message: format!(
                            "{}: at least {} dinners, but every dinner is at most {}.",
                            band_label(rule.band),
                            rule.times,
                            band_label(cap),
                        ),
                        rules: vec![(RuleFamily::Difficulty, index)],
                    });
                }
            }
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

pub fn family_label(family: RuleFamily) -> &'static str {
    match family {
        RuleFamily::Difficulty => "Difficulty",
        RuleFamily::Protein => "Protein",
        RuleFamily::Cuisine => "Cuisine",
        RuleFamily::Ingredient => "Ingredient",
    }
}

/// What a count rule counts, after its number: "Beef dinners", "dinner with Spinach".
pub fn rule_noun(family: RuleFamily, name: &str, times: u32) -> String {
    let dinners = if times == 1 { "dinner" } else { "dinners" };
    if family == RuleFamily::Ingredient {
        format!("{dinners} with {name}")
    } else {
        format!("{name} {dinners}")
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

    fn protein(name: &str) -> RuleTarget {
        RuleTarget::Lookup {
            family: RuleFamily::Protein,
            id: Uuid::now_v7(),
            name: name.to_string(),
        }
    }

    #[test]
    fn a_preset_without_caps_rolls_without_day_limits() {
        let settings: RollSettings = serde_json::from_str(r#"{"num_days": "5"}"#).expect("parses");
        assert!(settings.normalized().constraints().day_limits.is_none());
    }

    #[test]
    fn an_old_presets_day_limits_become_every_dinner_caps() {
        let saved = r#"{
            "num_days": "5",
            "day_limits": {
                "enabled": true,
                "default": {"min_difficulty": "Easy", "max_difficulty": "Medium", "max_time_mins": 40},
                "overrides": [{"day": 2, "limits": {"min_difficulty": "Hard"}}]
            }
        }"#;
        let settings = serde_json::from_str::<RollSettings>(saved)
            .expect("old presets parse")
            .normalized();
        assert!(
            settings.every_dinner
                == EveryDinner {
                    max_time_mins: Some(40),
                    max_difficulty: Some(DifficultyBand::Medium),
                }
        );
        let plan = settings.constraints().day_limits.expect("capped");
        assert!(plan.overrides.is_empty());
        assert_eq!(plan.default.min_difficulty, None);
        let json = serde_json::to_string(&settings).expect("serialises");
        assert!(!json.contains("day_limits"));
    }

    #[test]
    fn switched_off_day_limits_and_a_hard_cap_mean_no_cap() {
        let off: RollSettings = serde_json::from_str(
            r#"{"day_limits": {"enabled": false, "default": {"max_time_mins": 30}}}"#,
        )
        .expect("parses");
        assert!(off.normalized().every_dinner == EveryDinner::default());
        let hard: RollSettings =
            serde_json::from_str(r#"{"every_dinner": {"max_difficulty": "Hard"}}"#)
                .expect("parses");
        assert!(hard.normalized().constraints().day_limits.is_none());
    }

    #[test]
    fn every_dinner_caps_round_trip_through_a_preset() {
        let settings = RollSettings {
            every_dinner: EveryDinner {
                max_time_mins: Some(30),
                max_difficulty: Some(DifficultyBand::Easy),
            },
            ..RollSettings::default()
        };
        let json = serde_json::to_value(&settings).expect("serialises");
        let back: RollSettings = serde_json::from_value(json).expect("parses");
        assert!(back.normalized() == settings);
    }

    #[test]
    fn levels_other_than_must_load_as_must_off() {
        let saved: RollSettings = serde_json::from_str(
            r#"{
                "proteins": [
                    {"id": "01a04351-39b7-7761-8bfb-58dad6017b5e", "name": "Beef", "times": 2},
                    {"id": "01a04351-39b7-7761-8bfb-58dad6017b5f", "name": "Fish", "importance": "High"}
                ],
                "difficulties": [{"band": "Easy", "times": 1, "importance": "Low"}]
            }"#,
        )
        .expect("parses");
        let levels: Vec<Importance> = saved
            .normalized()
            .lines()
            .iter()
            .map(|l| l.importance)
            .collect();
        assert_eq!(
            levels,
            [Importance::Medium, Importance::Required, Importance::Medium]
        );
    }

    #[test]
    fn the_must_toggle_maps_to_required_or_the_middle_level() {
        assert_eq!(importance_for(true), Importance::Required);
        assert_eq!(importance_for(false), Importance::Medium);
    }

    #[test]
    fn adding_the_same_rule_twice_keeps_one() {
        let thai = protein("Thai");
        let mut settings = RollSettings::default();
        settings.add_rule(&thai, 1, Bound::AtLeast, Importance::Medium);
        settings.add_rule(&thai, 3, Bound::AtMost, Importance::Required);
        settings.add_rule(
            &RuleTarget::Band(DifficultyBand::Easy),
            1,
            Bound::AtLeast,
            Importance::Medium,
        );
        let lines = settings.lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1].times, 1);
    }

    #[test]
    fn rules_are_listed_by_family_and_edited_by_position() {
        let mut settings = RollSettings::default();
        let spinach = RuleTarget::Lookup {
            family: RuleFamily::Ingredient,
            id: Uuid::now_v7(),
            name: "Spinach".to_string(),
        };
        settings.add_rule(&spinach, 1, Bound::AtLeast, Importance::Medium);
        settings.add_rule(&protein("Beef"), 2, Bound::AtLeast, Importance::Medium);
        settings.add_rule(
            &RuleTarget::Band(DifficultyBand::Hard),
            1,
            Bound::AtMost,
            Importance::Medium,
        );
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
    fn a_must_floor_above_the_difficulty_cap_conflicts() {
        let mut settings = RollSettings {
            difficulties: vec![band(
                DifficultyBand::Hard,
                1,
                Bound::AtLeast,
                Importance::Required,
            )],
            every_dinner: EveryDinner {
                max_time_mins: None,
                max_difficulty: Some(DifficultyBand::Medium),
            },
            ..RollSettings::default()
        };
        let found = settings.conflicts(7);
        assert_eq!(found.len(), 1);
        settings.make_nice_to_have(&found[0].rules);
        assert!(settings.conflicts(7).is_empty());
    }

    #[test]
    fn the_folded_summary_counts_the_rules_and_caps() {
        let mut settings = RollSettings::default();
        assert_eq!(settings.rules_summary(), "No rules yet");
        settings.proteins = vec![lookup("Beef", 2, Bound::AtLeast, Importance::Required)];
        settings.add_rule(
            &RuleTarget::Band(DifficultyBand::Easy),
            1,
            Bound::AtLeast,
            Importance::Medium,
        );
        assert_eq!(settings.rules_summary(), "2 rules, 1 Must: Easy, Beef");
        settings.every_dinner.max_time_mins = Some(30);
        settings.every_dinner.max_difficulty = Some(DifficultyBand::Medium);
        assert_eq!(
            settings.rules_summary(),
            "4 rules, 1 Must: Easy, Beef, max 30 min +1 more"
        );
    }
}
