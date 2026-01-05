#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use crate::theme::{State, ThemeEntry};
use crate::theme::bundled_state;
use indexmap::IndexMap;
use serde_json::json;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn render_css_for_web(state_json: &str) -> String {
    match serde_json::from_str::<State>(state_json) {
        Ok(s) => s.css_for_web(),
        Err(_) => "".into(),
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_android_styles(state_json: &str, selector: &str, classes_json: &str) -> String {
    let classes: Vec<String> = serde_json::from_str(classes_json).unwrap_or_default();
    match serde_json::from_str::<State>(state_json) {
        Ok(s) => serde_json::to_string(&s.android_styles_for(selector, &classes)).unwrap_or_else(|_| "{}".into()),
        Err(_) => "{}".into(),
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_default_state_json() -> String {
    let st = bundled_state();
    match serde_json::to_string(&st.to_json()) {
        Ok(s) => s,
        Err(_) => "{}".to_string(),
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn register_theme_json(state_json: &str, theme_json: &str) -> String {
    match (serde_json::from_str::<State>(state_json), serde_json::from_str::<serde_json::Value>(theme_json)) {
        (Ok(mut state), Ok(theme_obj)) => {
            if let (Some(name), Some(theme_entry)) = (theme_obj.get("name"), theme_obj.get("theme")) {
                if let Ok(entry) = serde_json::from_value::<ThemeEntry>(theme_entry.clone()) {
                    let theme_name = name.as_str().unwrap_or("").to_string();
                    if !theme_name.is_empty() {
                        state.themes.insert(theme_name, entry);
                    }
                }
            }
            match serde_json::to_string(&state.to_json()) {
                Ok(s) => s,
                Err(_) => "{}".to_string(),
            }
        }
        _ => "{}".to_string(),
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_theme_json(state_json: &str, theme_name: &str) -> String {
    match serde_json::from_str::<State>(state_json) {
        Ok(mut state) => {
            if state.themes.contains_key(theme_name) {
                state.default_theme = theme_name.to_string();
                state.current_theme = theme_name.to_string();
            }
            match serde_json::to_string(&state.to_json()) {
                Ok(s) => s,
                Err(_) => "{}".to_string(),
            }
        }
        _ => "{}".to_string(),
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_theme_list_json(state_json: &str) -> String {
    match serde_json::from_str::<State>(state_json) {
        Ok(state) => {
            let themes: Vec<serde_json::Value> = state.themes.iter().map(|(key, entry)| {
                json!({
                    "key": key,
                    "name": entry.name.as_ref().unwrap_or(key)
                })
            }).collect();
            serde_json::to_string(&themes).unwrap_or_else(|_| "[]".to_string())
        }
        _ => "[]".to_string(),
    }
}
