#[cfg(test)]
mod tests {
    use crate::theme::{State, ThemeEntry};
    use indexmap::IndexMap;
    use serde_json::json;

    #[test]
    fn default_theme_has_p2() {
        let mut st = State::new_default();
        st.register_tailwind_classes(["p-2".to_string()]);
        let css = st.css_for_web();
        assert!(css.contains(".p-2{"));
        assert!(css.contains("padding:8px"));
    }

    #[test]
    fn android_conversion() {
        let mut st = State::new_default();
        let mut styles = IndexMap::new();
        let mut button_props = IndexMap::new();
        button_props.insert("backgroundColor".to_string(), json!("#007bff"));
        styles.insert("button".to_string(), button_props);
        st.add_theme("default", styles);
        st.set_theme("default").ok();
        
        let out = st.android_styles_for("button", &[]);
        assert!(out.get("backgroundColor").is_some());
    }

    #[test]
    fn android_flex_row_default() {
        let st = State::new_default();
        let styles = st.android_styles_for("div", &["flex".to_string()]);
        assert_eq!(styles.get("androidOrientation").and_then(|v| v.as_str()), Some("horizontal"));
        assert_eq!(styles.get("flexDirection").and_then(|v| v.as_str()), Some("row"));
        
        let styles = st.android_styles_for("div", &[]);
        assert_eq!(styles.get("androidOrientation").and_then(|v| v.as_str()), Some("vertical"));
        assert_eq!(styles.get("flexDirection").and_then(|v| v.as_str()), Some("column"));
    }

    #[test]
    fn test_button_bg_override() {
        let mut themes = IndexMap::new();
        
        let mut variables = IndexMap::new();
        variables.insert("color.bg".to_string(), "#ffffff".to_string());
        
        let mut selectors = IndexMap::new();
        let mut button_props = IndexMap::new();
        button_props.insert("background-color".to_string(), json!("#2563eb"));
        selectors.insert("button".to_string(), button_props);
        
        let default_theme = ThemeEntry {
            name: Some("default".to_string()),
            inherits: None,
            selectors,
            variables,
            breakpoints: IndexMap::new(),
        };
        
        themes.insert("default".to_string(), default_theme);
        
        let mut state = State::new_default();
        state.themes = themes;
        state.current_theme = "default".to_string();
        
        let classes = vec!["bg-bg".to_string(), "p-4".to_string()];
        let styles = state.android_styles_for("button", &classes);
        
        assert_eq!(styles.get("backgroundColor").and_then(|v: &serde_json::Value| v.as_str()), Some("#ffffff"));
        assert_eq!(styles.get("paddingTop"), Some(&serde_json::json!(16)));
        assert_eq!(styles.get("paddingVertical"), Some(&serde_json::json!(16)));
    }

    #[test]
    fn test_class_selector_matching() {
        let mut themes = IndexMap::new();
        let mut selectors = IndexMap::new();
        
        let mut bg_primary = IndexMap::new();
        bg_primary.insert("background-color".to_string(), json!("#3b82f6"));
        selectors.insert(".bg-primary".to_string(), bg_primary);
        
        let default_theme = ThemeEntry {
            name: Some("default".to_string()),
            inherits: None,
            selectors,
            variables: IndexMap::new(),
            breakpoints: IndexMap::new(),
        };
        
        themes.insert("default".to_string(), default_theme);
        
        let mut state = State::new_default();
        state.themes = themes;
        state.current_theme = "default".to_string();
        
        let classes = vec!["bg-primary".to_string()];
        let styles = state.android_styles_for("div", &classes);
        
        assert_eq!(styles.get("backgroundColor").and_then(|v| v.as_str()), Some("#3b82f6"));
    }
}
