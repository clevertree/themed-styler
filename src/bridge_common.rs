use super::{State, ThemeEntry};
use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct UsageSnapshot {
  #[serde(default)]
  pub selectors: Vec<String>,
  #[serde(default)]
  pub classes: Vec<String>,
}

#[derive(Deserialize, Default)]
pub struct ThemesInput {
  #[serde(default)]
  pub themes: IndexMap<String, ThemeEntry>,
  #[serde(default)]
  pub current_theme: Option<String>,
}

pub fn build_state(usage: UsageSnapshot, themes_input: ThemesInput) -> State {
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
    eprintln!("[build_state] themes: {:?}", state.themes.keys().collect::<Vec<_>>());
    eprintln!("[build_state] current_theme: {:?}", state.current_theme);
    eprintln!("[build_state] default_theme: {:?}", state.default_theme);
  }
  state.register_selectors(usage.selectors);
  for class in usage.classes {
    state.used_classes.insert(class);
  }
  state
}

pub fn parse_usage_json(json: &str) -> UsageSnapshot {
  serde_json::from_str(json).unwrap_or_default()
}

pub fn parse_themes_json(json: &str) -> ThemesInput {
  match serde_json::from_str(json) {
    Ok(input) => input,
    Err(e) => {
      eprintln!("[parse_themes_json] Deserialization error: {}", e);
      eprintln!("[parse_themes_json] JSON sample (first 300 chars): {}", &json.chars().take(300).collect::<String>());
      ThemesInput::default()
    }
  }
}
