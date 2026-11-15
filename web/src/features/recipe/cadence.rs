/// The friendly settings for how often a recipe comes round, each a minimum
/// and maximum gap in weeks.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cadence {
    AnyTime,
    Often,
    Sometimes,
    Rarely,
}

impl Cadence {
    pub const ALL: [Cadence; 4] = [
        Cadence::AnyTime,
        Cadence::Often,
        Cadence::Sometimes,
        Cadence::Rarely,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Cadence::AnyTime => "Any time",
            Cadence::Often => "Often",
            Cadence::Sometimes => "Sometimes",
            Cadence::Rarely => "Rarely",
        }
    }

    /// (minimum, maximum) gap in weeks.
    pub fn gaps(self) -> (Option<i16>, Option<i16>) {
        match self {
            Cadence::AnyTime => (None, None),
            Cadence::Often => (None, Some(2)),
            Cadence::Sometimes => (Some(2), Some(6)),
            Cadence::Rarely => (Some(6), Some(12)),
        }
    }

    /// The preset these gaps are, if they are one.
    pub fn of(min: Option<i16>, max: Option<i16>) -> Option<Cadence> {
        Cadence::ALL.into_iter().find(|c| c.gaps() == (min, max))
    }
}

/// The longest gap the database accepts.
pub const MAX_GAP_WEEKS: u32 = 52;

/// The gaps in weeks, in words.
pub fn gap_words(min: Option<i16>, max: Option<i16>) -> String {
    match (min, max) {
        (Some(min), Some(max)) => format!("{min} to {max} weeks apart"),
        (Some(min), None) => format!("At least {min} weeks apart"),
        (None, Some(max)) => format!("At most {max} weeks apart"),
        (None, None) => "No limit".to_string(),
    }
}

/// How a recipe's gaps read on its page: the preset's name, or the weeks.
pub fn describe(min: Option<i16>, max: Option<i16>) -> String {
    Cadence::of(min, max).map_or_else(|| gap_words(min, max), |c| c.label().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_preset_reads_back_from_its_own_gaps() {
        for cadence in Cadence::ALL {
            let (min, max) = cadence.gaps();
            assert_eq!(Cadence::of(min, max), Some(cadence));
        }
    }

    #[test]
    fn gaps_matching_no_preset_are_described_in_weeks() {
        assert_eq!(describe(Some(3), Some(8)), "3 to 8 weeks apart");
        assert_eq!(describe(Some(2), Some(6)), "Sometimes");
    }
}
