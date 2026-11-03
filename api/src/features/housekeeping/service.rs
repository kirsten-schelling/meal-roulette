use crate::features::{cuisine, ingredient, protein, tag, unit};
use domain::housekeeping::{Finding, LookupKind};
use sqlx::PgPool;

/// Shorter names differ by one character too easily ("g" / "kg", "ml" / "mg")
/// to treat as typos.
const MIN_FUZZY_LEN: usize = 5;

pub async fn collect(pool: &PgPool) -> Result<Vec<Finding>, sqlx::Error> {
    let cuisines = cuisine::repository::list(pool).await?;
    let proteins = protein::repository::list(pool).await?;
    let units = unit::repository::list(pool).await?;
    let tags = tag::repository::list(pool).await?;
    let ingredients = ingredient::repository::list(pool).await?;

    let cuisine_names: Vec<&str> = cuisines.iter().map(|c| c.cuisine.name.as_str()).collect();
    let protein_names: Vec<&str> = proteins.iter().map(|p| p.protein.name.as_str()).collect();
    let unit_names: Vec<&str> = units.iter().map(|u| u.unit.name.as_str()).collect();
    let tag_names: Vec<&str> = tags.iter().map(|t| t.tag.name.as_str()).collect();
    let ingredient_names: Vec<&str> = ingredients.iter().map(|i| i.name.as_str()).collect();

    let mut findings = Vec::new();
    push_unused(
        &mut findings,
        LookupKind::Cuisine,
        cuisines
            .iter()
            .map(|c| (c.cuisine.name.as_str(), c.used_by.len())),
    );
    push_unused(
        &mut findings,
        LookupKind::Protein,
        proteins
            .iter()
            .map(|p| (p.protein.name.as_str(), p.used_by.len())),
    );
    push_unused(
        &mut findings,
        LookupKind::Unit,
        units
            .iter()
            .map(|u| (u.unit.name.as_str(), u.used_by.len())),
    );
    push_unused(
        &mut findings,
        LookupKind::Tag,
        tags.iter().map(|t| (t.tag.name.as_str(), t.used_by.len())),
    );

    for (kind, names) in [
        (LookupKind::Cuisine, &cuisine_names),
        (LookupKind::Protein, &protein_names),
        (LookupKind::Unit, &unit_names),
        (LookupKind::Tag, &tag_names),
        (LookupKind::Ingredient, &ingredient_names),
    ] {
        let pairs = near_duplicates(names);
        if !pairs.is_empty() {
            findings.push(Finding::NearDuplicates { kind, pairs });
        }
    }

    let missing: Vec<String> = ingredients
        .iter()
        .filter(|i| i.purchase_size.is_none())
        .map(|i| i.name.clone())
        .collect();
    if !missing.is_empty() {
        findings.push(Finding::MissingPurchaseSize { names: missing });
    }
    Ok(findings)
}

fn push_unused<'a>(
    findings: &mut Vec<Finding>,
    kind: LookupKind,
    values: impl Iterator<Item = (&'a str, usize)>,
) {
    let names: Vec<String> = values
        .filter(|(_, uses)| *uses == 0)
        .map(|(name, _)| name.to_string())
        .collect();
    if !names.is_empty() {
        findings.push(Finding::Unused { kind, names });
    }
}

fn normalize(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub fn near_duplicates(names: &[&str]) -> Vec<(String, String)> {
    let normalized: Vec<String> = names.iter().map(|n| normalize(n)).collect();
    let mut pairs = Vec::new();
    for i in 0..names.len() {
        for j in (i + 1)..names.len() {
            let (a, b) = (&normalized[i], &normalized[j]);
            let same = a == b;
            let typo = a.chars().count() >= MIN_FUZZY_LEN
                && b.chars().count() >= MIN_FUZZY_LEN
                && within_one_edit(a, b);
            if same || typo {
                pairs.push((names[i].to_string(), names[j].to_string()));
            }
        }
    }
    pairs
}

/// True when `a` and `b` differ by exactly one insertion, deletion or substitution.
fn within_one_edit(a: &str, b: &str) -> bool {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > 1 {
        return false;
    }
    let prefix = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    if prefix == a.len() && prefix == b.len() {
        return false;
    }
    let (ra, rb) = (&a[prefix..], &b[prefix..]);
    match ra.len().cmp(&rb.len()) {
        std::cmp::Ordering::Equal => ra[1..] == rb[1..],
        std::cmp::Ordering::Less => ra == &rb[1..],
        std::cmp::Ordering::Greater => &ra[1..] == rb,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catches_case_and_spacing() {
        let pairs = near_duplicates(&["Garlic", "garlic ", "Onion"]);
        assert_eq!(pairs, vec![("Garlic".to_string(), "garlic ".to_string())]);
    }

    #[test]
    fn catches_one_typo() {
        let pairs = near_duplicates(&["Chinese", "Chineese", "Thai"]);
        assert_eq!(pairs, vec![("Chinese".to_string(), "Chineese".to_string())]);
    }

    #[test]
    fn leaves_short_and_distinct_names_alone() {
        assert!(near_duplicates(&["g", "kg", "ml", "mg", "Italian", "Indian"]).is_empty());
    }

    #[test]
    fn one_edit_rules() {
        assert!(within_one_edit("chinese", "chineese"));
        assert!(within_one_edit("tomato", "tomata"));
        assert!(!within_one_edit("tomato", "tomato"));
        assert!(!within_one_edit("tomato", "tomatoes"));
    }
}
