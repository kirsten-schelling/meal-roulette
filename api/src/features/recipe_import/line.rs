use super::jsonld::{collapse_whitespace, decode_entities};
use rust_decimal::Decimal;

/// One ingredient line such as `1 1/2 cups flour, sifted`, split into its parts.
#[derive(Debug, PartialEq)]
pub struct ParsedLine {
    pub qty: Option<Decimal>,
    /// The unit as written on the page, if the line had a recognised one.
    pub unit: Option<String>,
    pub name: String,
    pub note: Option<String>,
}

/// Spellings that mean the same unit, so `tablespoons` on a page finds a saved `tbsp`.
const UNIT_GROUPS: &[(&str, &[&str])] = &[
    ("tbsp", &["tablespoon", "tablespoons", "tbsp", "tbs", "tbl"]),
    ("tsp", &["teaspoon", "teaspoons", "tsp"]),
    ("cup", &["cup", "cups"]),
    ("oz", &["ounce", "ounces", "oz"]),
    ("lb", &["pound", "pounds", "lb", "lbs"]),
    ("g", &["gram", "grams", "gramme", "grammes", "g"]),
    ("kg", &["kilogram", "kilograms", "kilo", "kilos", "kg"]),
    (
        "ml",
        &[
            "milliliter",
            "milliliters",
            "millilitre",
            "millilitres",
            "ml",
        ],
    ),
    ("l", &["liter", "liters", "litre", "litres", "l"]),
    ("clove", &["clove", "cloves"]),
    ("pinch", &["pinch", "pinches"]),
    ("dash", &["dash", "dashes"]),
    ("can", &["can", "cans", "tin", "tins"]),
    ("slice", &["slice", "slices"]),
    ("stick", &["stick", "sticks"]),
    ("bunch", &["bunch", "bunches"]),
    ("sprig", &["sprig", "sprigs"]),
    ("handful", &["handful", "handfuls"]),
    (
        "package",
        &["package", "packages", "pkg", "packet", "packets"],
    ),
];

fn known_unit(word: &str) -> Option<&'static str> {
    UNIT_GROUPS
        .iter()
        .find(|(_, spellings)| spellings.contains(&word))
        .map(|(canonical, _)| *canonical)
}

/// A unit name reduced to a form two spellings of the same unit share.
pub fn canonical_unit(name: &str) -> String {
    let word = name.trim().trim_end_matches('.').to_lowercase();
    match known_unit(&word) {
        Some(canonical) => canonical.to_string(),
        None => singular(&word),
    }
}

/// An ingredient name reduced to a form its singular and plural share.
pub fn match_key(name: &str) -> String {
    singular(&collapse_whitespace(&name.to_lowercase()))
}

fn singular(word: &str) -> String {
    if let Some(stem) = word.strip_suffix("ies").filter(|s| !s.is_empty()) {
        return format!("{stem}y");
    }
    if let Some(stem) = word.strip_suffix("oes").filter(|s| !s.is_empty()) {
        return format!("{stem}o");
    }
    match word.strip_suffix('s') {
        Some(stem) if !stem.is_empty() && !stem.ends_with('s') => stem.to_string(),
        _ => word.to_string(),
    }
}

fn vulgar_fraction(c: char) -> Option<(i64, i64)> {
    Some(match c {
        '½' => (1, 2),
        '¼' => (1, 4),
        '¾' => (3, 4),
        '⅓' => (1, 3),
        '⅔' => (2, 3),
        '⅛' => (1, 8),
        '⅜' => (3, 8),
        '⅝' => (5, 8),
        '⅞' => (7, 8),
        _ => return None,
    })
}

fn ratio(top: i64, bottom: i64) -> Option<Decimal> {
    (bottom != 0).then(|| Decimal::from(top) / Decimal::from(bottom))
}

fn is_fraction(token: &str) -> bool {
    let mut chars = token.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => vulgar_fraction(c).is_some(),
        _ => token.split_once('/').is_some_and(|(a, b)| {
            !a.is_empty()
                && !b.is_empty()
                && a.bytes().all(|c| c.is_ascii_digit())
                && b.bytes().all(|c| c.is_ascii_digit())
        }),
    }
}

/// A single number token: `2`, `1.5`, `3/4`, `½` or `1½`.
fn number(token: &str) -> Option<Decimal> {
    if token.is_empty() {
        return None;
    }
    if let Some((top, bottom)) = token.split_once('/') {
        return ratio(top.parse().ok()?, bottom.parse().ok()?);
    }
    let last = token.chars().last()?;
    if let Some((top, bottom)) = vulgar_fraction(last) {
        let whole = &token[..token.len() - last.len_utf8()];
        let whole = if whole.is_empty() {
            Decimal::ZERO
        } else {
            whole.parse::<Decimal>().ok()?
        };
        return Some(whole + ratio(top, bottom)?);
    }
    if token.bytes().all(|c| c.is_ascii_digit() || c == b'.') {
        return token.parse().ok();
    }
    None
}

/// The leading quantity of `tokens` and how many tokens it used, plus a range's other end.
fn leading_quantity(tokens: &[&str]) -> Option<(Decimal, usize, Option<String>)> {
    let first = *tokens.first()?;
    if let Some((low, high)) = first.split_once(['-', '–', '—']) {
        if let (Some(low_n), Some(_)) = (number(low), number(high)) {
            return Some((low_n, 1, Some(first.to_string())));
        }
    }
    let mut qty = number(first)?;
    let mut used = 1;
    let whole = first.bytes().all(|c| c.is_ascii_digit());
    if let Some(next) = tokens.get(1).filter(|t| whole && is_fraction(t)) {
        qty += number(next)?;
        used = 2;
    }
    if tokens
        .get(used)
        .is_some_and(|t| t.eq_ignore_ascii_case("to"))
    {
        if let Some(high) = tokens.get(used + 1).filter(|t| number(t).is_some()) {
            let range = format!("{first} to {high}");
            return Some((qty, used + 2, Some(range)));
        }
    }
    Some((qty, used, None))
}

/// Removes `( ... )` groups, returning the text without them and what they said.
fn take_parentheses(text: &str) -> (String, Vec<String>) {
    let mut kept = String::new();
    let mut notes = Vec::new();
    let mut depth = 0usize;
    let mut current = String::new();
    for c in text.chars() {
        match c {
            '(' => {
                if depth == 0 {
                    kept.push(' ');
                } else {
                    current.push(c);
                }
                depth += 1;
            }
            ')' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    let inner = collapse_whitespace(&current);
                    if !inner.is_empty() {
                        notes.push(inner);
                    }
                    current.clear();
                } else {
                    current.push(c);
                }
            }
            _ if depth > 0 => current.push(c),
            _ => kept.push(c),
        }
    }
    (collapse_whitespace(&kept), notes)
}

pub fn parse_line(raw: &str) -> ParsedLine {
    let text = collapse_whitespace(&decode_entities(raw));
    let (without_parens, mut notes) = take_parentheses(&text);
    let (main, after_comma) = match without_parens.split_once(',') {
        Some((main, rest)) => (main.trim().to_string(), Some(rest.trim().to_string())),
        None => (without_parens, None),
    };

    let tokens: Vec<&str> = main.split_whitespace().collect();
    let mut at = 0;
    let mut qty = None;
    if let Some((value, used, range)) = leading_quantity(&tokens) {
        qty = Some(value.round_dp(2)).filter(|q| *q > Decimal::ZERO);
        at = used;
        notes.splice(0..0, range);
    }

    let mut unit = None;
    if let Some(word) = tokens.get(at) {
        let word = word.trim_end_matches('.').to_lowercase();
        let allowed = qty.is_some() || matches!(word.as_str(), "pinch" | "dash");
        if allowed && known_unit(&word).is_some() {
            unit = Some(word);
            at += 1;
        }
    }
    if unit.is_some() && tokens.get(at).is_some_and(|t| t.eq_ignore_ascii_case("of")) {
        at += 1;
    }

    let mut name = tokens[at..].join(" ");
    for tail in [" to taste", " as needed"] {
        if name.to_lowercase().ends_with(tail) {
            name.truncate(name.len() - tail.len());
            notes.push(tail.trim().to_string());
        }
    }
    let name = name
        .trim_matches(|c: char| c.is_whitespace() || ".:;-".contains(c))
        .to_string();
    if let Some(rest) = after_comma.filter(|r| !r.is_empty()) {
        notes.push(rest);
    }

    ParsedLine {
        qty,
        unit,
        name: if name.is_empty() { text } else { name },
        note: (!notes.is_empty()).then(|| notes.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(s: &str) -> Option<Decimal> {
        Some(s.parse().unwrap())
    }

    fn line(text: &str) -> (Option<Decimal>, Option<String>, String, Option<String>) {
        let p = parse_line(text);
        (p.qty, p.unit, p.name, p.note)
    }

    #[test]
    fn splits_quantity_unit_and_name() {
        assert_eq!(
            line("200 g rice noodles"),
            (dec("200"), Some("g".into()), "rice noodles".into(), None)
        );
        assert_eq!(
            line("2 cups chicken stock"),
            (dec("2"), Some("cups".into()), "chicken stock".into(), None)
        );
    }

    #[test]
    fn counts_have_no_unit() {
        assert_eq!(line("2 eggs"), (dec("2"), None, "eggs".into(), None));
    }

    #[test]
    fn reads_fractions_decimals_and_mixed_numbers() {
        assert_eq!(parse_line("1/2 tsp salt").qty, dec("0.5"));
        assert_eq!(parse_line("1 1/2 cups flour").qty, dec("1.5"));
        assert_eq!(parse_line("½ cup milk").qty, dec("0.5"));
        assert_eq!(parse_line("1½ cups milk").qty, dec("1.5"));
        assert_eq!(parse_line("0.25 kg butter").qty, dec("0.25"));
        assert_eq!(parse_line("1/3 cup sugar").qty, dec("0.33"));
    }

    #[test]
    fn keeps_a_range_in_the_note() {
        let p = parse_line("2-3 tbsp olive oil");
        assert_eq!(p.qty, dec("2"));
        assert_eq!(p.note.as_deref(), Some("2-3"));
        let p = parse_line("1 to 2 cloves garlic");
        assert_eq!((p.qty, p.unit.as_deref()), (dec("1"), Some("cloves")));
        assert_eq!(p.note.as_deref(), Some("1 to 2"));
    }

    #[test]
    fn moves_parentheses_and_comma_text_into_the_note() {
        assert_eq!(
            line("1 (14 oz) can chopped tomatoes"),
            (
                dec("1"),
                Some("can".into()),
                "chopped tomatoes".into(),
                Some("14 oz".into())
            )
        );
        assert_eq!(
            line("1 large onion, finely diced"),
            (
                dec("1"),
                None,
                "large onion".into(),
                Some("finely diced".into())
            )
        );
    }

    #[test]
    fn lines_without_a_quantity_keep_the_whole_name() {
        assert_eq!(
            line("Salt, to taste"),
            (None, None, "Salt".into(), Some("to taste".into()))
        );
        assert_eq!(
            line("Pinch of salt"),
            (None, Some("pinch".into()), "salt".into(), None)
        );
        assert_eq!(
            line("fresh basil to taste"),
            (None, None, "fresh basil".into(), Some("to taste".into()))
        );
    }

    #[test]
    fn drops_the_word_of_after_a_unit() {
        assert_eq!(parse_line("2 cups of water").name, "water");
    }

    #[test]
    fn a_unit_word_without_a_quantity_stays_in_the_name() {
        assert_eq!(parse_line("cup cakes").name, "cup cakes");
    }

    #[test]
    fn decodes_entities_in_lines() {
        assert_eq!(parse_line("1 cup mac &amp; cheese").name, "mac & cheese");
    }

    #[test]
    fn canonical_units_and_names_ignore_spelling_variants() {
        assert_eq!(canonical_unit("Tablespoons"), canonical_unit("tbsp"));
        assert_eq!(canonical_unit("cups"), canonical_unit("Cup"));
        assert_eq!(canonical_unit("each"), "each");
        assert_eq!(match_key("Tomatoes"), match_key("tomato"));
        assert_eq!(match_key("Berries"), match_key("berry"));
        assert_eq!(match_key("  Egg  "), "egg");
    }
}
