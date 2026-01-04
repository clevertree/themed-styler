use crate::{State};
use indexmap::{IndexMap, IndexSet};

/// Return an empty default State. Themes are loaded from the client.
pub fn bundled_state() -> State {
    State {
        themes: IndexMap::new(),
        default_theme: String::new(),
        current_theme: String::new(),
        display_density: 1.0,
        scaled_density: 1.0,
        used_classes: IndexSet::new(),
        used_tags: IndexSet::new(),
        used_tag_classes: IndexSet::new(),
    }
}

