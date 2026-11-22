//! A plan's shopping list: every ingredient its dinners need, summed across
//! the recipes. Lines are ticked off as they are bought; ingredients the
//! household already has can be slept for a few weeks, which keeps them
//! listed but marked as not being added. `/shopping-list` shows the latest
//! active plan's list; `/shopping-list/:plan_id` shows any plan's.

use std::collections::HashMap;

use crate::components::ui::{
    ActionMenu, ActionMenuItem, Alert, Badge, Button, ButtonVariant, ConfirmDialog, Heading, Icon,
    IconName, Size, Spinner, Text, Tone,
};
use crate::features::history::format::{dinner_count, plan_title};
use crate::features::history::hooks::use_local_offset::use_local_offset;
use crate::features::shopping_list::components::{
    AddListItem, CategoryPicker, LineRow, ManageCategoriesDialog, SleepBar,
};
use crate::features::shopping_list::hooks::use_shopping_categories::use_shopping_categories;
use crate::features::shopping_list::hooks::use_shopping_list::{
    add_covered_item, add_list_line, clear_bought_items, generate_shopping_list,
    remove_custom_item, set_item_purchased, set_line_category, sleep_ingredients,
    use_get_shopping_list, wake_ingredient,
};
use crate::hooks::use_toast::use_toast;
use crate::models::{date_label, error_message};
use crate::router::Route;
use dioxus::prelude::*;
use domain::shopping_category::ShoppingCategory;
use domain::shopping_list::ShoppingListForPlan;
use domain::shopping_list_item::{NewListLine, ShoppingListLine};
use time::UtcOffset;
use uuid::Uuid;

type Loaded = Option<Result<Option<ShoppingListForPlan>, ServerFnError>>;

#[component]
pub fn ShoppingList() -> Element {
    rsx! {
        ShoppingListView { plan_id: None }
    }
}

#[component]
pub fn PlanShoppingList(plan_id: Uuid) -> Element {
    rsx! {
        ShoppingListView { plan_id: Some(plan_id) }
    }
}

/// The lines the table shows, in table order. Empty until a list is loaded.
fn lines(loaded: &Loaded) -> Vec<ShoppingListLine> {
    match loaded {
        Some(Ok(Some(view))) => view
            .list
            .as_ref()
            .map(|detail| detail.items.clone())
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

/// The lines under each category in list order, then the uncategorised ones. A section
/// is named only when some line has a category.
fn sections(
    lines: Vec<ShoppingListLine>,
    categories: &[ShoppingCategory],
) -> Vec<(Option<String>, Vec<ShoppingListLine>)> {
    let (mut rest, mut named) = (lines, Vec::new());
    for category in categories {
        let (inside, outside): (Vec<_>, Vec<_>) = rest
            .into_iter()
            .partition(|line| line.category_id == Some(category.id));
        rest = outside;
        if !inside.is_empty() {
            named.push((Some(category.name.clone()), inside));
        }
    }
    if rest.is_empty() {
        return named;
    }
    let label = (!named.is_empty()).then(|| "Other".to_string());
    named.push((label, rest));
    named
}

fn page_subtitle(view: Option<&ShoppingListForPlan>, offset: UtcOffset) -> String {
    match view {
        Some(view) => {
            let plan = &view.plan;
            let archived = if plan.archived_at.is_some() {
                " This plan is archived."
            } else {
                ""
            };
            format!(
                "For {}, {}.{archived}",
                plan_title(plan.name.as_deref(), plan.created_at, offset),
                dinner_count(plan.num_days)
            )
        }
        None => "Everything you need to buy for the week's planned meals.".to_string(),
    }
}

#[component]
fn ShoppingListView(plan_id: ReadSignal<Option<Uuid>>) -> Element {
    let mut data = use_get_shopping_list(plan_id);
    let offset = use_local_offset();
    let toast = use_toast();
    let mut generating = use_signal(|| false);
    let mut selected = use_signal(Vec::<Uuid>::new);
    let mut basket_open = use_signal(|| false);
    let mut sleeping_open = use_signal(|| false);
    let mut covered_open = use_signal(|| false);
    // The covered line whose "Add anyway" is in flight.
    let mut adding_anyway = use_signal(|| None as Option<Uuid>);
    let mut sleeping = use_signal(|| false);
    // The line whose "Add back" is in flight.
    let mut waking = use_signal(|| None as Option<Uuid>);
    // Ticks made on this page, shown before the save lands so the box never
    // lags the click. A failed save drops its entry, which puts the box back.
    let mut ticks = use_signal(HashMap::<Uuid, bool>::new);
    let mut adding = use_signal(|| false);
    let mut add_open = use_signal(|| false);
    let mut confirm_rebuild = use_signal(|| false);
    // The own-item line whose "Remove" is in flight.
    let mut removing = use_signal(|| None as Option<Uuid>);
    let mut clearing = use_signal(|| false);
    let mut categories = use_shopping_categories();
    // The line whose category is being chosen.
    let mut categorising = use_signal(|| None as Option<ShoppingListLine>);
    let mut managing_categories = use_signal(|| false);

    let loaded = data();
    let view: Option<&ShoppingListForPlan> = match &loaded {
        Some(Ok(Some(view))) => Some(view),
        _ => None,
    };
    let shown_plan = view.map(|v| (v.plan.id, v.plan.household_id));
    let list_id = view.and_then(|v| v.list.as_ref()).map(|d| d.list.id);
    let has_list = view.is_some_and(|v| v.list.is_some());
    let is_stale = view
        .and_then(|v| v.list.as_ref())
        .is_some_and(|detail| detail.is_stale);
    let (covered, items): (Vec<ShoppingListLine>, Vec<ShoppingListLine>) =
        lines(&loaded).into_iter().partition(|line| line.is_covered);
    let subtitle = page_subtitle(view, offset());

    let purchased = move |line: &ShoppingListLine| {
        ticks
            .read()
            .get(&line.id)
            .copied()
            .unwrap_or(line.is_purchased)
    };

    let mut generate = move || {
        let Some((plan_id, _)) = shown_plan else {
            return;
        };
        if generating() {
            return;
        }
        generating.set(true);
        spawn(async move {
            match generate_shopping_list(plan_id).await {
                Ok(_) => {
                    data.restart();
                    let (title, message) = if has_list {
                        ("Rebuilt", "The list matches the plan again.")
                    } else {
                        ("Shopping list ready", "Every ingredient the week needs.")
                    };
                    toast.show(Some(title.to_string()), message.to_string(), Tone::Green);
                }
                Err(e) => toast.show(
                    Some("Couldn't make the shopping list".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            generating.set(false);
        });
    };

    let mut toggle_purchased = move |line_id: Uuid, now: bool| {
        ticks.write().insert(line_id, now);
        if now {
            selected.write().retain(|id| *id != line_id);
        }
        spawn(async move {
            if let Err(e) = set_item_purchased(line_id, now).await {
                ticks.write().remove(&line_id);
                toast.show(
                    Some("Couldn't save the tick".to_string()),
                    error_message(&e),
                    Tone::Red,
                );
            }
        });
    };

    let add_own_item = move |line: NewListLine| {
        let Some(list_id) = list_id else {
            return;
        };
        if adding() {
            return;
        }
        adding.set(true);
        spawn(async move {
            match add_list_line(list_id, line).await {
                Ok(_) => data.restart(),
                Err(e) => toast.show(
                    Some("Couldn't add it".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            adding.set(false);
        });
    };

    let mut remove_own_item = move |line: ShoppingListLine| {
        let Some(list_id) = list_id else {
            return;
        };
        if removing().is_some() {
            return;
        }
        removing.set(Some(line.id));
        spawn(async move {
            match remove_custom_item(list_id, line.id).await {
                Ok(_) => data.restart(),
                Err(e) => toast.show(
                    Some("Couldn't remove it".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            removing.set(None);
        });
    };

    let clear_bought = move |_: MouseEvent| {
        let Some(list_id) = list_id else {
            return;
        };
        if clearing() {
            return;
        }
        clearing.set(true);
        spawn(async move {
            match clear_bought_items(list_id).await {
                Ok(_) => data.restart(),
                Err(e) => toast.show(
                    Some("Couldn't clear them".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            clearing.set(false);
        });
    };

    let mut add_back = move |line: ShoppingListLine| {
        let Some((_, household_id)) = shown_plan else {
            return;
        };
        let Some(ingredient_id) = line.ingredient_id else {
            return;
        };
        if waking().is_some() {
            return;
        }
        waking.set(Some(line.id));
        spawn(async move {
            match wake_ingredient(household_id, ingredient_id).await {
                Ok(_) => {
                    data.restart();
                    toast.show(
                        Some("Added back".to_string()),
                        format!("{} is back on the list.", line.ingredient),
                        Tone::Green,
                    );
                }
                Err(e) => toast.show(
                    Some("Couldn't add it back".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            waking.set(None);
        });
    };

    let mut add_anyway = move |line: ShoppingListLine| {
        if adding_anyway().is_some() {
            return;
        }
        adding_anyway.set(Some(line.id));
        spawn(async move {
            match add_covered_item(line.id).await {
                Ok(_) => {
                    data.restart();
                    toast.show(
                        Some("Added to the list".to_string()),
                        format!("{} is on the list.", line.ingredient),
                        Tone::Green,
                    );
                }
                Err(e) => toast.show(
                    Some("Couldn't add it".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            adding_anyway.set(None);
        });
    };

    let sleep_selected = move |weeks: i32| {
        let Some((_, household_id)) = shown_plan else {
            return;
        };
        if sleeping() {
            return;
        }
        let picked = selected();
        let chosen: Vec<ShoppingListLine> = lines(&data.read())
            .into_iter()
            .filter(|line| {
                picked.contains(&line.id)
                    && line.ingredient_id.is_some()
                    && !line.is_covered
                    && line.sleep_until.is_none()
                    && !purchased(line)
            })
            .collect();
        if chosen.is_empty() {
            return;
        }
        sleeping.set(true);
        spawn(async move {
            let ids = chosen
                .iter()
                .filter_map(|line| line.ingredient_id)
                .collect();
            match sleep_ingredients(household_id, ids, weeks).await {
                Ok(slept) => {
                    selected.set(Vec::new());
                    data.restart();
                    let what = match chosen.as_slice() {
                        [only] => only.ingredient.clone(),
                        many => format!("{} ingredients", many.len()),
                    };
                    let until = slept.first().map_or_else(String::new, |s| {
                        format!(" until {}", date_label(s.sleep_until))
                    });
                    toast.show(
                        Some("Sleeping".to_string()),
                        format!("Not adding {what}{until}."),
                        Tone::Green,
                    );
                }
                Err(e) => toast.show(
                    Some("Couldn't sleep them".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
            sleeping.set(false);
        });
    };

    let pick_category = move |category_id: Option<Uuid>| {
        let Some(line) = categorising() else {
            return;
        };
        categorising.set(None);
        spawn(async move {
            match set_line_category(line.id, category_id).await {
                Ok(()) => data.restart(),
                Err(e) => toast.show(
                    Some("Couldn't set the category".to_string()),
                    error_message(&e),
                    Tone::Red,
                ),
            }
        });
    };

    let tick = move |line: &ShoppingListLine| {
        let id = line.id;
        EventHandler::new(move |now: bool| toggle_purchased(id, now))
    };

    let own_item_action = move |line: &ShoppingListLine| {
        let target = line.clone();
        let busy = removing().is_some();
        rsx! {
            ActionMenu { label: "More actions for {line.ingredient}",
                ActionMenuItem {
                    onclick: {
                        let target = line.clone();
                        move |_| categorising.set(Some(target.clone()))
                    },
                    "Category\u{2026}"
                }
                ActionMenuItem {
                    danger: true,
                    disabled: busy,
                    onclick: move |_| remove_own_item(target.clone()),
                    "Remove from list"
                }
            }
        }
    };

    let mut toggle_sleep_pick = move |id: Uuid| {
        let mut chosen = selected.write();
        match chosen.iter().position(|&x| x == id) {
            Some(at) => {
                chosen.remove(at);
            }
            None => chosen.push(id),
        }
    };

    let sleeping_count = items.iter().filter(|l| l.sleep_until.is_some()).count();
    let purchased_count = items
        .iter()
        .filter(|l| l.sleep_until.is_none() && purchased(l))
        .count();
    let to_buy = items.len() - sleeping_count - purchased_count;
    let all_bought = to_buy == 0 && purchased_count > 0;
    let category_list = match categories() {
        Some(Ok(list)) => list,
        _ => Vec::new(),
    };
    let to_buy_sections = sections(
        items
            .iter()
            .filter(|l| l.sleep_until.is_none() && !purchased(l))
            .cloned()
            .collect(),
        &category_list,
    );

    rsx! {
        div { class: "mr-page",
            div { class: "mr-pagehead",
                div { class: "mr-pagehead__titles",
                    Heading { level: 1, "Shopping List" }
                    Text { muted: true, "{subtitle}" }
                }
                div { class: "mr-pagehead__actions",
                    if let Some((plan_id, _)) = shown_plan {
                        Link {
                            to: Route::PlanDetail { id: plan_id },
                            class: "mr-btn mr-btn--outline mr-btn--md",
                            "Open plan"
                        }
                    }
                    if has_list {
                        button {
                            r#type: "button",
                            class: "mr-btn mr-btn--outline mr-btn--md",
                            aria_expanded: add_open(),
                            onclick: move |_| add_open.set(!add_open()),
                            "Add item"
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| managing_categories.set(true),
                            "Categories"
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            loading: generating(),
                            onclick: move |_| confirm_rebuild.set(true),
                            "Rebuild from plan"
                        }
                    }
                }
            }

            match &loaded {
                None => rsx! {
                    div { class: "mr-tablepanel mr-tablestate", Spinner {} }
                },
                Some(Err(e)) => rsx! {
                    Alert { tone: Tone::Red, title: "Couldn't load the shopping list", "{error_message(e)}" }
                },
                Some(Ok(None)) => rsx! {
                    div { class: "mr-empty",
                        div { class: "mr-empty__icon", aria_hidden: "true",
                            Icon { name: IconName::Dice }
                        }
                        Heading { level: 3, "No week to shop for" }
                        Text { muted: true, "Roll a week first, then make its shopping list here." }
                        Link { to: Route::Roll {}, class: "mr-btn mr-btn--solid mr-btn--lg", "Roll a week" }
                    }
                },
                Some(Ok(Some(_))) if !has_list => rsx! {
                    div { class: "mr-empty",
                        div { class: "mr-empty__icon",
                            Icon { name: IconName::Checklist }
                        }
                        Heading { level: 3, "No shopping list yet" }
                        Text { muted: true,
                            "Make one from this week's dinners: each ingredient is summed across the recipes that use it."
                        }
                        Button {
                            variant: ButtonVariant::Solid,
                            size: Size::Lg,
                            loading: generating(),
                            onclick: move |_| generate(),
                            "Make shopping list"
                        }
                    }
                },
                Some(Ok(Some(_))) => rsx! {
                    if is_stale {
                        div { class: "mr-stale",
                            Alert {
                                tone: Tone::Amber,
                                title: "The plan changed after this list was made",
                                div { class: "mr-stale__body",
                                    span { "Rebuild to match the current dinners. Ticked items stay ticked." }
                                    Button {
                                        variant: ButtonVariant::Outline,
                                        size: Size::Sm,
                                        loading: generating(),
                                        onclick: move |_| confirm_rebuild.set(true),
                                        "Rebuild from plan"
                                    }
                                }
                            }
                        }
                    }
                    p { class: "mr-listsummary", role: "status", aria_live: "polite",
                        "{to_buy} to buy, {purchased_count} purchased, {sleeping_count} sleeping"
                    }
                    if add_open() {
                        AddListItem { busy: adding(), on_add: add_own_item }
                    }
                    for (label, section) in to_buy_sections {
                        if let Some(label) = label {
                            h2 { class: "mr-linesection", "{label}" }
                        }
                        ul { class: "mr-linelist",
                        for line in section.iter() {
                            LineRow {
                                key: "{line.id}",
                                line: line.clone(),
                                checked: false,
                                selected: selected().contains(&line.id),
                                on_tick: tick(line),
                                if line.added_by_hand {
                                    {own_item_action(line)}
                                } else {
                                    ActionMenu { label: "More actions for {line.ingredient}",
                                        ActionMenuItem {
                                            onclick: {
                                                let target = line.clone();
                                                move |_| categorising.set(Some(target.clone()))
                                            },
                                            "Category\u{2026}"
                                        }
                                        ActionMenuItem {
                                            onclick: {
                                                let id = line.id;
                                                move |_| toggle_sleep_pick(id)
                                            },
                                            if selected().contains(&line.id) {
                                                "Don't sleep"
                                            } else {
                                                "Sleep for a while\u{2026}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        }
                    }
                    if items.is_empty() && covered.is_empty() {
                        Text { muted: true, "None of this week's recipes list any ingredients yet." }
                    } else if all_bought {
                        div { class: "mr-empty mr-empty--inline", role: "status", aria_live: "polite",
                            div { class: "mr-empty__icon", aria_hidden: "true", Icon { name: IconName::Check } }
                            Heading { level: 3, "Everything's in the basket" }
                            Text { muted: true,
                                "All {purchased_count} items are bought. Clear them to start the next shop."
                            }
                        }
                    } else if to_buy == 0 {
                        Text { muted: true, "Nothing left to buy." }
                    }
                    if purchased_count > 0 {
                        section { class: "mr-linegroup",
                            div { class: "mr-linegroup__head",
                                button {
                                    class: "mr-linegroup__toggle",
                                    r#type: "button",
                                    aria_expanded: "{basket_open()}",
                                    onclick: move |_| basket_open.set(!basket_open()),
                                    span { class: "mr-linegroup__chev", aria_hidden: "true",
                                        if basket_open() { "\u{25be}" } else { "\u{25b8}" }
                                    }
                                    "In basket ({purchased_count})"
                                }
                                Button {
                                    variant: ButtonVariant::Outline,
                                    size: Size::Sm,
                                    loading: clearing(),
                                    onclick: clear_bought,
                                    "Clear bought"
                                }
                            }
                            if basket_open() {
                                ul { class: "mr-linelist",
                                    for line in items.iter().filter(|l| l.sleep_until.is_none() && purchased(l)) {
                                        LineRow {
                                            key: "{line.id}",
                                            line: line.clone(),
                                            checked: true,
                                            on_tick: tick(line),
                                            if line.added_by_hand {
                                                {own_item_action(line)}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !covered.is_empty() {
                        section { class: "mr-linegroup",
                            div { class: "mr-linegroup__head",
                                button {
                                    class: "mr-linegroup__toggle",
                                    r#type: "button",
                                    aria_expanded: "{covered_open()}",
                                    onclick: move |_| covered_open.set(!covered_open()),
                                    span { class: "mr-linegroup__chev", aria_hidden: "true",
                                        if covered_open() { "\u{25be}" } else { "\u{25b8}" }
                                    }
                                    "Covered by pantry ({covered.len()})"
                                }
                            }
                            if covered_open() {
                                ul { class: "mr-linelist",
                                    for line in covered.iter() {
                                        LineRow {
                                            key: "{line.id}",
                                            line: line.clone(),
                                            checked: false,
                                            on_tick: None,
                                            Button {
                                                variant: ButtonVariant::Ghost,
                                                size: Size::Sm,
                                                loading: adding_anyway() == Some(line.id),
                                                disabled: adding_anyway().is_some(),
                                                onclick: {
                                                    let target = line.clone();
                                                    move |_| add_anyway(target.clone())
                                                },
                                                "Add anyway"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if sleeping_count > 0 {
                        section { class: "mr-linegroup",
                            div { class: "mr-linegroup__head",
                                button {
                                    class: "mr-linegroup__toggle",
                                    r#type: "button",
                                    aria_expanded: "{sleeping_open()}",
                                    onclick: move |_| sleeping_open.set(!sleeping_open()),
                                    span { class: "mr-linegroup__chev", aria_hidden: "true",
                                        if sleeping_open() { "\u{25be}" } else { "\u{25b8}" }
                                    }
                                    "Sleeping ({sleeping_count})"
                                }
                            }
                            if sleeping_open() {
                                ul { class: "mr-linelist",
                                    for line in items.iter().filter(|l| l.sleep_until.is_some()) {
                                        LineRow {
                                            key: "{line.id}",
                                            line: line.clone(),
                                            checked: false,
                                            on_tick: None,
                                            if let Some(until) = line.sleep_until {
                                                Badge { tone: Tone::Amber, "Back {date_label(until)}" }
                                                Button {
                                                    variant: ButtonVariant::Ghost,
                                                    size: Size::Sm,
                                                    loading: waking() == Some(line.id),
                                                    disabled: waking().is_some(),
                                                    onclick: {
                                                        let target = line.clone();
                                                        move |_| add_back(target.clone())
                                                    },
                                                    "Add back"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
            }

            if confirm_rebuild() {
                ConfirmDialog {
                    title: "Rebuild from plan?",
                    confirm_label: "Rebuild",
                    on_confirm: move |_| {
                        confirm_rebuild.set(false);
                        generate();
                    },
                    on_cancel: move |_| confirm_rebuild.set(false),
                    p { class: "mr-confirm__line",
                        strong { "Keeps: " }
                        "items you added yourself, ticks on ingredients still in the plan, and sleeping ingredients."
                    }
                    p { class: "mr-confirm__line",
                        strong { "Changes: " }
                        "amounts are recalculated from this week's dinners, and ingredients no longer in the plan are removed."
                    }
                }
            }

            if let Some(line) = categorising() {
                CategoryPicker {
                    name: line.ingredient.clone(),
                    current: line.category_id,
                    categories: category_list.clone(),
                    on_pick: pick_category,
                    on_manage: move |_| {
                        categorising.set(None);
                        managing_categories.set(true);
                    },
                    on_close: move |_| categorising.set(None),
                }
            }

            if managing_categories() {
                ManageCategoriesDialog {
                    categories: categories(),
                    on_changed: move |_| categories.restart(),
                    on_close: move |_| {
                        managing_categories.set(false);
                        data.restart();
                    },
                }
            }

            if !selected().is_empty() {
                SleepBar {
                    count: selected().len(),
                    busy: sleeping(),
                    on_sleep: sleep_selected,
                    on_cancel: move |_| selected.set(Vec::new()),
                }
            }
        }
    }
}
