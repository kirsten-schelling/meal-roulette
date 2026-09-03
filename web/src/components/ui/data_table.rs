use dioxus::prelude::*;
use std::collections::HashSet;
use std::rc::Rc;

/// Horizontal alignment for a column's cells (header + body).
#[derive(Clone, Copy, PartialEq, Default)]
pub enum CellAlign {
    #[default]
    Start,
    #[expect(dead_code)]
    Center,
    End,
}

impl CellAlign {
    fn cls(self) -> &'static str {
        match self {
            CellAlign::Start => "",
            CellAlign::Center => "mr-td--center",
            CellAlign::End => "mr-td--end",
        }
    }
}

/// One column of a [`DataTable`]. It carries its header text and — the important
/// part — *how to draw a cell* for a given row (`cell`). That render closure is
/// what keeps the table generic: the table never has to know what a `T` is, only
/// how each column turns one into UI.
///
/// It's the same shape as a React column def: `{ header, cell: (row) => <…> }`.
pub struct Column<T> {
    header: String,
    align: CellAlign,
    cell: Rc<dyn Fn(&T) -> Element>,
}

impl<T> Column<T> {
    /// Build a column. `cell` receives a `&T` (one row) and returns that cell's UI.
    ///
    /// ```ignore
    /// Column::new("Title", |r: &Recipe| rsx! { "{r.title}" })
    /// ```
    pub fn new(header: impl Into<String>, cell: impl Fn(&T) -> Element + 'static) -> Self {
        Self {
            header: header.into(),
            align: CellAlign::Start,
            cell: Rc::new(cell),
        }
    }

    /// Change the column's alignment (e.g. right-align a numeric column).
    pub fn align(mut self, align: CellAlign) -> Self {
        self.align = align;
        self
    }
}

// Written by hand rather than derived: the render closure lives behind an `Rc`,
// which clones no matter what `T` is, so we don't want a `T: Clone` bound here.
impl<T> Clone for Column<T> {
    fn clone(&self) -> Self {
        Self {
            header: self.header.clone(),
            align: self.align,
            cell: self.cell.clone(),
        }
    }
}

// Component props must be `PartialEq` so Dioxus can skip needless re-renders.
// Two columns count as equal when their header, alignment, and (same) closure
// match — `Rc::ptr_eq` compares the closure by identity.
impl<T> PartialEq for Column<T> {
    fn eq(&self, other: &Self) -> bool {
        self.header == other.header
            && self.align == other.align
            && Rc::ptr_eq(&self.cell, &other.cell)
    }
}

/// A generic data table. Give it a set of [`Column`]s and a `Vec` of rows of any
/// type `T`; each column's closure renders its own cell. The table itself knows
/// nothing about the row type — swap in `Ingredient`, `PlannedMeal`, … unchanged.
///
/// `selectable` adds a leading checkbox column (with select-all in the header).
/// Selection is by row index and lives inside the table; `on_select` reports
/// the sorted index set after every change.
#[component]
pub fn DataTable<T: Clone + PartialEq + 'static>(
    columns: Vec<Column<T>>,
    rows: Vec<T>,
    /// Shown in place of the body when there are no rows.
    #[props(default = "Nothing to show yet.".to_string())]
    empty: String,
    #[props(default)] selectable: bool,
    on_select: Option<EventHandler<Vec<usize>>>,
) -> Element {
    let mut selected = use_signal(HashSet::<usize>::new);

    let row_count = rows.len();
    let col_count = columns.len() + usize::from(selectable);
    let all_selected = selectable && row_count > 0 && selected.read().len() == row_count;

    let notify = move || {
        if let Some(cb) = &on_select {
            let mut v: Vec<usize> = selected.read().iter().copied().collect();
            v.sort_unstable();
            cb.call(v);
        }
    };

    rsx! {
        div { class: "mr-table-wrap",
            table { class: "mr-table",
                thead {
                    tr {
                        if selectable {
                            th { class: "mr-td--check",
                                input {
                                    class: "mr-table__check",
                                    r#type: "checkbox",
                                    checked: all_selected,
                                    aria_label: "Select all rows",
                                    onchange: move |_| {
                                        if selected.read().len() == row_count {
                                            selected.write().clear();
                                        } else {
                                            *selected.write() = (0..row_count).collect();
                                        }
                                        notify();
                                    },
                                }
                            }
                        }
                        for col in columns.iter() {
                            th { class: col.align.cls(), "{col.header}" }
                        }
                    }
                }
                tbody {
                    if rows.is_empty() {
                        tr {
                            td {
                                class: "mr-table__empty",
                                colspan: "{col_count}",
                                "{empty}"
                            }
                        }
                    } else {
                        for (i, row) in rows.iter().enumerate() {
                            tr { key: "{i}",
                                if selectable {
                                    td { class: "mr-td--check",
                                        input {
                                            class: "mr-table__check",
                                            r#type: "checkbox",
                                            checked: selected.read().contains(&i),
                                            aria_label: "Select row",
                                            onchange: move |_| {
                                                {
                                                    let mut s = selected.write();
                                                    if !s.remove(&i) {
                                                        s.insert(i);
                                                    }
                                                }
                                                notify();
                                            },
                                        }
                                    }
                                }
                                for col in columns.iter() {
                                    td { class: col.align.cls(), {(col.cell)(row)} }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
