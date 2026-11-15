use crate::components::ui::ComboOption;

/// What the add-recipe dialog starts with, for a recipe wanted to satisfy a
/// roll rule. The ingredient's hint is its unit.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct RecipeSeed {
    pub cuisine: Option<ComboOption>,
    pub protein: Option<ComboOption>,
    pub ingredient: Option<ComboOption>,
    pub difficulty: Option<i16>,
}
