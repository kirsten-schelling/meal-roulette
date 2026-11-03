//! Every rolled week, newest first, split into active, favourite and archived
//! tabs. A row opens its plan; the star, archive and delete act in place.

use crate::components::ui::{
    Alert, Button, ButtonVariant, CellAlign, Column, ConfirmDialog, DataTable, Heading, Icon,
    IconButton, IconName, Size, Spinner, Tabs, Text, Tone,
};
use crate::features::history::components::FavouriteStar;
use crate::features::history::format::{dinner_count, plan_title};
use crate::features::history::hooks::use_history::{
    delete_plan, set_plan_archived, set_plan_favourite, use_get_plan_history,
};
use crate::features::history::hooks::use_local_offset::use_local_offset;
use crate::hooks::use_toast::use_toast;
use crate::models::error_message;
use crate::router::Route;
use dioxus::prelude::*;
use domain::meal_plan::MealPlanSummary;
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq)]
enum HistoryTab {
    Active,
    Favourites,
    Archived,
}

const TABS: [HistoryTab; 3] = [
    HistoryTab::Active,
    HistoryTab::Favourites,
    HistoryTab::Archived,
];

impl HistoryTab {
    fn label(self) -> &'static str {
        match self {
            HistoryTab::Active => "Active",
            HistoryTab::Favourites => "Favourites",
            HistoryTab::Archived => "Archived",
        }
    }

    fn includes(self, plan: &MealPlanSummary) -> bool {
        let archived = plan.archived_at.is_some();
        match self {
            HistoryTab::Active => !archived,
            HistoryTab::Favourites => plan.is_favourite && !archived,
            HistoryTab::Archived => archived,
        }
    }
}

/// The empty state for a tab with no plans in it.
fn empty_tab(tab: HistoryTab) -> Element {
    let (icon, title, hint) = match tab {
        HistoryTab::Active => (
            IconName::Dice,
            "No active weeks",
            "Roll a week and it shows up here.",
        ),
        HistoryTab::Favourites => (
            IconName::Star,
            "No favourites yet",
            "Star a week you liked to keep it here.",
        ),
        HistoryTab::Archived => (
            IconName::Archive,
            "Nothing archived",
            "Archive a week to tuck it away without deleting it.",
        ),
    };
    rsx! {
        div { class: "mr-empty",
            div { class: "mr-empty__icon", aria_hidden: "true", Icon { name: icon } }
            Heading { level: 3, "{title}" }
            Text { muted: true, "{hint}" }
            if tab == HistoryTab::Active {
                Link { to: Route::Roll {}, class: "mr-btn mr-btn--solid mr-btn--lg", "Roll a week" }
            }
        }
    }
}

#[component]
pub fn History() -> Element {
    let mut history = use_get_plan_history();
    let offset = use_local_offset();
    let toast = use_toast();
    let navigator = use_navigator();
    let mut tab = use_signal(|| HistoryTab::Active);
    // The plan whose star or archive change is in flight. One change at a
    // time: the other controls wait until it lands.
    let mut starring = use_signal(|| None as Option<Uuid>);
    let mut archiving = use_signal(|| None as Option<Uuid>);
    let busy = move || starring().is_some() || archiving().is_some();
    // The plan waiting on the delete confirmation; `Some` mounts the dialog.
    let mut deleting = use_signal(|| None as Option<MealPlanSummary>);
    let mut delete_busy = use_signal(|| false);

    let mut toggle_favourite = move |plan: MealPlanSummary| {
        if busy() {
            return;
        }
        let name = plan_title(plan.name.as_deref(), plan.created_at, offset());
        starring.set(Some(plan.id));
        spawn(async move {
            let on = !plan.is_favourite;
            match set_plan_favourite(plan.id, on).await {
                Ok(_) => {
                    history.restart();
                    let message = if on {
                        format!("{name} is in Favourites.")
                    } else {
                        format!("{name} is no longer a favourite.")
                    };
                    toast.show(
                        Some(if on { "Starred" } else { "Unstarred" }.to_string()),
                        message,
                        Tone::Green,
                    );
                }
                Err(e) => toast.show(
                    Some("Couldn't update the favourite".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            starring.set(None);
        });
    };

    let mut toggle_archived = move |plan: MealPlanSummary| {
        if busy() {
            return;
        }
        let name = plan_title(plan.name.as_deref(), plan.created_at, offset());
        archiving.set(Some(plan.id));
        spawn(async move {
            let archive = plan.archived_at.is_none();
            match set_plan_archived(plan.id, archive).await {
                Ok(_) => {
                    history.restart();
                    let (title, message) = if archive {
                        ("Archived", format!("{name} moved to Archived."))
                    } else {
                        ("Unarchived", format!("{name} is back in Active."))
                    };
                    toast.show(Some(title.to_string()), message, Tone::Green);
                }
                Err(e) => toast.show(
                    Some(
                        if archive {
                            "Couldn't archive"
                        } else {
                            "Couldn't unarchive"
                        }
                        .to_string(),
                    ),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            archiving.set(None);
        });
    };

    let mut delete_confirmed = move || {
        let Some(plan) = deleting() else {
            return;
        };
        if delete_busy() {
            return;
        }
        let name = plan_title(plan.name.as_deref(), plan.created_at, offset());
        delete_busy.set(true);
        spawn(async move {
            match delete_plan(plan.id).await {
                Ok(_) => {
                    history.restart();
                    toast.show(
                        Some("Deleted".to_string()),
                        format!("{name} was deleted."),
                        Tone::Green,
                    );
                }
                Err(e) => toast.show(
                    Some("Couldn't delete".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            deleting.set(None);
            delete_busy.set(false);
        });
    };

    let columns: Vec<Column<MealPlanSummary>> = vec![
        Column::new("Plan", move |p: &MealPlanSummary| {
            rsx! {
                span { class: "mr-cell-strong mr-nowrap",
                    "{plan_title(p.name.as_deref(), p.created_at, offset())}"
                }
            }
        }),
        Column::new("Dinners", |p: &MealPlanSummary| {
            let titles = p.dinners.join(", ");
            rsx! {
                div { class: "mr-dinners", title: "{titles}",
                    span { class: "mr-dinners__count", "{dinner_count(p.num_days)}" }
                    span { class: "mr-dinners__titles", "{titles}" }
                }
            }
        }),
        Column::new("Seed", |p: &MealPlanSummary| match p.seed {
            Some(seed) => rsx! { "{seed}" },
            None => rsx! {
                span { class: "mr-text--muted", "none" }
            },
        })
        .align(CellAlign::End),
        Column::new("", move |p: &MealPlanSummary| {
            let plan = p.clone();
            rsx! {
                FavouriteStar {
                    on: p.is_favourite,
                    busy: starring() == Some(p.id),
                    onclick: move |_| toggle_favourite(plan.clone()),
                }
            }
        })
        .interactive(),
        Column::new("", move |p: &MealPlanSummary| {
            let plan = p.clone();
            let doomed = p.clone();
            let archived = p.archived_at.is_some();
            rsx! {
                div { class: "mr-rowactions",
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: Size::Sm,
                        loading: archiving() == Some(p.id),
                        disabled: busy(),
                        onclick: move |_| toggle_archived(plan.clone()),
                        if archived {
                            "Unarchive"
                        } else {
                            "Archive"
                        }
                    }
                    IconButton {
                        label: "Delete this week".to_string(),
                        danger: true,
                        onclick: move |_| deleting.set(Some(doomed.clone())),
                        Icon { name: IconName::X }
                    }
                }
            }
        })
        .align(CellAlign::End)
        .interactive(),
    ];

    let loaded = history();
    let plans: &[MealPlanSummary] = match &loaded {
        Some(Ok(list)) => list,
        _ => &[],
    };
    let current = tab();
    let labels: Vec<String> = TABS
        .iter()
        .map(|t| {
            let count = plans.iter().filter(|p| t.includes(p)).count();
            format!("{} ({count})", t.label())
        })
        .collect();
    let selected = TABS.iter().position(|t| *t == current).unwrap_or(0);
    let visible: Vec<MealPlanSummary> = plans
        .iter()
        .filter(|p| current.includes(p))
        .cloned()
        .collect();
    let visible_ids: Vec<Uuid> = visible.iter().map(|p| p.id).collect();
    let open_plan = move |index: usize| {
        if let Some(&id) = visible_ids.get(index) {
            navigator.push(Route::PlanDetail { id });
        }
    };

    rsx! {
        div { class: "mr-page",
            div { class: "mr-pagehead",
                div { class: "mr-pagehead__titles",
                    Heading { level: 1, "History" }
                    Text { muted: true,
                        "Every week you've rolled. Open one to see its dinners, star the ones worth repeating."
                    }
                }
                div { class: "mr-pagehead__actions",
                    Link {
                        to: Route::Roll {},
                        class: "mr-btn mr-btn--outline mr-btn--md",
                        "Roll a new week"
                    }
                }
            }

            div { class: "mr-history__tabs",
                Tabs {
                    labels,
                    selected,
                    on_select: move |i: usize| tab.set(TABS.get(i).copied().unwrap_or(HistoryTab::Active)),
                }
            }

            match &loaded {
                None => rsx! {
                    div { class: "mr-tablepanel mr-tablestate", Spinner {} }
                },
                Some(Err(e)) => rsx! {
                    Alert { tone: Tone::Red, title: "Couldn't load the history", "{error_message(e)}" }
                },
                Some(Ok(_)) if visible.is_empty() => empty_tab(current),
                Some(Ok(_)) => rsx! {
                    div { class: "mr-tablepanel",
                        DataTable::<MealPlanSummary> {
                            columns: columns.clone(),
                            rows: visible.clone(),
                            on_row_click: open_plan,
                        }
                    }
                },
            }

            if let Some(plan) = deleting() {
                ConfirmDialog {
                    title: format!("Delete {}?", plan_title(plan.name.as_deref(), plan.created_at, offset())),
                    confirm_label: "Delete".to_string(),
                    danger: true,
                    loading: delete_busy(),
                    on_confirm: move |_| delete_confirmed(),
                    on_cancel: move |_| {
                        if !delete_busy() {
                            deleting.set(None);
                        }
                    },
                    Text {
                        "This removes the plan and its {dinner_count(plan.num_days)} for good. Its shopping list and cooking-log entries are kept, and its recipes can be deleted again unless another plan or the cooking log still uses them."
                    }
                }
            }
        }
    }
}
