use crate::components::ui::*;
use crate::features::housekeeping::hooks::use_housekeeping::use_get_housekeeping;
use crate::router::Route;
use dioxus::prelude::*;
use domain::housekeeping::{Finding, LookupKind};

fn plural(kind: LookupKind) -> &'static str {
    match kind {
        LookupKind::Cuisine => "cuisines",
        LookupKind::Protein => "proteins",
        LookupKind::Unit => "units",
        LookupKind::Tag => "tags",
        LookupKind::Ingredient => "ingredients",
    }
}

/// The manage-dialog tab a lookup list opens on; ingredients have no dialog.
fn manage_param(kind: LookupKind) -> String {
    match kind {
        LookupKind::Ingredient => String::new(),
        other => plural(other).to_string(),
    }
}

fn describe(finding: &Finding) -> (String, String, String) {
    match finding {
        Finding::Unused { kind, names } => (
            format!("{} unused {}", names.len(), plural(*kind)),
            names.join(", "),
            manage_param(*kind),
        ),
        Finding::NearDuplicates { kind, pairs } => (
            format!("Possible duplicate {}", plural(*kind)),
            pairs
                .iter()
                .map(|(a, b)| format!("{a} / {b}"))
                .collect::<Vec<_>>()
                .join(", "),
            manage_param(*kind),
        ),
        Finding::MissingPurchaseSize { names } => (
            format!("{} ingredients have no purchase size", names.len()),
            names.join(", "),
            String::new(),
        ),
    }
}

/// Library upkeep on Home. Renders nothing while loading, on error, or when
/// there is nothing to report.
#[component]
pub fn HousekeepingCard() -> Element {
    let findings = use_get_housekeeping();

    let Some(Ok(list)) = findings() else {
        return rsx! {};
    };
    if list.is_empty() {
        return rsx! {};
    }

    rsx! {
        Card { header: "Library housekeeping".to_string(),
            div { class: "mr-housekeeping",
                for (i, finding) in list.iter().enumerate() {
                    {
                        let (title, detail, manage) = describe(finding);
                        rsx! {
                            Link {
                                key: "{i}",
                                to: Route::Library { manage },
                                class: "mr-housekeeping__item",
                                span { class: "mr-cell-strong", "{title}" }
                                span { class: "mr-housekeeping__detail", "{detail}" }
                                span { class: "mr-dashlink__arrow", Icon { name: IconName::ChevronRight } }
                            }
                        }
                    }
                }
            }
        }
    }
}
