//! Primitive UI components — the small, reusable building blocks the rest of
//! the app composes. Styling lives in `assets/primitives.css` (design tokens);
//! these files only apply the `mr-*` classes and expose a typed Rust API.

pub mod alert;
pub mod badge;
pub mod brand_mark;
pub mod button;
pub mod card;
pub mod checkbox;
pub mod chip;
pub mod combobox;
pub mod confirm_dialog;
pub mod data_table;
pub mod field;
pub mod icon;
pub mod icon_button;
pub mod input;
pub mod modal;
pub mod popover;
pub mod rating;
pub mod search_bar;
pub mod select;
pub mod separator;
pub mod skeleton;
pub mod spinner;
pub mod stack;
pub mod switch;
pub mod table;
pub mod tabs;
pub mod textarea;
pub mod toast;
pub mod typography;

pub use alert::Alert;
pub use badge::{Badge, Tone};
pub use brand_mark::BrandMark;
pub use button::{Button, ButtonVariant, Size};
pub use card::Card;
pub use checkbox::Checkbox;
pub use chip::Chip;
pub use combobox::{ComboOption, Combobox, Picker};
pub use confirm_dialog::ConfirmDialog;
pub use data_table::{CellAlign, Column, DataTable};
pub use field::Field;
pub use icon::{Icon, IconName};
pub use icon_button::IconButton;
pub use input::Input;
pub use modal::Modal;
pub use popover::Popover;
pub use rating::{Rating, RatingScale};
pub use search_bar::{FilterField, SearchBar};
pub use select::{Select, SelectOption};
#[expect(unused_imports)]
pub use separator::Orientation;
pub use separator::Separator;
pub use skeleton::Skeleton;
pub use spinner::Spinner;
pub use stack::{Align, HStack, VStack};
#[expect(unused_imports)]
pub use stack::{Dir, Stack};
pub use switch::Switch;
pub use table::Table;
pub use tabs::Tabs;
pub use textarea::Textarea;
pub use toast::Toast;
pub use typography::{Heading, Text};
