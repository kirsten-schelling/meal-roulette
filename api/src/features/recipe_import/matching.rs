use super::line::{canonical_unit, match_key};
use domain::recipe_import::MatchKind;
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

/// A match at or above this confidence starts out as the chosen ingredient; below it the line
/// starts as a new ingredient and the match is only offered.
pub const CONFIDENT: f32 = 0.75;

/// Below this a library ingredient isn't worth mentioning at all.
const WORTH_OFFERING: f32 = 0.4;

/// Words that describe size, preparation or serving rather than the ingredient itself.
const DESCRIPTORS: &[&str] = &[
    "large",
    "small",
    "medium",
    "big",
    "jumbo",
    "extra",
    "virgin",
    "fresh",
    "freshly",
    "ripe",
    "raw",
    "chopped",
    "diced",
    "sliced",
    "minced",
    "grated",
    "crushed",
    "peeled",
    "cubed",
    "shredded",
    "melted",
    "softened",
    "beaten",
    "trimmed",
    "halved",
    "quartered",
    "drained",
    "rinsed",
    "boneless",
    "skinless",
    "finely",
    "roughly",
    "thinly",
    "thickly",
    "coarsely",
    "lightly",
    "good",
    "quality",
    "wedge",
    "wedges",
    "slice",
    "slices",
    "chunk",
    "chunks",
    "piece",
    "pieces",
    "strip",
    "strips",
    "optional",
    "garnish",
    "serving",
];

/// The ingredient wording left once sizes, preparation and serving words are removed, in
/// lowercase with its original word forms (`large eggs` gives `eggs`).
pub fn clean_phrase(name: &str) -> String {
    let lowered = name.to_lowercase();
    let words: Vec<&str> = lowered
        .split(|c: char| c.is_whitespace() || ",;:.!?\"".contains(c))
        .filter(|w| !w.is_empty())
        .collect();
    let mut kept: Vec<&str> = Vec::new();
    for (i, word) in words.iter().enumerate() {
        if i > 0 && matches!(*word, "for" | "to") {
            break;
        }
        let garlic_clove = matches!(*word, "clove" | "cloves") && kept.last() == Some(&"garlic");
        if DESCRIPTORS.contains(word) || garlic_clove || (kept.is_empty() && *word == "of") {
            continue;
        }
        kept.push(word);
    }
    if kept.is_empty() {
        return words.join(" ");
    }
    kept.join(" ")
}

/// The form two spellings of the same phrase share; this is what aliases are stored under.
pub fn phrase_key(phrase: &str) -> String {
    match_key(&clean_phrase(phrase))
}

pub fn capitalise_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Found {
    pub index: usize,
    pub confidence: f32,
    pub kind: MatchKind,
}

/// The library ingredient `phrase` most resembles. `names` are the library ids and names in
/// order; `aliases` maps a remembered phrase key to the ingredient id chosen for it.
pub fn find_match(
    phrase: &str,
    names: &[(Uuid, &str)],
    aliases: &HashMap<String, Uuid>,
) -> Option<Found> {
    let cleaned = clean_phrase(phrase);
    let key = match_key(&cleaned);
    if key.is_empty() {
        return None;
    }

    if let Some(index) = names
        .iter()
        .position(|(_, name)| name.trim().to_lowercase() == cleaned)
    {
        return Some(found(index, 1.0, MatchKind::Exact));
    }
    if let Some(index) = aliases
        .get(&key)
        .and_then(|id| names.iter().position(|(n, _)| n == id))
    {
        return Some(found(index, 0.98, MatchKind::Remembered));
    }

    let keys: Vec<String> = names.iter().map(|(_, n)| phrase_key(n)).collect();
    if let Some(index) = keys.iter().position(|k| *k == key) {
        return Some(found(index, 0.95, MatchKind::Singular));
    }

    keys.iter()
        .enumerate()
        .filter_map(|(index, candidate)| {
            similarity(&key, candidate)
                .map(|confidence| found(index, confidence, MatchKind::Similar))
        })
        .filter(|f| f.confidence >= WORTH_OFFERING)
        .max_by(|a, b| {
            a.confidence
                .total_cmp(&b.confidence)
                .then(b.index.cmp(&a.index))
        })
}

fn found(index: usize, confidence: f32, kind: MatchKind) -> Found {
    Found {
        index,
        confidence,
        kind,
    }
}

fn similarity(phrase: &str, candidate: &str) -> Option<f32> {
    let a: Vec<&str> = phrase.split(' ').collect();
    let b: Vec<&str> = candidate.split(' ').collect();
    let shared = a.iter().filter(|t| b.contains(t)).count();
    let mut best: Option<f32> = None;

    if shared > 0 {
        let union = a.len() + b.len() - shared;
        let jaccard = shared as f32 / union as f32;
        let score = if shared == a.len() && shared == b.len() {
            0.9
        } else if shared == a.len() || shared == b.len() {
            0.4 + 0.4 * jaccard
        } else {
            0.3 * jaccard
        };
        best = Some(score);
    }

    let distance = edit_distance(phrase, candidate);
    let longest = phrase.chars().count().max(candidate.chars().count());
    let allowed = match longest {
        0..=3 => 0,
        4..=7 => 1,
        _ => 2,
    };
    if distance > 0 && distance <= allowed {
        let typo = 0.85 - 0.07 * (distance - 1) as f32;
        best = Some(best.map_or(typo, |b| b.max(typo)));
    }
    best
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = if ca == cb {
                diagonal
            } else {
                1 + diagonal.min(above).min(row[j])
            };
            diagonal = above;
        }
    }
    row[b.len()]
}

#[derive(Clone, Copy, PartialEq)]
enum Family {
    Volume,
    Mass,
}

/// The unit's family and how many millilitres or grams one of it holds.
fn measure(unit: &str) -> Option<(Family, Decimal)> {
    let (family, base) = match canonical_unit(unit).as_str() {
        "ml" => (Family::Volume, "1"),
        "l" => (Family::Volume, "1000"),
        "tsp" => (Family::Volume, "5"),
        "tbsp" => (Family::Volume, "15"),
        "cup" => (Family::Volume, "250"),
        "g" => (Family::Mass, "1"),
        "kg" => (Family::Mass, "1000"),
        "oz" => (Family::Mass, "28.35"),
        "lb" => (Family::Mass, "453.59"),
        _ => return None,
    };
    Some((family, base.parse().ok()?))
}

/// What a line's amount becomes when written in the library ingredient's unit.
#[derive(Debug, PartialEq)]
pub enum Amount {
    Keep(Option<Decimal>),
    Converted(Decimal),
    /// The units can't be related; the caller keeps the page's wording instead.
    Unrelated,
}

pub fn convert_amount(qty: Option<Decimal>, from: Option<&str>, to: &str) -> Amount {
    let Some(from) = from else {
        return Amount::Keep(qty);
    };
    if canonical_unit(from) == canonical_unit(to) {
        return Amount::Keep(qty);
    }
    let Some(qty) = qty else {
        return Amount::Keep(None);
    };
    match (measure(from), measure(to)) {
        (Some((a, per_from)), Some((b, per_to))) if a == b => {
            let converted = (qty * per_from / per_to).round_dp(2);
            if converted > Decimal::ZERO {
                Amount::Converted(converted)
            } else {
                Amount::Unrelated
            }
        }
        _ => Amount::Unrelated,
    }
}
