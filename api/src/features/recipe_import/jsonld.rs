use serde_json::Value;

/// A schema.org Recipe as found on a page, before any matching against saved data.
#[derive(Debug, PartialEq)]
pub struct ParsedRecipe {
    pub title: String,
    pub time_mins: Option<i16>,
    pub servings: Option<i16>,
    pub ingredient_lines: Vec<String>,
}

const MAX_DEPTH: usize = 8;

/// The first schema.org Recipe in the page's JSON-LD blocks, if any.
pub fn parse_recipe(html: &str) -> Option<ParsedRecipe> {
    script_blocks(html).into_iter().find_map(|block| {
        let value: Value = serde_json::from_str(clean_block(&block)).ok()?;
        find_recipe(&value, 0).and_then(read_recipe)
    })
}

/// The bodies of every `<script type="application/ld+json">` element. Scanned by hand: the page
/// is only searched for these blocks, never rendered.
fn script_blocks(html: &str) -> Vec<String> {
    // ASCII lowercasing keeps byte offsets identical, so positions found here index `html`.
    let lower = html.to_ascii_lowercase();
    let mut blocks = Vec::new();
    let mut from = 0;
    while let Some(open) = lower[from..].find("<script") {
        let tag_start = from + open;
        let Some(tag_len) = lower[tag_start..].find('>') else {
            break;
        };
        let body_start = tag_start + tag_len + 1;
        let tag = &lower[tag_start..body_start];
        let end = lower[body_start..]
            .find("</script")
            .map_or(html.len(), |i| body_start + i);
        if tag.contains("ld+json") {
            blocks.push(html[body_start..end].to_string());
        }
        from = end;
    }
    blocks
}

/// Some sites wrap the JSON in an HTML comment or a CDATA section.
fn clean_block(block: &str) -> &str {
    let mut text = block.trim();
    for (start, end) in [
        ("<!--", "-->"),
        ("//<![CDATA[", "//]]>"),
        ("<![CDATA[", "]]>"),
    ] {
        if let Some(inner) = text.strip_prefix(start) {
            text = inner.strip_suffix(end).unwrap_or(inner).trim();
        }
    }
    text
}

fn is_recipe_type(value: &Value) -> bool {
    match value.get("@type") {
        Some(Value::String(t)) => type_is_recipe(t),
        Some(Value::Array(types)) => types.iter().any(|t| t.as_str().is_some_and(type_is_recipe)),
        _ => false,
    }
}

fn type_is_recipe(t: &str) -> bool {
    t.rsplit(['/', '#']).next() == Some("Recipe")
}

/// Walks arrays, `@graph` and nested objects (a Recipe is often the `mainEntity` of a page).
fn find_recipe(value: &Value, depth: usize) -> Option<&Value> {
    if depth > MAX_DEPTH {
        return None;
    }
    match value {
        Value::Array(items) => items.iter().find_map(|v| find_recipe(v, depth + 1)),
        Value::Object(map) => {
            if is_recipe_type(value) {
                return Some(value);
            }
            map.get("@graph")
                .and_then(|g| find_recipe(g, depth + 1))
                .or_else(|| map.values().find_map(|v| find_recipe(v, depth + 1)))
        }
        _ => None,
    }
}

fn read_recipe(recipe: &Value) -> Option<ParsedRecipe> {
    let title = text_of(recipe.get("name")?)?;
    let ingredient_lines = recipe
        .get("recipeIngredient")
        .or_else(|| recipe.get("ingredients"))
        .map(string_list)
        .unwrap_or_default();
    Some(ParsedRecipe {
        title,
        time_mins: read_time(recipe),
        servings: recipe.get("recipeYield").and_then(read_servings),
        ingredient_lines,
    })
}

fn text_of(value: &Value) -> Option<String> {
    let text = match value {
        Value::String(s) => s.as_str(),
        Value::Array(items) => return items.iter().find_map(text_of),
        _ => return None,
    };
    let text = collapse_whitespace(&decode_entities(text));
    (!text.is_empty()).then_some(text)
}

fn string_list(value: &Value) -> Vec<String> {
    match value {
        Value::Array(items) => items.iter().filter_map(text_of).collect(),
        other => text_of(other).into_iter().collect(),
    }
}

fn read_time(recipe: &Value) -> Option<i16> {
    let minutes = |key: &str| {
        recipe
            .get(key)
            .and_then(Value::as_str)
            .and_then(iso_duration_minutes)
    };
    let total = minutes("totalTime").filter(|m| *m > 0);
    let parts = match (minutes("prepTime"), minutes("cookTime")) {
        (None, None) => None,
        (prep, cook) => Some(prep.unwrap_or(0) + cook.unwrap_or(0)),
    }
    .filter(|m| *m > 0);
    i16::try_from(total.or(parts)?).ok()
}

/// Whole minutes in an ISO 8601 duration such as `PT1H30M` or `P1DT2H`; seconds round up.
pub fn iso_duration_minutes(text: &str) -> Option<i64> {
    let rest = text.trim().to_ascii_uppercase();
    let mut rest = rest.strip_prefix('P')?;
    let mut minutes = 0i64;
    let mut in_time = false;
    let mut digits = String::new();
    while let Some(c) = rest.chars().next() {
        rest = &rest[c.len_utf8()..];
        match c {
            'T' => in_time = true,
            '0'..='9' | '.' => digits.push(c),
            unit => {
                let n: f64 = digits.parse().ok()?;
                digits.clear();
                let per_unit = match (unit, in_time) {
                    ('W', false) => 7.0 * 24.0 * 60.0,
                    ('D', false) => 24.0 * 60.0,
                    ('H', true) => 60.0,
                    ('M', true) => 1.0,
                    ('S', true) => 1.0 / 60.0,
                    _ => return None,
                };
                minutes += (n * per_unit).ceil() as i64;
            }
        }
    }
    digits.is_empty().then_some(minutes)
}

fn read_servings(value: &Value) -> Option<i16> {
    let text = match value {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(items) => return items.iter().find_map(read_servings),
        _ => return None,
    };
    let digits: String = text
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse::<i16>().ok().filter(|n| *n >= 1)
}

pub fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Replaces the HTML entities that turn up in recipe text: the named basics and numeric ones.
pub fn decode_entities(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        rest = &rest[amp..];
        let decoded = rest[1..]
            .find(';')
            .filter(|end| *end <= 8)
            .and_then(|end| entity(&rest[1..1 + end]).map(|c| (c, end + 2)));
        match decoded {
            Some((c, consumed)) => {
                out.push(c);
                rest = &rest[consumed..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn entity(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some(' '),
        "frac12" => Some('½'),
        "frac14" => Some('¼'),
        "frac34" => Some('¾'),
        _ => {
            let number = name.strip_prefix('#')?;
            let code = match number.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => number.parse().ok()?,
            };
            char::from_u32(code)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(json: &str) -> String {
        format!(
            r#"<html><head><title>x</title><script type="application/ld+json">{json}</script></head><body></body></html>"#
        )
    }

    fn parse(json: &str) -> Option<ParsedRecipe> {
        parse_recipe(&page(json))
    }

    #[test]
    fn reads_a_plain_recipe_object() {
        let recipe = parse(include_str!("fixtures/plain.json")).unwrap();
        assert_eq!(recipe.title, "Pad Thai");
        assert_eq!(recipe.time_mins, Some(45));
        assert_eq!(recipe.servings, Some(4));
        assert_eq!(
            recipe.ingredient_lines,
            vec!["200 g rice noodles", "2 eggs", "1 tbsp tamarind paste"]
        );
    }

    #[test]
    fn finds_the_recipe_inside_a_graph() {
        let recipe = parse(include_str!("fixtures/graph.json")).unwrap();
        assert_eq!(recipe.title, "Tom & Jerry's Soup");
        assert_eq!(recipe.time_mins, Some(75));
        assert_eq!(recipe.servings, Some(6));
        assert_eq!(recipe.ingredient_lines.len(), 2);
    }

    #[test]
    fn finds_the_recipe_in_a_top_level_array_with_a_type_list() {
        let recipe = parse(include_str!("fixtures/array.json")).unwrap();
        assert_eq!(recipe.title, "Shakshuka");
        assert_eq!(recipe.time_mins, Some(30));
        assert_eq!(recipe.servings, Some(2));
    }

    #[test]
    fn finds_a_recipe_nested_as_the_main_entity() {
        let json = r#"{"@type":"WebPage","mainEntity":{"@type":"Recipe","name":"Dal","recipeIngredient":["1 cup lentils"]}}"#;
        assert_eq!(parse(json).unwrap().title, "Dal");
    }

    #[test]
    fn uses_the_second_block_when_the_first_has_no_recipe() {
        let html = format!(
            "{}{}",
            page(r#"{"@type":"Organization","name":"Site"}"#),
            page(r#"{"@type":"Recipe","name":"Soup","recipeIngredient":[]}"#)
        );
        assert_eq!(parse_recipe(&html).unwrap().title, "Soup");
    }

    #[test]
    fn adds_prep_and_cook_time_when_there_is_no_total() {
        let json = r#"{"@type":"Recipe","name":"Stew","prepTime":"PT15M","cookTime":"PT1H","recipeYield":"Serves 4-6"}"#;
        let recipe = parse(json).unwrap();
        assert_eq!(recipe.time_mins, Some(75));
        assert_eq!(recipe.servings, Some(4));
    }

    #[test]
    fn leaves_missing_fields_empty() {
        let recipe = parse(r#"{"@type":"Recipe","name":"Toast"}"#).unwrap();
        assert_eq!(recipe.time_mins, None);
        assert_eq!(recipe.servings, None);
        assert!(recipe.ingredient_lines.is_empty());
    }

    #[test]
    fn accepts_a_numeric_yield_and_a_yield_list() {
        let a = parse(r#"{"@type":"Recipe","name":"A","recipeYield":8}"#).unwrap();
        let b =
            parse(r#"{"@type":"Recipe","name":"B","recipeYield":["12","12 muffins"]}"#).unwrap();
        assert_eq!((a.servings, b.servings), (Some(8), Some(12)));
    }

    #[test]
    fn tolerates_comment_and_cdata_wrappers() {
        let recipe = parse("<!-- {\"@type\":\"Recipe\",\"name\":\"Wrapped\"} -->").unwrap();
        assert_eq!(recipe.title, "Wrapped");
        let recipe =
            parse("//<![CDATA[\n{\"@type\":\"Recipe\",\"name\":\"Cdata\"}\n//]]>").unwrap();
        assert_eq!(recipe.title, "Cdata");
    }

    #[test]
    fn matches_the_script_tag_case_insensitively() {
        let html =
            r#"<SCRIPT TYPE="application/LD+JSON">{"@type":"Recipe","name":"Loud"}</SCRIPT>"#;
        assert_eq!(parse_recipe(html).unwrap().title, "Loud");
    }

    #[test]
    fn ignores_pages_without_a_recipe_or_with_broken_json() {
        assert!(parse_recipe("<p>nothing</p>").is_none());
        assert!(parse(r#"{"@type":"Article","name":"News"}"#).is_none());
        assert!(parse("{not json").is_none());
        assert!(parse(r#"{"@type":"Recipe"}"#).is_none());
    }

    #[test]
    fn parses_iso_durations() {
        assert_eq!(iso_duration_minutes("PT20M"), Some(20));
        assert_eq!(iso_duration_minutes("PT1H30M"), Some(90));
        assert_eq!(iso_duration_minutes("P1DT2H"), Some(1560));
        assert_eq!(iso_duration_minutes("PT90S"), Some(2));
        assert_eq!(iso_duration_minutes("PT"), Some(0));
        assert_eq!(iso_duration_minutes("45 minutes"), None);
        assert_eq!(iso_duration_minutes("PT5"), None);
    }

    #[test]
    fn decodes_entities() {
        assert_eq!(
            decode_entities("Tom &amp; Jerry&#39;s &frac12; cup"),
            "Tom & Jerry's ½ cup"
        );
        assert_eq!(decode_entities("a &bogus; b & c"), "a &bogus; b & c");
    }
}
