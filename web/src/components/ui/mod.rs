//! Primitive UI components — the small, reusable building blocks the rest of
//! the app composes. Styling lives in `assets/primitives.css` (design tokens);
//! these files only apply the `mr-*` classes and expose a typed Rust API.

pub mod alert;
pub mod badge;
pub mod button;
pub mod card;
pub mod checkbox;
pub mod input;
pub mod modal;
pub mod select;
pub mod separator;
pub mod skeleton;
pub mod spinner;
pub mod stack;
pub mod switch;
pub mod table;
pub mod tabs;
pub mod typography;

pub use alert::Alert;
pub use badge::{Badge, Tone};
pub use button::{Button, ButtonVariant, Size};
pub use card::Card;
pub use checkbox::Checkbox;
pub use input::Input;
pub use modal::Modal;
pub use select::{Select, SelectOption};
pub use separator::{Orientation, Separator};
pub use skeleton::Skeleton;
pub use spinner::Spinner;
pub use stack::{Align, Dir, HStack, Stack, VStack};
pub use switch::Switch;
pub use table::Table;
pub use tabs::Tabs;
pub use typography::{Heading, Text};
