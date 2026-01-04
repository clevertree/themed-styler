use super::{State, ThemeEntry};
use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct ThemesInput {
  #[serde(default)]
  pub themes: IndexMap<String, ThemeEntry>,
  #[serde(default)]
  pub current_theme: Option<String>,
}

pub fn build_state(themes_input: ThemesInput) -> State {
  let mut state = State::new_default();
  if !themes_input.themes.is_empty() {
    state.themes = themes_input.themes;
    if let Some(current) = themes_input.current_theme.clone() {
      if state.themes.contains_key(&current) {
        state.current_theme = current;
      }
    }
    if state.default_theme.is_empty() {
      if let Some((name, _)) = state.themes.iter().next() {
        state.default_theme = name.clone();
      }
    }
  }
  state
}

pub fn parse_themes_json(json: &str) -> ThemesInput {
  match serde_json::from_str(json) {
    Ok(input) => input,
    Err(e) => {
      eprintln!("[parse_themes_json] Deserialization error: {}", e);
      ThemesInput::default()
    }
  }
}
