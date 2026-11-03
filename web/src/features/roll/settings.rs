use domain::roll::{
    Bound, DayLimitPlan, DayLimits, DayLimitsAt, DifficultyBand, DifficultyRule, Importance,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Days a fresh roll form asks for, and so the recipes a first week needs.
pub const DEFAULT_NUM_DAYS: usize = 7;

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

impl RollSettings {
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
        Importance::Required => "Required",
        Importance::High => "High",
        Importance::Medium => "Medium",
        Importance::Low => "Low",
    }
}

pub fn bound_label(bound: Bound) -> &'static str {
    match bound {
        Bound::AtLeast => "At least",
        Bound::AtMost => "At most",
    }
}

pub fn band_label(band: DifficultyBand) -> &'static str {
    match band {
        DifficultyBand::Easy => "Easy (very easy or easy)",
        DifficultyBand::Medium => "Medium",
        DifficultyBand::Hard => "Hard (hard or ambitious)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
