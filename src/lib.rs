use indexmap::{IndexMap, IndexSet};
use once_cell::sync::Lazy;
use serde::{de::Deserializer, Deserialize, Serialize};
use serde_json::json;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
mod default_state;
mod color;
use default_state::bundled_state;

// Default display density (1.0 = mdpi baseline)
fn default_display_density() -> f32 { 1.0 }
fn default_scaled_density() -> f32 { 1.0 }

pub type CssProps = IndexMap<String, serde_json::Value>;
pub type SelectorStyles = IndexMap<String, CssProps>; // selector -> props

/// Convert dp to pixels using display density
fn dp_to_px(dp: f32, density: f32) -> i32 {
    (dp * density).round() as i32
}

/// Convert sp to pixels using scaled density  
fn sp_to_px(sp: f32, scaled_density: f32) -> f32 {
    sp * scaled_density
}

/// Parse a CSS value and convert to Android pixels if needed
fn parse_and_convert_to_px(value: &serde_json::Value, density: f32) -> Option<serde_json::Value> {
    match value {
        serde_json::Value::Number(n) => {
            // Bare number treated as dp
            let dp = n.as_f64()? as f32;
            Some(serde_json::json!(dp_to_px(dp, density)))
        }
        serde_json::Value::String(s) => {
            // Parse string with units
            let trimmed = s.trim();
            if trimmed.ends_with("px") {
                // Treat px as density-independent pixels (dp) for cross-platform parity
                let px = trimmed.trim_end_matches("px").trim().parse::<f32>().ok()?;
                Some(serde_json::json!(dp_to_px(px, density)))
            } else if trimmed.ends_with("dp") {
                let dp = trimmed.trim_end_matches("dp").trim().parse::<f32>().ok()?;
                Some(serde_json::json!(dp_to_px(dp, density)))
            } else if let Ok(num) = trimmed.parse::<f32>() {
                // Bare number as string, treat as dp
                Some(serde_json::json!(dp_to_px(num, density)))
            } else {
                // Keep as-is (e.g., "wrap_content", "match_parent")
                None
            }
        }
        _ => None
    }
}

fn deserialize_variables<'de, D>(deserializer: D) -> Result<IndexMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    let mut out: IndexMap<String, String> = IndexMap::new();
    if let Some(v) = value {
        flatten_variables(None, &v, &mut out);
    }
    Ok(out)
}

fn flatten_variables(prefix: Option<&str>, value: &serde_json::Value, out: &mut IndexMap<String, String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                let key = if let Some(p) = prefix {
                    format!("{}.{}", p, k)
                } else {
                    k.to_string()
                };
                flatten_variables(Some(&key), v, out);
            }
        }
        serde_json::Value::Array(arr) => {
            for (idx, v) in arr.iter().enumerate() {
                let key = if let Some(p) = prefix {
                    format!("{}.{}", p, idx)
                } else {
                    idx.to_string()
                };
                flatten_variables(Some(&key), v, out);
            }
        }
        serde_json::Value::Null => {}
        serde_json::Value::Bool(b) => {
            if let Some(p) = prefix {
                out.insert(p.to_string(), b.to_string());
            }
        }
        serde_json::Value::Number(n) => {
            if let Some(p) = prefix {
                out.insert(p.to_string(), n.to_string());
            }
        }
        serde_json::Value::String(s) => {
            if let Some(p) = prefix {
                out.insert(p.to_string(), s.clone());
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeEntry {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub inherits: Option<String>,
    #[serde(default)]
    pub selectors: SelectorStyles,
    #[serde(default, deserialize_with = "deserialize_variables")]
    pub variables: IndexMap<String, String>,
    #[serde(default, deserialize_with = "deserialize_variables")]
    pub breakpoints: IndexMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct State {
    // New format: each theme has selectors, variables, breakpoints, and optional inherits
    pub themes: IndexMap<String, ThemeEntry>,
    pub default_theme: String,
    pub current_theme: String,
    // Platform-specific metadata for unit conversions
    #[serde(default = "default_display_density")]
    pub display_density: f32, // Android displayMetrics.density (1.0 for mdpi, 2.0 for xhdpi, etc.)
    #[serde(default = "default_scaled_density")]
    pub scaled_density: f32,  // Android displayMetrics.scaledDensity for SP conversions
    
    #[serde(default)]
    pub used_classes: IndexSet<String>,   // observed classes on elements
    #[serde(default)]
    pub used_tags: IndexSet<String>,      // observed tags on elements
    /// Observed (tag, class) pairs. Encoded as "tag|class" for JSON simplicity.
    #[serde(default)]
    pub used_tag_classes: IndexSet<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("theme not found: {0}")]
    ThemeNotFound(String),
}

impl State {
    pub fn new_default() -> Self {
        // Prefer embedded Rust bundled defaults
        return bundled_state();
    }

    /// Public helper to access the embedded default state.
    pub fn default_state() -> Self {
        bundled_state()
    }

    pub fn set_theme(&mut self, theme: impl Into<String>) -> Result<(), Error> {
        let name = theme.into();
        if !self.themes.contains_key(&name) {
            return Err(Error::ThemeNotFound(name));
        }
        self.current_theme = name;
        Ok(())
    }

    pub fn add_theme(&mut self, name: impl Into<String>, styles: SelectorStyles) {
        let name = name.into();
        let entry = self.themes.entry(name).or_default();
        for (sel, props) in styles.into_iter() {
            let e = entry.selectors.entry(sel).or_default();
            merge_props(e, &props);
        }
    }

    pub fn set_variables(&mut self, vars: IndexMap<String, String>) {
        // Back-compat: set on current theme entry
        let cur = self.current_theme.clone();
        let entry = self.themes.entry(cur).or_default();
        entry.variables = vars;
    }

    pub fn set_breakpoints(&mut self, map: IndexMap<String, String>) {
        let cur = self.current_theme.clone();
        let entry = self.themes.entry(cur).or_default();
        entry.breakpoints = map;
    }

    pub fn process_styles(&self, mut styles: IndexMap<String, serde_json::Value>) -> IndexMap<String, serde_json::Value> {
        let density = self.display_density;
        
        // Expand shorthands
        // Order matters: Horizontal/Vertical should be expanded before general shorthands
        // so that specific ones win if they were already present.
        if let Some(ph) = styles.get("paddingHorizontal").cloned() {
            styles.entry("paddingLeft".into()).or_insert(ph.clone());
            styles.entry("paddingRight".into()).or_insert(ph.clone());
        }
        if let Some(pv) = styles.get("paddingVertical").cloned() {
            styles.entry("paddingTop".into()).or_insert(pv.clone());
            styles.entry("paddingBottom".into()).or_insert(pv.clone());
        }
        if let Some(p) = styles.get("padding").cloned() {
            styles.entry("paddingTop".into()).or_insert(p.clone());
            styles.entry("paddingBottom".into()).or_insert(p.clone());
            styles.entry("paddingLeft".into()).or_insert(p.clone());
            styles.entry("paddingRight".into()).or_insert(p.clone());
        }
        if let Some(mh) = styles.get("marginHorizontal").cloned() {
            styles.entry("marginLeft".into()).or_insert(mh.clone());
            styles.entry("marginRight".into()).or_insert(mh.clone());
        }
        if let Some(mv) = styles.get("marginVertical").cloned() {
            styles.entry("marginTop".into()).or_insert(mv.clone());
            styles.entry("marginBottom".into()).or_insert(mv.clone());
        }
        if let Some(m) = styles.get("margin").cloned() {
            styles.entry("marginTop".into()).or_insert(m.clone());
            styles.entry("marginBottom".into()).or_insert(m.clone());
            styles.entry("marginLeft".into()).or_insert(m.clone());
            styles.entry("marginRight".into()).or_insert(m.clone());
        }
        if let Some(r) = styles.get("borderRadius").cloned() {
            styles.entry("borderTopLeftRadius".into()).or_insert(r.clone());
            styles.entry("borderTopRightRadius".into()).or_insert(r.clone());
            styles.entry("borderBottomLeftRadius".into()).or_insert(r.clone());
            styles.entry("borderBottomRightRadius".into()).or_insert(r.clone());
        }

        // Convert only dimension properties to pixels
        let dimension_props = [
            "width", "height", "minWidth", "minHeight", "maxWidth", "maxHeight",
            "padding", "paddingTop", "paddingBottom", "paddingLeft", "paddingRight",
            "paddingHorizontal", "paddingVertical",
            "margin", "marginTop", "marginBottom", "marginLeft", "marginRight",
            "marginHorizontal", "marginVertical",
            "borderRadius", "borderTopLeftRadius", "borderTopRightRadius", "borderBottomLeftRadius", "borderBottomRightRadius",
            "borderWidth", "borderTopWidth", "borderBottomWidth", "borderLeftWidth", "borderRightWidth",
            "gap", "rowGap", "columnGap", "elevation", "fontSize", "lineHeight", "letterSpacing"
        ];
        
        for prop in &dimension_props {
            if let Some(value) = styles.get(*prop).cloned() {
                if let Some(converted) = parse_and_convert_to_px(&value, density) {
                    styles.insert(prop.to_string(), converted);
                }
            }
        }
        
        styles
    }

    pub fn set_default_theme(&mut self, name: impl Into<String>) {
        self.default_theme = name.into();
    }

    pub fn register_selectors<I: IntoIterator<Item = String>>(&mut self, _selectors: I) {
        // Deprecated: selectors are now part of the theme entry
    }

    pub fn register_tailwind_classes<I: IntoIterator<Item = String>>(&mut self, classes: I) {
        for c in classes {
            self.used_classes.insert(c);
        }
    }

    pub fn register_tags<I: IntoIterator<Item = String>>(&mut self, tags: I) {
        for t in tags {
            self.used_tags.insert(t);
        }
    }

    pub fn register_tag_class(&mut self, tag: impl Into<String>, class_: impl Into<String>) {
        let key = format!("{}|{}", tag.into(), class_.into());
        self.used_tag_classes.insert(key);
    }


    pub fn clear_usage(&mut self) {
        self.used_classes.clear();
        self.used_tags.clear();
        self.used_tag_classes.clear();
    }

    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "themes": self.themes,
            "default_theme": self.default_theme,
            "current_theme": self.current_theme,
            "display_density": self.display_density,
            "scaled_density": self.scaled_density,
            "used_classes": self.used_classes,
            "used_tags": self.used_tags,
            "used_tag_classes": self.used_tag_classes,
        })
    }

    pub fn from_json(value: serde_json::Value) -> anyhow::Result<Self> {
        let state: State = serde_json::from_value(value)?;
        Ok(state)
    }

    pub fn css_for_web(&self) -> String {
        // Compute CSS resolved from the effective theme (with inheritance)
        let (eff, vars) = self.effective_theme_all();
        let bps = self.effective_breakpoints();
        let mut rules: Vec<(String, CssProps)> = Vec::new();
        
        // Build closure: if a (tag,class) pair is observed, consider both the tag and the class as used too
        let mut used_tags: IndexSet<String> = self.used_tags.clone();
        let mut used_classes: IndexSet<String> = self.used_classes.clone();
        for key in &self.used_tag_classes {
            if let Some((t, c)) = split_tag_class_key(key) {
                used_tags.insert(t);
                used_classes.insert(c);
            }
        }

        // Helper to decide if a themed selector should be emitted based on observed usage.
        // Supported selector forms:
        //  - tag           (e.g., "h1")
        //  - .class        (e.g., ".text-sm"), optional pseudo ":hover"
        //  - tag.class     (e.g., "h1.text-sm"), optional pseudo ":hover"
        for (sel, props) in eff.iter() {
            if should_emit_selector(sel, &used_tags, &used_classes, &self.used_tag_classes) {
                rules.push((sel.clone(), props.clone()));
            }
        }

        // Also emit dynamic utility properties for used classes
        for class in &used_classes {
            let (bp_key, hover, base) = parse_prefixed_class(class);
            let selector = if hover { format!(".{}:hover", css_escape_class(&base)) } else { format!(".{}", css_escape_class(&base)) };

            // 1) Exact selector in effective theme (e.g. ".x:hover")
            if let Some(props) = eff.get(&selector) {
                let final_sel = wrap_with_media(&selector, bp_key.as_deref(), &bps);
                rules.push((final_sel, props.clone()));
                continue;
            }
            // 2) Dynamic generation for the base class (ignoring hover/breakpoint for props)
            if let Some(dynamic_props) = dynamic_css_properties_for_class(&base, &vars) {
                let sel = if hover { format!(".{}:hover", css_escape_class(&base)) } else { format!(".{}", css_escape_class(&base)) };
                let final_sel = wrap_with_media(&sel, bp_key.as_deref(), &bps);
                rules.push((final_sel, dynamic_props));
                continue;
            }
            // 3) Fallback: class key itself in theme (rare)
            if let Some(props) = eff.get(&base) {
                let final_sel = wrap_with_media(&selector, bp_key.as_deref(), &bps);
                rules.push((final_sel, props.clone()));
            }
        }

        post_process_css(&rules, &vars)
    }

    pub fn android_base_styles(&self, selector: &str, classes: &[String]) -> IndexMap<String, serde_json::Value> {
        let (eff, vars) = self.effective_theme_all();
        let mut out: IndexMap<String, serde_json::Value> = IndexMap::new();

        // Pre-insert androidOrientation to ensure it's early in the map for gap processing
        out.insert("androidOrientation".to_string(), serde_json::json!("vertical"));

        let mut combined_props = CssProps::new();

        // 1. Apply hardcoded platform defaults (lowest priority)
        match selector.to_lowercase().as_str() {
            "div" => {
                combined_props.insert("width".into(), json!("match_parent"));
            }
            "p" => {
                combined_props.insert("width".into(), json!("match_parent"));
                combined_props.insert("margin-vertical".into(), json!("16px"));
            }
            "h1" => {
                combined_props.insert("width".into(), json!("match_parent"));
                combined_props.insert("font-size".into(), json!("32px"));
                combined_props.insert("font-weight".into(), json!("bold"));
                combined_props.insert("margin-vertical".into(), json!("21.44px"));
            }
            "h2" => {
                combined_props.insert("width".into(), json!("match_parent"));
                combined_props.insert("font-size".into(), json!("24px"));
                combined_props.insert("font-weight".into(), json!("bold"));
                combined_props.insert("margin-vertical".into(), json!("19.92px"));
            }
            "h3" => {
                combined_props.insert("width".into(), json!("match_parent"));
                combined_props.insert("font-size".into(), json!("18.72px"));
                combined_props.insert("font-weight".into(), json!("bold"));
                combined_props.insert("margin-vertical".into(), json!("18.72px"));
            }
            "h4" => {
                combined_props.insert("width".into(), json!("match_parent"));
                combined_props.insert("font-size".into(), json!("16px"));
                combined_props.insert("font-weight".into(), json!("bold"));
                combined_props.insert("margin-vertical".into(), json!("21.28px"));
            }
            "h5" => {
                combined_props.insert("width".into(), json!("match_parent"));
                combined_props.insert("font-size".into(), json!("13.28px"));
                combined_props.insert("font-weight".into(), json!("bold"));
                combined_props.insert("margin-vertical".into(), json!("22.17px"));
            }
            "h6" => {
                combined_props.insert("width".into(), json!("match_parent"));
                combined_props.insert("font-size".into(), json!("10.72px"));
                combined_props.insert("font-weight".into(), json!("bold"));
                combined_props.insert("margin-vertical".into(), json!("24.96px"));
            }
            "input" => {
                combined_props.insert("padding-vertical".into(), json!("8px"));
                combined_props.insert("padding-horizontal".into(), json!("12px"));
                combined_props.insert("border-radius".into(), json!("4px"));
                combined_props.insert("border-width".into(), json!("1px"));
                combined_props.insert("border-color".into(), json!("#cccccc"));
                combined_props.insert("background-color".into(), json!("#ffffff"));
                combined_props.insert("color".into(), json!("#000000"));
                combined_props.insert("placeholder-color".into(), json!("#88888870"));
                combined_props.insert("min-height".into(), json!("40px"));
                combined_props.insert("android-gravity".into(), json!("center_vertical"));
            }
            "select" => {
                combined_props.insert("padding-vertical".into(), json!("8px"));
                combined_props.insert("padding-horizontal".into(), json!("12px"));
                combined_props.insert("border-radius".into(), json!("4px"));
                combined_props.insert("border-width".into(), json!("1px"));
                combined_props.insert("border-color".into(), json!("#cccccc"));
                combined_props.insert("background-color".into(), json!("#ffffff"));
                combined_props.insert("color".into(), json!("#000000"));
                combined_props.insert("min-height".into(), json!("40px"));
                combined_props.insert("android-gravity".into(), json!("center_vertical"));
            }
            "textarea" => {
                combined_props.insert("padding".into(), json!("12px"));
                combined_props.insert("border-radius".into(), json!("4px"));
                combined_props.insert("border-width".into(), json!("1px"));
                combined_props.insert("border-color".into(), json!("#cccccc"));
                combined_props.insert("background-color".into(), json!("#ffffff"));
                combined_props.insert("color".into(), json!("#000000"));
                combined_props.insert("placeholder-color".into(), json!("color-mix(in srgb, currentColor 75%, grey)"));
                combined_props.insert("min-height".into(), json!("80px"));
                combined_props.insert("android-gravity".into(), json!("top"));
            }
            "button" => {
                combined_props.insert("padding-vertical".into(), json!("8px"));
                combined_props.insert("padding-horizontal".into(), json!("16px"));
                combined_props.insert("border-radius".into(), json!("4px"));
                combined_props.insert("background-color".into(), json!("#2196F3"));
                combined_props.insert("color".into(), json!("#ffffff"));
                combined_props.insert("android-gravity".into(), json!("center"));
            }
            _ => {}
        }

        if selector == "button" || selector == "input" || selector == "textarea" || classes.iter().any(|c| c.contains("bg-")) {
            log::debug!("[android_base_styles] selector={} classes={:?}", selector, classes);
        }

        // 2. Apply theme selector styles (overwrites defaults)
        if let Some(props) = eff.get(selector) {
            merge_props(&mut combined_props, props);
        }

        // 3. Apply class styles (overwrites selector)
        for class in classes {
            // Normalize input: strip leading dot if present (Android may pass ".bg-primary" as selector format)
            let normalized_class = if class.starts_with('.') {
                class[1..].to_string()
            } else {
                class.clone()
            };
            
            let (_bp, _hover, base) = parse_prefixed_class(&normalized_class);
            // Prefer base selector match from theme
            let sel = class_to_selector(&base);
            if let Some(props) = eff.get(&sel) {
                merge_props(&mut combined_props, props);
                continue;
            }
            // Dynamic mapping for base class
            if let Some(dynamic_props) = dynamic_css_properties_for_class(&base, &vars) {
                merge_props(&mut combined_props, &dynamic_props);
                continue;
            }
            if let Some(props) = eff.get(&base) {
                merge_props(&mut combined_props, props);
            }
        }

        if selector == "input" || selector == "textarea" {
            log::debug!("[android_base_styles] combined_props for {}: {:?}", selector, combined_props);
        }

        merge_android_props(&mut out, &combined_props, &vars);
        
        // CSS semantics: display: flex defaults to flexDirection: row
        if let Some(display) = out.get("display") {
            if display.as_str() == Some("flex") && !out.contains_key("flexDirection") {
                out.insert("flexDirection".to_string(), serde_json::json!("row"));
                out.insert("androidOrientation".to_string(), serde_json::json!("horizontal"));
            }
        }

        // Fallback for block elements to be column if not specified
        if !out.contains_key("flexDirection") {
            match selector.to_lowercase().as_str() {
                "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" => {
                    out.insert("flexDirection".to_string(), serde_json::json!("column"));
                    out.insert("androidOrientation".to_string(), serde_json::json!("vertical"));
                }
                _ => {}
            }
        }
        
        // Ensure androidOrientation is in sync with flexDirection if it was set but orientation wasn't
        if let Some(fd) = out.get("flexDirection").and_then(|v| v.as_str()) {
            if !out.contains_key("androidOrientation") {
                let orientation = if fd == "column" || fd == "column-reverse" {
                    "vertical"
                } else {
                    "horizontal"
                };
                out.insert("androidOrientation".to_string(), serde_json::json!(orientation));
            }
        }

        out
    }

    /// Android-specific style transformations
    /// Converts CSS properties to Android-compatible values with platform-specific defaults
    /// Handles unit conversions (dp/sp to px) using display density
    pub fn android_styles_for(&self, selector: &str, classes: &[String]) -> IndexMap<String, serde_json::Value> {
        let mut styles = self.android_base_styles(selector, classes);
        
        let density = self.display_density;
        let scaled_density = self.scaled_density;

        // Convert flexDirection to Android orientation EARLY so layout-dependent props (like gap) can use it
        if let Some(flex_dir) = styles.get("flexDirection") {
            let orientation = if flex_dir.as_str() == Some("row") { "horizontal" } else { "vertical" };
            styles.shift_insert(0, "androidOrientation".to_string(), serde_json::json!(orientation));
        }
        
        // Convert all dimension properties to pixels
        let dimension_props = [
            "width", "height", "minWidth", "minHeight", "maxWidth", "maxHeight",
            "padding", "paddingTop", "paddingBottom", "paddingLeft", "paddingRight",
            "paddingHorizontal", "paddingVertical",
            "margin", "marginTop", "marginBottom", "marginLeft", "marginRight",
            "marginHorizontal", "marginVertical",
            "borderRadius", "borderWidth", "borderTopWidth", "borderBottomWidth",
            "borderLeftWidth", "borderRightWidth",
            "gap", "rowGap", "columnGap", "elevation", "lineHeight", "letterSpacing"
        ];
        
        for prop in &dimension_props {
            if let Some(value) = styles.get(*prop).cloned() {
                if let Some(converted) = parse_and_convert_to_px(&value, density) {
                    styles.insert(prop.to_string(), converted);
                }
            }
        }
        
        // Convert font sizes (use scaled density for accessibility)
        if let Some(font_size) = styles.get("fontSize").cloned() {
            if let Some(serde_json::Value::Number(n)) = parse_and_convert_to_px(&font_size, density).as_ref() {
                // For text, use scaled density for accessibility
                let sp_value = n.as_f64().unwrap_or(14.0) as f32 / density * scaled_density;
                styles.insert("fontSize".to_string(), serde_json::json!(sp_value));
            }
        }
        
        // Convert flexWrap to Android-friendly format
        if let Some(flex_wrap) = styles.get("flexWrap") {
            if flex_wrap.as_str() == Some("wrap") {
                styles.insert("androidFlexWrap".to_string(), serde_json::json!(true));
            }
        }

        // Map opacity to androidAlpha
        if let Some(opacity) = styles.get("opacity").cloned() {
            styles.insert("androidAlpha".to_string(), opacity);
        }

        let is_horizontal = styles.get("androidOrientation").and_then(|v| v.as_str()) == Some("horizontal");
        let mut gravity_parts = Vec::new();

        // Convert alignItems (cross-axis) to Android gravity equivalents
        if let Some(align_items) = styles.get("alignItems") {
            let part = match align_items.as_str() {
                Some("center") => if is_horizontal { "center_vertical" } else { "center_horizontal" },
                Some("flex-start") | Some("start") => if is_horizontal { "top" } else { "start" },
                Some("flex-end") | Some("end") => if is_horizontal { "bottom" } else { "end" },
                Some("stretch") => if is_horizontal { "fill_vertical" } else { "fill_horizontal" },
                _ => ""
            };
            if !part.is_empty() {
                gravity_parts.push(part);
            }
        }
        
        // Convert justifyContent (main-axis) to Android gravity equivalents
        if let Some(justify) = styles.get("justifyContent") {
            let part = match justify.as_str() {
                Some("center") => if is_horizontal { "center_horizontal" } else { "center_vertical" },
                Some("flex-start") | Some("start") => if is_horizontal { "start" } else { "top" },
                Some("flex-end") | Some("end") => if is_horizontal { "end" } else { "bottom" },
                _ => ""
            };
            if !part.is_empty() {
                gravity_parts.push(part);
            }

            // Also keep layout gravity for compatibility or non-LinearLayout parents
            let layout_gravity = match justify.as_str() {
                Some("center") => "center_horizontal",
                Some("flex-start") | Some("start") => "start",
                Some("flex-end") | Some("end") => "end",
                Some("space-between") | Some("between") => "space_between",
                Some("space-around") | Some("around") => "space_around",
                _ => ""
            };
            if !layout_gravity.is_empty() {
                styles.insert("androidLayoutGravity".to_string(), serde_json::json!(layout_gravity));
            }
        }

        if !gravity_parts.is_empty() {
            let gravity = if gravity_parts.contains(&"center_vertical") && gravity_parts.contains(&"center_horizontal") {
                "center".to_string()
            } else {
                gravity_parts.join("|")
            };
            styles.insert("androidGravity".to_string(), serde_json::json!(gravity));
        }

        // Handle border shorthand: "1px solid #color"
        if let Some(serde_json::Value::String(border)) = styles.get("border").cloned() {
            let parts: Vec<&str> = border.split_whitespace().collect();
            for part in parts {
                if part.ends_with("px") {
                    if let Ok(w) = part.trim_end_matches("px").parse::<f32>() {
                        styles.insert("borderWidth".to_string(), serde_json::json!(dp_to_px(w, density)));
                    }
                } else if part.starts_with('#') {
                    styles.insert("borderColor".to_string(), serde_json::json!(part));
                }
            }
        }

        // Map boxShadow to elevation
        if let Some(serde_json::Value::String(shadow)) = styles.get("boxShadow").cloned() {
            if !shadow.is_empty() {
                let elevation = if shadow.contains("20px") { 24 }
                               else if shadow.contains("15px") { 16 }
                               else if shadow.contains("10px") { 8 }
                               else { 4 };
                styles.insert("elevation".to_string(), serde_json::json!(dp_to_px(elevation as f32, density)));
            }
        }

        // Convert overflow-x/overflow-y to Android scrolling hints
        if let Some(overflow_x) = styles.get("overflowX") {
            if overflow_x.as_str() == Some("auto") || overflow_x.as_str() == Some("scroll") {
                styles.insert("androidScrollHorizontal".to_string(), serde_json::json!(true));
            }
        }
        if let Some(overflow_y) = styles.get("overflowY") {
            if overflow_y.as_str() == Some("auto") || overflow_y.as_str() == Some("scroll") {
                styles.insert("androidScrollVertical".to_string(), serde_json::json!(true));
            }
        }
        
        // Convert textAlign to Android gravity
        if let Some(text_align) = styles.get("textAlign") {
            let gravity = match text_align.as_str() {
                Some("center") => "center_horizontal",
                Some("right") | Some("end") => "end",
                Some("left") | Some("start") => "start",
                _ => ""
            };
            if !gravity.is_empty() {
                styles.insert("androidTextGravity".to_string(), serde_json::json!(gravity));
            }
        }

        // Convert objectFit to Android scaleType
        if let Some(object_fit) = styles.get("objectFit") {
            let scale_type = match object_fit.as_str() {
                Some("cover") => "center_crop",
                Some("contain") => "fit_center",
                Some("fill") => "fit_xy",
                Some("none") => "center",
                Some("scale-down") => "center_inside",
                _ => ""
            };
            if !scale_type.is_empty() {
                styles.insert("androidScaleType".to_string(), serde_json::json!(scale_type));
            }
        }

        // Handle full width/height
        if let Some(h) = styles.get("height").cloned() {
            if h.as_str() == Some("100%") {
                styles.insert("height".to_string(), serde_json::json!("match_parent"));
            }
        }
        if let Some(w) = styles.get("width").cloned() {
            if w.as_str() == Some("100%") {
                styles.insert("width".to_string(), serde_json::json!("match_parent"));
            }
        }

        // Handle flex/weight: if flex is present, set the dimension in the orientation direction to 0
        // Note: We don't set it to 0 here anymore because we don't know if the parent is a LinearLayout.
        // The NativeRenderer will handle setting it to 0 if it's inside a LinearLayout.
        if styles.contains_key("flex") || styles.contains_key("flexGrow") {
            // Just ensure we have some dimension if not specified
            if !styles.contains_key("width") {
                styles.insert("width".to_string(), serde_json::json!("wrap_content"));
            }
            if !styles.contains_key("height") {
                styles.insert("height".to_string(), serde_json::json!("wrap_content"));
            }
        }
        
        // Convert fontWeight to Android typeface style
        if let Some(font_weight) = styles.get("fontWeight") {
            let is_bold = match font_weight {
                serde_json::Value::String(s) => s.contains("bold") || s == "600" || s == "700" || s == "500",
                serde_json::Value::Number(n) => {
                    let weight = n.as_i64().unwrap_or(400);
                    weight >= 500
                }
                _ => false
            };
            if is_bold {
                styles.insert("androidTypefaceStyle".to_string(), serde_json::json!("bold"));
            }
        }
        
        // Convert boxShadow to elevation
        if let Some(box_shadow) = styles.get("boxShadow") {
            if let Some(shadow_str) = box_shadow.as_str() {
                if !shadow_str.is_empty() {
                    let elevation_dp = if shadow_str.contains("20px") { 24.0 }
                    else if shadow_str.contains("15px") { 16.0 }
                    else if shadow_str.contains("10px") { 8.0 }
                    else if shadow_str.contains("5px") { 4.0 }
                    else { 4.0 };
                    styles.insert("elevation".to_string(), serde_json::json!(dp_to_px(elevation_dp, density)));
                }
            }
        }
        
        styles
    }

    // Previously supported loading YAML at runtime; now defaults are embedded.

    // Build the inheritance chain from current theme upward via `inherits` and default fallback
    fn theme_chain(&self) -> Vec<String> {
        let mut chain = Vec::new();
        // Resolve base names
        let default_name = if self.themes.contains_key(&self.default_theme) {
            self.default_theme.clone()
        } else if let Some((k, _)) = self.themes.first() { k.clone() } else { return chain };
        let mut current_name = if self.themes.contains_key(&self.current_theme) {
            self.current_theme.clone()
        } else { default_name.clone() };
        // push child first
        let mut seen: IndexSet<String> = IndexSet::new();
        while !seen.contains(&current_name) {
            seen.insert(current_name.clone());
            chain.push(current_name.clone());
            // next parent via inherits, else stop
            let inherits = self.themes.get(&current_name).and_then(|t| t.inherits.clone());
            if let Some(p) = inherits {
                current_name = p;
            } else {
                break;
            }
        }
        if !chain.iter().any(|n| n == &default_name) {
            chain.push(default_name);
        }
        chain
    }

    // Compute effective selectors + variables + breakpoints with inheritance.
    // Child overrides parent/default on conflicts (expected for "inherits").
    fn effective_theme_all(&self) -> (SelectorStyles, IndexMap<String, String>) {
        let mut selectors: SelectorStyles = SelectorStyles::new();
        let mut vars: IndexMap<String, String> = IndexMap::new();
        // Merge default -> parents -> child so child wins on conflicts
        let chain = self.theme_chain();
        for name in chain.into_iter().rev() {
            if let Some(entry) = self.themes.get(&name) {
                // merge selectors: later (child) overrides earlier (parent/default)
                for (sel, props) in entry.selectors.iter() {
                    // Support multiple selectors separated by commas (e.g., "h1, h2, h3")
                    if sel.contains(',') {
                        for s in sel.split(',') {
                            let s = s.trim();
                            if s.is_empty() { continue; }
                            let e = selectors.entry(s.to_string()).or_default();
                            merge_props(e, props);
                        }
                    } else {
                        let e = selectors.entry(sel.clone()).or_default();
                        merge_props(e, props);
                    }
                }
                // merge variables
                for (k, v) in entry.variables.iter() {
                    vars.insert(k.clone(), v.clone());
                }
            }
        }
        (selectors, vars)
    }

    // Effective breakpoints with inheritance; child overrides parent/default.
    pub fn effective_breakpoints(&self) -> IndexMap<String, String> {
        let mut bps: IndexMap<String, String> = IndexMap::new();
        let chain = self.theme_chain();
        for name in chain.into_iter().rev() {
            if let Some(entry) = self.themes.get(&name) {
                for (k, v) in entry.breakpoints.iter() {
                    bps.insert(k.clone(), v.clone());
                }
            }
        }
        bps
    }
}

fn split_tag_class_key(key: &str) -> Option<(String, String)> {
    let mut it = key.splitn(2, '|');
    let t = it.next()?.to_string();
    let c = it.next()?.to_string();
    if t.is_empty() || c.is_empty() { return None; }
    Some((t, c))
}

fn strip_hover_suffix(selector: &str) -> (&str, bool) {
    if let Some(stripped) = selector.strip_suffix(":hover") { (stripped, true) } else { (selector, false) }
}

fn should_emit_selector(sel: &str, used_tags: &IndexSet<String>, used_classes: &IndexSet<String>, used_tag_classes: &IndexSet<String>) -> bool {
    // Optionally handle :hover suffix
    let (base, _hover) = strip_hover_suffix(sel);

    // tag-only
    if is_simple_tag(base) {
        return used_tags.contains(base) || used_tag_classes.iter().any(|k| k.split('|').next() == Some(base));
    }

    // .class-only
    if let Some(class_name) = base.strip_prefix('.') {
        // Normalize potential escaped class names as-is
        return used_classes.contains(class_name) || used_tag_classes.iter().any(|k| k.ends_with(&format!("|{}", class_name)));
    }

    // tag.class
    if let Some((tag, class_name)) = split_tag_class_selector(base) {
        let key = format!("{}|{}", tag, class_name);
        return used_tag_classes.contains(&key) || (used_tags.contains(&tag) && used_classes.contains(&class_name));
    }

    // Other complex selectors are currently ignored
    false
}

fn is_simple_tag(s: &str) -> bool {
    // Match simple HTML tag-ish identifiers
    let mut chars = s.chars();
    match chars.next() { Some(c) if c.is_ascii_alphabetic() => {}, _ => return false }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn split_tag_class_selector(s: &str) -> Option<(String, String)> {
    // "tag.class" -> (tag, class)
    let mut parts = s.splitn(2, '.');
    let tag = parts.next()?.to_string();
    let class_name = parts.next()?.to_string();
    if tag.is_empty() || class_name.is_empty() { return None; }
    Some((tag, class_name))
}

// wasm-bindgen exports (only when compiling to wasm32)
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

// Expose crate version to JS via wasm-bindgen
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_version() -> String {
    // CARGO_PKG_VERSION is provided at compile time
    env!("CARGO_PKG_VERSION").to_string()
}

// Plain Rust accessor for crate version used by Android JNI glue
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Return the embedded default state as a JSON string.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_default_state_json() -> String {
    let st = bundled_state();
    match serde_json::to_string(&st.to_json()) {
        Ok(s) => s,
        Err(_) => "{}".to_string(),
    }
}

/// Register a theme from JSON. On duplicate, replace the theme's selectors, inheritance, and variables.
/// Expected JSON format: `{ "name": "theme-name", "theme": { "inherits": "parent", "selectors": {...}, "variables": {...}, "breakpoints": {...} } }`
/// Returns the updated state as JSON, or "{}" on error.
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

/// Set the default and current theme. Returns the updated state as JSON.
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

/// Get all theme keys and names as JSON array: [{ "key": "default", "name": "Default Theme" }, ...]
/// Returns array of themes from the state JSON.
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

fn merge_props(into: &mut CssProps, from: &CssProps) {
    for (k, v) in from.iter() {
        into.insert(k.clone(), v.clone());
    }
}

// merge_indexmap removed — unused

fn css_props_string(props: &CssProps, vars: &IndexMap<String, String>) -> String {
    let mut buf = String::new();
    for (k, v) in props.iter() {
        let key = crate::utils::kebab_case(k);
        buf.push_str(&key);
        buf.push(':');
        let val = if v.is_string() {
            let s = v.as_str().unwrap();
            resolve_vars(s, vars)
        } else {
            v.to_string()
        };
        buf.push_str(&val);
        if !val.ends_with(';') {
            buf.push(';');
        }
    }
    buf
}

/// Parse var() references manually (replaces regex dependency)
/// Matches: var(--name), var(name), with optional whitespace
/// Supports alphanumeric, underscore, dot, and dash in variable names
fn parse_var_references(input: &str) -> Vec<(usize, usize, String)> {
    let mut results = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    
    while i < bytes.len() {
        // Look for "var("
        if i + 4 <= bytes.len() && &bytes[i..i+4] == b"var(" {
            let start = i;
            i += 4;
            
            // Skip whitespace
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\n' || bytes[i] == b'\r') {
                i += 1;
            }
            
            // Check for optional -- prefix
            let has_prefix = i + 2 <= bytes.len() && &bytes[i..i+2] == b"--";
            if has_prefix {
                i += 2;
            }
            
            // Collect variable name: [a-zA-Z0-9_.-]+
            let name_start = i;
            while i < bytes.len() {
                let c = bytes[i];
                if (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || 
                   (c >= b'0' && c <= b'9') || c == b'_' || c == b'.' || c == b'-' {
                    i += 1;
                } else {
                    break;
                }
            }
            
            let name_end = i;
            if name_start < name_end {
                // Skip trailing whitespace
                while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\n' || bytes[i] == b'\r') {
                    i += 1;
                }
                
                // Check for closing )
                if i < bytes.len() && bytes[i] == b')' {
                    let end = i + 1;
                    let var_name = std::str::from_utf8(&bytes[name_start..name_end])
                        .unwrap_or("").to_string();
                    results.push((start, end, var_name));
                    i = end;
                    continue;
                }
            }
        }
        i += 1;
    }
    
    results
}

// Tailwind color palette - embedded from tailwind-colors.html
static TAILWIND_COLORS: Lazy<IndexMap<&'static str, IndexMap<&'static str, &'static str>>> = Lazy::new(|| {
    let mut colors = IndexMap::new();
    
    let mut slate = IndexMap::new();
    slate.insert("50", "#f8fafc"); slate.insert("100", "#f1f5f9"); slate.insert("200", "#e2e8f0");
    slate.insert("300", "#cbd5e1"); slate.insert("400", "#94a3b8"); slate.insert("500", "#64748b");
    slate.insert("600", "#475569"); slate.insert("700", "#334155"); slate.insert("800", "#1e293b");
    slate.insert("900", "#0f172a"); slate.insert("950", "#020617");
    colors.insert("slate", slate);
    
    let mut gray = IndexMap::new();
    gray.insert("50", "#f9fafb"); gray.insert("100", "#f3f4f6"); gray.insert("200", "#e5e7eb");
    gray.insert("300", "#d1d5db"); gray.insert("400", "#9ca3af"); gray.insert("500", "#6b7280");
    gray.insert("600", "#4b5563"); gray.insert("700", "#374151"); gray.insert("800", "#1f2937");
    gray.insert("900", "#111827"); gray.insert("950", "#030712");
    colors.insert("gray", gray);
    
    let mut zinc = IndexMap::new();
    zinc.insert("50", "#fafafa"); zinc.insert("100", "#f4f4f5"); zinc.insert("200", "#e4e4e7");
    zinc.insert("300", "#d4d4d8"); zinc.insert("400", "#a1a1aa"); zinc.insert("500", "#71717a");
    zinc.insert("600", "#52525b"); zinc.insert("700", "#3f3f46"); zinc.insert("800", "#27272a");
    zinc.insert("900", "#18181b"); zinc.insert("950", "#09090b");
    colors.insert("zinc", zinc);
    
    let mut neutral = IndexMap::new();
    neutral.insert("50", "#fafafa"); neutral.insert("100", "#f5f5f5"); neutral.insert("200", "#e5e5e5");
    neutral.insert("300", "#d4d4d4"); neutral.insert("400", "#a3a3a3"); neutral.insert("500", "#737373");
    neutral.insert("600", "#525252"); neutral.insert("700", "#404040"); neutral.insert("800", "#262626");
    neutral.insert("900", "#171717"); neutral.insert("950", "#0a0a0a");
    colors.insert("neutral", neutral);
    
    let mut stone = IndexMap::new();
    stone.insert("50", "#fafaf9"); stone.insert("100", "#f5f5f4"); stone.insert("200", "#e7e5e4");
    stone.insert("300", "#d6d3d1"); stone.insert("400", "#a8a29e"); stone.insert("500", "#78716c");
    stone.insert("600", "#57534e"); stone.insert("700", "#44403c"); stone.insert("800", "#292524");
    stone.insert("900", "#1c1917"); stone.insert("950", "#0c0a09");
    colors.insert("stone", stone);
    
    let mut red = IndexMap::new();
    red.insert("50", "#fef2f2"); red.insert("100", "#fee2e2"); red.insert("200", "#fecaca");
    red.insert("300", "#fca5a5"); red.insert("400", "#f87171"); red.insert("500", "#ef4444");
    red.insert("600", "#dc2626"); red.insert("700", "#b91c1c"); red.insert("800", "#991b1b");
    red.insert("900", "#7f1d1d"); red.insert("950", "#450a0a");
    colors.insert("red", red);
    
    let mut orange = IndexMap::new();
    orange.insert("50", "#fff7ed"); orange.insert("100", "#ffedd5"); orange.insert("200", "#fed7aa");
    orange.insert("300", "#fdba74"); orange.insert("400", "#fb923c"); orange.insert("500", "#f97316");
    orange.insert("600", "#ea580c"); orange.insert("700", "#c2410c"); orange.insert("800", "#9a3412");
    orange.insert("900", "#7c2d12"); orange.insert("950", "#431407");
    colors.insert("orange", orange);
    
    let mut amber = IndexMap::new();
    amber.insert("50", "#fffbeb"); amber.insert("100", "#fef3c7"); amber.insert("200", "#fde68a");
    amber.insert("300", "#fcd34d"); amber.insert("400", "#fbbf24"); amber.insert("500", "#f59e0b");
    amber.insert("600", "#d97706"); amber.insert("700", "#b45309"); amber.insert("800", "#92400e");
    amber.insert("900", "#78350f"); amber.insert("950", "#451a03");
    colors.insert("amber", amber);
    
    let mut blue = IndexMap::new();
    blue.insert("50", "#eff6ff"); blue.insert("100", "#dbeafe"); blue.insert("200", "#bfdbfe");
    blue.insert("300", "#93c5fd"); blue.insert("400", "#60a5fa"); blue.insert("500", "#3b82f6");
    blue.insert("600", "#2563eb"); blue.insert("700", "#1d4ed8"); blue.insert("800", "#1e40af");
    blue.insert("900", "#1e3a8a"); blue.insert("950", "#0b1c52");
    colors.insert("blue", blue);
    
    let mut lime = IndexMap::new();
    lime.insert("50", "#f7fee7"); lime.insert("100", "#ecfccb"); lime.insert("200", "#d9f99d");
    lime.insert("300", "#bef264"); lime.insert("400", "#a3e635"); lime.insert("500", "#84cc16");
    lime.insert("600", "#65a30d"); lime.insert("700", "#4d7c0f"); lime.insert("800", "#3f6212");
    lime.insert("900", "#365314"); lime.insert("950", "#1a2e05");
    colors.insert("lime", lime);
    
    let mut green = IndexMap::new();
    green.insert("50", "#f0fdf4"); green.insert("100", "#dcfce7"); green.insert("200", "#bbf7d0");
    green.insert("300", "#86efac"); green.insert("400", "#4ade80"); green.insert("500", "#22c55e");
    green.insert("600", "#16a34a"); green.insert("700", "#15803d"); green.insert("800", "#166534");
    green.insert("900", "#14532d"); green.insert("950", "#052e16");
    colors.insert("green", green);
    
    let mut emerald = IndexMap::new();
    emerald.insert("50", "#ecfdf5"); emerald.insert("100", "#d1fae5"); emerald.insert("200", "#a7f3d0");
    emerald.insert("300", "#6ee7b7"); emerald.insert("400", "#34d399"); emerald.insert("500", "#10b981");
    emerald.insert("600", "#059669"); emerald.insert("700", "#047857"); emerald.insert("800", "#065f46");
    emerald.insert("900", "#064e3b"); emerald.insert("950", "#022c22");
    colors.insert("emerald", emerald);
    
    let mut teal = IndexMap::new();
    teal.insert("50", "#f0fdfa"); teal.insert("100", "#ccfbf1"); teal.insert("200", "#99f6e4");
    teal.insert("300", "#5eead4"); teal.insert("400", "#2dd4bf"); teal.insert("500", "#14b8a6");
    teal.insert("600", "#0d9488"); teal.insert("700", "#0f766e"); teal.insert("800", "#115e59");
    teal.insert("900", "#134e4a"); teal.insert("950", "#042f2e");
    colors.insert("teal", teal);
    
    let mut cyan = IndexMap::new();
    cyan.insert("50", "#ecfeff"); cyan.insert("100", "#cffafe"); cyan.insert("200", "#a5f3fc");
    cyan.insert("300", "#67e8f9"); cyan.insert("400", "#22d3ee"); cyan.insert("500", "#06b6d4");
    cyan.insert("600", "#0891b2"); cyan.insert("700", "#0e7490"); cyan.insert("800", "#155e75");
    cyan.insert("900", "#164e63"); cyan.insert("950", "#083344");
    colors.insert("cyan", cyan);
    
    let mut sky = IndexMap::new();
    sky.insert("50", "#f0f9ff"); sky.insert("100", "#e0f2fe"); sky.insert("200", "#bae6fd");
    sky.insert("300", "#7dd3fc"); sky.insert("400", "#38bdf8"); sky.insert("500", "#0ea5e9");
    sky.insert("600", "#0284c7"); sky.insert("700", "#0369a1"); sky.insert("800", "#075985");
    sky.insert("900", "#0c4a6e"); sky.insert("950", "#082f49");
    colors.insert("sky", sky);
    
    let mut blue = IndexMap::new();
    blue.insert("50", "#eff6ff"); blue.insert("100", "#dbeafe"); blue.insert("200", "#bfdbfe");
    blue.insert("300", "#93c5fd"); blue.insert("400", "#60a5fa"); blue.insert("500", "#3b82f6");
    blue.insert("600", "#2563eb"); blue.insert("700", "#1d4ed8"); blue.insert("800", "#1e40af");
    blue.insert("900", "#1e3a8a"); blue.insert("950", "#172554");
    colors.insert("blue", blue);
    
    let mut indigo = IndexMap::new();
    indigo.insert("50", "#eef2ff"); indigo.insert("100", "#e0e7ff"); indigo.insert("200", "#c7d2fe");
    indigo.insert("300", "#a5b4fc"); indigo.insert("400", "#818cf8"); indigo.insert("500", "#6366f1");
    indigo.insert("600", "#4f46e5"); indigo.insert("700", "#4338ca"); indigo.insert("800", "#3730a3");
    indigo.insert("900", "#312e81"); indigo.insert("950", "#1e1b4b");
    colors.insert("indigo", indigo);
    
    let mut violet = IndexMap::new();
    violet.insert("50", "#f5f3ff"); violet.insert("100", "#ede9fe"); violet.insert("200", "#ddd6fe");
    violet.insert("300", "#c4b5fd"); violet.insert("400", "#a78bfa"); violet.insert("500", "#8b5cf6");
    violet.insert("600", "#7c3aed"); violet.insert("700", "#6d28d9"); violet.insert("800", "#5b21b6");
    violet.insert("900", "#4c1d95"); violet.insert("950", "#2e1065");
    colors.insert("violet", violet);
    
    let mut purple = IndexMap::new();
    purple.insert("50", "#faf5ff"); purple.insert("100", "#f3e8ff"); purple.insert("200", "#e9d5ff");
    purple.insert("300", "#d8b4fe"); purple.insert("400", "#c084fc"); purple.insert("500", "#a855f7");
    purple.insert("600", "#9333ea"); purple.insert("700", "#7e22ce"); purple.insert("800", "#6b21a8");
    purple.insert("900", "#581c87"); purple.insert("950", "#3b0764");
    colors.insert("purple", purple);
    
    let mut fuchsia = IndexMap::new();
    fuchsia.insert("50", "#fdf4ff"); fuchsia.insert("100", "#fae8ff"); fuchsia.insert("200", "#f5d0fe");
    fuchsia.insert("300", "#f0abfc"); fuchsia.insert("400", "#e879f9"); fuchsia.insert("500", "#d946ef");
    fuchsia.insert("600", "#c026d3"); fuchsia.insert("700", "#a21caf"); fuchsia.insert("800", "#86198f");
    fuchsia.insert("900", "#701a75"); fuchsia.insert("950", "#4a044e");
    colors.insert("fuchsia", fuchsia);
    
    let mut pink = IndexMap::new();
    pink.insert("50", "#fdf2f8"); pink.insert("100", "#fce7f3"); pink.insert("200", "#fbcfe8");
    pink.insert("300", "#f9a8d4"); pink.insert("400", "#f472b6"); pink.insert("500", "#ec4899");
    pink.insert("600", "#db2777"); pink.insert("700", "#be185d"); pink.insert("800", "#9d174d");
    pink.insert("900", "#831843"); pink.insert("950", "#500724");
    colors.insert("pink", pink);
    
    let mut rose = IndexMap::new();
    rose.insert("50", "#fff1f2"); rose.insert("100", "#ffe4e6"); rose.insert("200", "#fecdd3");
    rose.insert("300", "#fda4af"); rose.insert("400", "#fb7185"); rose.insert("500", "#f43f5e");
    rose.insert("600", "#e11d48"); rose.insert("700", "#be123c"); rose.insert("800", "#9f1239");
    rose.insert("900", "#881337"); rose.insert("950", "#4c0519");
    colors.insert("rose", rose);
    
    colors
});

fn resolve_vars(input: &str, vars: &IndexMap<String, String>) -> String {
    let var_refs = parse_var_references(input);
    
    if var_refs.is_empty() {
        // Fast path: no var() references, just check for $ prefix
        if input.starts_with('$') {
            if let Some(val) = vars.get(&input[1..]) {
                return val.clone();
            }
        }
        return input.to_string();
    }
    
    // Replace var() references from right to left to preserve indices
    let mut out = input.to_string();
    for (start, end, var_name) in var_refs.iter().rev() {
        if let Some(val) = vars.get(var_name) {
            out.replace_range(*start..*end, val);
        }
    }
    
    // Also handle $ prefix for direct variable references
    if out.starts_with('$') {
        if let Some(val) = vars.get(&out[1..]) {
            return val.clone();
        }
    }
    
    out
}

fn camel_case(name: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for ch in name.chars() {
        if ch == '-' {
            upper = true;
            continue;
        }
        if upper {
            out.extend(ch.to_uppercase());
            upper = false;
        } else {
            out.push(ch);
        }
    }
    out
}

fn css_value_to_android(
    value: &serde_json::Value,
    vars: &IndexMap<String, String>,
    current_color: Option<&str>,
) -> serde_json::Value {
    if let Some(s) = value.as_str() {
        if s.contains("color-mix") {
            log::debug!("[css_value_to_android] color-mix detected: {} current_color={:?}", s, current_color);
        }
    }
    match value {
        serde_json::Value::String(s) => {
            let s2 = resolve_vars(s, vars);
            let s3 = color::resolve_color(&s2, current_color, vars);
            if let Some(n) = s3.strip_suffix("px") {
                if let Ok(parsed) = n.trim().parse::<f64>() {
                    return json!(parsed);
                }
            }
            json!(s3)
        }
        _ => value.clone(),
    }
}

fn merge_android_props(
    into: &mut IndexMap<String, serde_json::Value>,
    css_props: &CssProps,
    vars: &IndexMap<String, String>,
) {
    log::debug!("[merge_android_props] START props_count={}", css_props.len());
    // 1. Find current color for currentColor resolution
    let mut current_color = css_props.get("color")
        .and_then(|v| v.as_str())
        .map(|s| resolve_vars(s, vars));
    
    if current_color.is_none() {
        current_color = into.get("color").and_then(|v| v.as_str()).map(|s| s.to_string());
    }

    if let Some(ref c) = current_color {
        log::debug!("[merge_android_props] current_color resolved to: {}", c);
    }

    for (k, v) in css_props.iter() {
        let val = css_value_to_android(v, vars, current_color.as_deref());
        
        if k == "placeholder-color" || k == "placeholderColor" {
            log::debug!("[merge_android_props] placeholder-color: input={:?} output={:?}", v, val);
        }

        match k.as_str() {
            "padding" => {
                into.insert("paddingTop".to_string(), val.clone());
                into.insert("paddingBottom".to_string(), val.clone());
                into.insert("paddingLeft".to_string(), val.clone());
                into.insert("paddingRight".to_string(), val.clone());
                into.insert("paddingHorizontal".to_string(), val.clone());
                into.insert("paddingVertical".to_string(), val.clone());
                into.insert("padding".to_string(), val);
            }
            "padding-horizontal" | "paddingHorizontal" => {
                into.insert("paddingLeft".to_string(), val.clone());
                into.insert("paddingRight".to_string(), val.clone());
                into.insert("paddingHorizontal".to_string(), val);
            }
            "padding-vertical" | "paddingVertical" => {
                into.insert("paddingTop".to_string(), val.clone());
                into.insert("paddingBottom".to_string(), val.clone());
                into.insert("paddingVertical".to_string(), val);
            }
            "margin" => {
                into.insert("marginTop".to_string(), val.clone());
                into.insert("marginBottom".to_string(), val.clone());
                into.insert("marginLeft".to_string(), val.clone());
                into.insert("marginRight".to_string(), val.clone());
                into.insert("marginHorizontal".to_string(), val.clone());
                into.insert("marginVertical".to_string(), val.clone());
                into.insert("margin".to_string(), val);
            }
            "margin-horizontal" | "marginHorizontal" => {
                into.insert("marginLeft".to_string(), val.clone());
                into.insert("marginRight".to_string(), val.clone());
                into.insert("marginHorizontal".to_string(), val);
            }
            "margin-vertical" | "marginVertical" => {
                into.insert("marginTop".to_string(), val.clone());
                into.insert("marginBottom".to_string(), val.clone());
                into.insert("marginVertical".to_string(), val);
            }
            "border-radius" | "borderRadius" => {
                into.insert("borderTopLeftRadius".to_string(), val.clone());
                into.insert("borderTopRightRadius".to_string(), val.clone());
                into.insert("borderBottomLeftRadius".to_string(), val.clone());
                into.insert("borderBottomRightRadius".to_string(), val.clone());
                into.insert("borderRadius".to_string(), val);
            }
            "background-color" => { into.insert("backgroundColor".to_string(), val); }
            "text-align" => { into.insert("textAlign".to_string(), val); }
            "flex-direction" | "flexDirection" => {
                let orientation = if val.as_str() == Some("column") || val.as_str() == Some("column-reverse") {
                    "vertical"
                } else {
                    "horizontal"
                };
                into.insert("androidOrientation".to_string(), serde_json::json!(orientation));
                into.insert("flexDirection".to_string(), val);
            }
            _ => {
                into.insert(camel_case(k), val);
            }
        }
    }
}

fn dynamic_css_properties_for_class(class: &str, vars: &IndexMap<String, String>) -> Option<CssProps> {
    // Display utilities
    match class {
        "block" => { let mut p = CssProps::new(); p.insert("display".into(), json!("block")); return Some(p); }
        "inline-block" => { let mut p = CssProps::new(); p.insert("display".into(), json!("inline-block")); return Some(p); }
        "inline" => { let mut p = CssProps::new(); p.insert("display".into(), json!("inline")); return Some(p); }
        "inline-flex" => { let mut p = CssProps::new(); p.insert("display".into(), json!("inline-flex")); return Some(p); }
        "grid" => { let mut p = CssProps::new(); p.insert("display".into(), json!("grid")); return Some(p); }
        "hidden" => { let mut p = CssProps::new(); p.insert("display".into(), json!("none")); return Some(p); }
        _ => {}
    }
    // Flexbox shorthands
    match class {
        "flex" => { let mut p = CssProps::new(); p.insert("display".into(), json!("flex")); return Some(p); }
        "flex-row" => { let mut p = CssProps::new(); p.insert("display".into(), json!("flex")); p.insert("flexDirection".into(), json!("row")); return Some(p); }
        "flex-col" => { let mut p = CssProps::new(); p.insert("display".into(), json!("flex")); p.insert("flexDirection".into(), json!("column")); return Some(p); }
        "flex-wrap" => { let mut p = CssProps::new(); p.insert("display".into(), json!("flex")); p.insert("flex-wrap".into(), json!("wrap")); return Some(p); }
        "flex-nowrap" => { let mut p = CssProps::new(); p.insert("display".into(), json!("flex")); p.insert("flex-wrap".into(), json!("nowrap")); return Some(p); }
        "flex-wrap-reverse" => { let mut p = CssProps::new(); p.insert("display".into(), json!("flex")); p.insert("flex-wrap".into(), json!("wrap-reverse")); return Some(p); }
        "flex-1" => { let mut p = CssProps::new(); p.insert("flex".into(), json!(1)); return Some(p); }
        "w-full" => { let mut p = CssProps::new(); p.insert("width".into(), json!("match_parent")); return Some(p); }
        "h-full" => { let mut p = CssProps::new(); p.insert("height".into(), json!("match_parent")); return Some(p); }
        _ => {}
    }
    if let Some(value) = class.strip_prefix("z-") {
        if let Ok(z) = value.parse::<i32>() {
            let mut p = CssProps::new();
            p.insert("elevation".into(), json!(z));
            return Some(p);
        }
    }
    if let Some(rest) = class.strip_prefix("items-") {
        let mut p = CssProps::new();
        let v = match rest { "start" => "flex-start", "end" => "flex-end", "center" => "center", "stretch" => "stretch", other => other };
        p.insert("align-items".into(), json!(v));
        return Some(p);
    }
    if let Some(rest) = class.strip_prefix("justify-") {
        let mut p = CssProps::new();
        let v = match rest { "start" => "flex-start", "end" => "flex-end", "center" => "center", "between" => "space-between", "around" => "space-around", "evenly" => "space-evenly", other => other };
        p.insert("justify-content".into(), json!(v));
        return Some(p);
    }
    if let Some(value) = class.strip_prefix("p-") {
        return parse_tailwind_spacing(value, &|px| padding_props(&["padding"], px));
    }
    if let Some(value) = class.strip_prefix("px-") {
        return parse_tailwind_spacing(value, &|px| padding_props(&["padding-left", "padding-right"], px));
    }
    if let Some(value) = class.strip_prefix("py-") {
        return parse_tailwind_spacing(value, &|px| padding_props(&["padding-top", "padding-bottom"], px));
    }
    for &(prefix, prop) in &[("pt-", "padding-top"), ("pr-", "padding-right"), ("pb-", "padding-bottom"), ("pl-", "padding-left")] {
        if let Some(value) = class.strip_prefix(prefix) {
            return parse_tailwind_spacing(value, &|px| padding_props(&[prop], px));
        }
    }
    // Margin utilities
    if let Some(value) = class.strip_prefix("m-") {
        if value == "auto" {
            let mut p = CssProps::new();
            p.insert("margin".into(), json!("auto"));
            return Some(p);
        }
        return parse_tailwind_spacing(value, &|px| margin_props(&["margin"], px));
    }
    if let Some(value) = class.strip_prefix("mx-") {
        if value == "auto" {
            let mut p = CssProps::new();
            p.insert("margin-left".into(), json!("auto"));
            p.insert("margin-right".into(), json!("auto"));
            return Some(p);
        }
        return parse_tailwind_spacing(value, &|px| margin_props(&["margin-left", "margin-right"], px));
    }
    if let Some(value) = class.strip_prefix("my-") {
        if value == "auto" {
            let mut p = CssProps::new();
            p.insert("margin-top".into(), json!("auto"));
            p.insert("margin-bottom".into(), json!("auto"));
            return Some(p);
        }
        return parse_tailwind_spacing(value, &|px| margin_props(&["margin-top", "margin-bottom"], px));
    }
    for &(prefix, prop) in &[("mt-", "margin-top"), ("mr-", "margin-right"), ("mb-", "margin-bottom"), ("ml-", "margin-left")] {
        if let Some(value) = class.strip_prefix(prefix) {
            if value == "auto" {
                let mut p = CssProps::new();
                p.insert(prop.into(), json!("auto"));
                return Some(p);
            }
            return parse_tailwind_spacing(value, &|px| margin_props(&[prop], px));
        }
    }
    // Gap utilities (works in Android with Flexbox)
    if let Some(value) = class.strip_prefix("gap-") {
        if !value.starts_with("x-") && !value.starts_with("y-") {
            return parse_tailwind_spacing(value, &|px| {
                let mut props = CssProps::new();
                props.insert("gap".into(), json!(format!("{}px", px)));
                props
            });
        }
    }
    if let Some(value) = class.strip_prefix("gap-x-") {
        return parse_tailwind_spacing(value, &|px| {
            let mut props = CssProps::new();
            props.insert("column-gap".into(), json!(format!("{}px", px)));
            props
        });
    }
    if let Some(value) = class.strip_prefix("gap-y-") {
        return parse_tailwind_spacing(value, &|px| {
            let mut props = CssProps::new();
            props.insert("row-gap".into(), json!(format!("{}px", px)));
            props
        });
    }
    // Space utilities (space-x-*, space-y-*)
    if let Some(value) = class.strip_prefix("space-x-") {
        return parse_tailwind_spacing(value, &|px| {
            let mut props = CssProps::new();
            // In CSS, this is typically done with :not(:last-child) selector
            // For now, we'll set it as a custom property that can be used
            props.insert("--space-x".into(), json!(format!("{}px", px)));
            props
        });
    }
    if let Some(value) = class.strip_prefix("space-y-") {
        return parse_tailwind_spacing(value, &|px| {
            let mut props = CssProps::new();
            props.insert("--space-y".into(), json!(format!("{}px", px)));
            props
        });
    }
    // Font weight utilities
    match class {
        "font-thin" => { let mut p = CssProps::new(); p.insert("font-weight".into(), json!("100")); return Some(p); }
        "font-extralight" => { let mut p = CssProps::new(); p.insert("font-weight".into(), json!("200")); return Some(p); }
        "font-light" => { let mut p = CssProps::new(); p.insert("font-weight".into(), json!("300")); return Some(p); }
        "font-normal" => { let mut p = CssProps::new(); p.insert("font-weight".into(), json!("400")); return Some(p); }
        "font-medium" => { let mut p = CssProps::new(); p.insert("font-weight".into(), json!("500")); return Some(p); }
        "font-semibold" => { let mut p = CssProps::new(); p.insert("font-weight".into(), json!("600")); return Some(p); }
        "font-bold" => { let mut p = CssProps::new(); p.insert("font-weight".into(), json!("700")); return Some(p); }
        "font-extrabold" => { let mut p = CssProps::new(); p.insert("font-weight".into(), json!("800")); return Some(p); }
        "font-black" => { let mut p = CssProps::new(); p.insert("font-weight".into(), json!("900")); return Some(p); }
        _ => {}
    }
    // Font family utilities
    match class {
        "font-sans" => { let mut p = CssProps::new(); p.insert("font-family".into(), json!("system-ui, -apple-system, sans-serif")); return Some(p); }
        "font-serif" => { let mut p = CssProps::new(); p.insert("font-family".into(), json!("Georgia, serif")); return Some(p); }
        "font-mono" => { let mut p = CssProps::new(); p.insert("font-family".into(), json!("ui-monospace, monospace")); return Some(p); }
        _ => {}
    }
    // Text size utilities
    match class {
        "text-xs" => { let mut p = CssProps::new(); p.insert("font-size".into(), json!("12px")); p.insert("line-height".into(), json!("16px")); return Some(p); }
        "text-sm" => { let mut p = CssProps::new(); p.insert("font-size".into(), json!("14px")); p.insert("line-height".into(), json!("20px")); return Some(p); }
        "text-base" => { let mut p = CssProps::new(); p.insert("font-size".into(), json!("16px")); p.insert("line-height".into(), json!("24px")); return Some(p); }
        "text-lg" => { let mut p = CssProps::new(); p.insert("font-size".into(), json!("18px")); p.insert("line-height".into(), json!("28px")); return Some(p); }
        "text-xl" => { let mut p = CssProps::new(); p.insert("font-size".into(), json!("20px")); p.insert("line-height".into(), json!("28px")); return Some(p); }
        "text-2xl" => { let mut p = CssProps::new(); p.insert("font-size".into(), json!("24px")); p.insert("line-height".into(), json!("32px")); return Some(p); }
        "text-3xl" => { let mut p = CssProps::new(); p.insert("font-size".into(), json!("30px")); p.insert("line-height".into(), json!("36px")); return Some(p); }
        "text-4xl" => { let mut p = CssProps::new(); p.insert("font-size".into(), json!("36px")); p.insert("line-height".into(), json!("40px")); return Some(p); }
        "text-5xl" => { let mut p = CssProps::new(); p.insert("font-size".into(), json!("48px")); p.insert("line-height".into(), json!("1")); return Some(p); }
        "text-6xl" => { let mut p = CssProps::new(); p.insert("font-size".into(), json!("60px")); p.insert("line-height".into(), json!("1")); return Some(p); }
        _ => {}
    }
    // Text alignment
    match class {
        "text-left" => { let mut p = CssProps::new(); p.insert("text-align".into(), json!("left")); return Some(p); }
        "text-center" => { let mut p = CssProps::new(); p.insert("text-align".into(), json!("center")); return Some(p); }
        "text-right" => { let mut p = CssProps::new(); p.insert("text-align".into(), json!("right")); return Some(p); }
        "text-justify" => { let mut p = CssProps::new(); p.insert("text-align".into(), json!("justify")); return Some(p); }
        _ => {}
    }
    // Overflow utilities
    match class {
        "overflow-auto" => { let mut p = CssProps::new(); p.insert("overflow".into(), json!("auto")); return Some(p); }
        "overflow-hidden" => { let mut p = CssProps::new(); p.insert("overflow".into(), json!("hidden")); return Some(p); }
        "overflow-visible" => { let mut p = CssProps::new(); p.insert("overflow".into(), json!("visible")); return Some(p); }
        "overflow-scroll" => { let mut p = CssProps::new(); p.insert("overflow".into(), json!("scroll")); return Some(p); }
        "overflow-x-auto" => { let mut p = CssProps::new(); p.insert("overflow-x".into(), json!("auto")); return Some(p); }
        "overflow-x-hidden" => { let mut p = CssProps::new(); p.insert("overflow-x".into(), json!("hidden")); return Some(p); }
        "overflow-x-scroll" => { let mut p = CssProps::new(); p.insert("overflow-x".into(), json!("scroll")); return Some(p); }
        "overflow-y-auto" => { let mut p = CssProps::new(); p.insert("overflow-y".into(), json!("auto")); return Some(p); }
        "overflow-y-hidden" => { let mut p = CssProps::new(); p.insert("overflow-y".into(), json!("hidden")); return Some(p); }
        "overflow-y-scroll" => { let mut p = CssProps::new(); p.insert("overflow-y".into(), json!("scroll")); return Some(p); }
        _ => {}
    }
    // Opacity utilities
    if let Some(value) = class.strip_prefix("opacity-") {
        if let Ok(opacity) = value.parse::<f32>() {
            let mut p = CssProps::new();
            p.insert("opacity".into(), json!(opacity / 100.0));
            return Some(p);
        }
    }
    // Shadow utilities (basic cross-platform support)
    match class {
        "shadow-sm" => { let mut p = CssProps::new(); p.insert("box-shadow".into(), json!("0 1px 2px 0 rgba(0, 0, 0, 0.05)")); return Some(p); }
        "shadow" => { let mut p = CssProps::new(); p.insert("box-shadow".into(), json!("0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px -1px rgba(0, 0, 0, 0.1)")); return Some(p); }
        "shadow-md" => { let mut p = CssProps::new(); p.insert("box-shadow".into(), json!("0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1)")); return Some(p); }
        "shadow-lg" => { let mut p = CssProps::new(); p.insert("box-shadow".into(), json!("0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.1)")); return Some(p); }
        "shadow-xl" => { let mut p = CssProps::new(); p.insert("box-shadow".into(), json!("0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1)")); return Some(p); }
        "shadow-2xl" => { let mut p = CssProps::new(); p.insert("box-shadow".into(), json!("0 25px 50px -12px rgba(0, 0, 0, 0.25)")); return Some(p); }
        "shadow-none" => { let mut p = CssProps::new(); p.insert("box-shadow".into(), json!("none")); return Some(p); }
        _ => {}
    }
    // Parse arbitrary values like bg-[var(--primary)], text-[#ff0000], etc.
    if let Some(arb_value) = parse_arbitrary_value(class) {
        return Some(arb_value);
    }
    // text-{color}-{shade}
    if let Some(rest) = class.strip_prefix("text-") {
        if let Some(hex) = get_tailwind_color_with_vars(rest, vars) {
            let mut props = CssProps::new();
            props.insert("color".into(), json!(hex));
            return Some(props);
        }
    }
    // bg-{color}-{shade}
    if let Some(rest) = class.strip_prefix("bg-") {
        match rest {
            "white" => { let mut p = CssProps::new(); p.insert("background-color".into(), json!("#ffffff")); return Some(p); }
            "black" => { let mut p = CssProps::new(); p.insert("background-color".into(), json!("#000000")); return Some(p); }
            "transparent" => { let mut p = CssProps::new(); p.insert("background-color".into(), json!("#00000000")); return Some(p); }
            _ => {}
        }
        if let Some(hex) = get_tailwind_color_with_vars(rest, vars) {
            let mut props = CssProps::new();
            props.insert("background-color".into(), json!(hex));
            return Some(props);
        }
    }
    // divide-{color}-{shade} (sets border-color for child dividers)
    if let Some(rest) = class.strip_prefix("divide-") {
        if let Some(hex) = get_tailwind_color_with_vars(rest, vars) {
            let mut props = CssProps::new();
            props.insert("border-color".into(), json!(hex));
            return Some(props);
        }
    }
    if class == "border" {
        return Some(border_props(None, 1, vars));
    }
    if let Some(rest) = class.strip_prefix("border-") {
        // Parse border-* classes
        // Possible patterns:
        // - border-{color}-{shade} → border-color
        // - border-{side}-{color}-{shade} → border-{side}-color
        // - border-{width} → border-width
        // - border-{side}-{width} → border-{side}-width
        
        let parts: Vec<&str> = rest.split('-').collect();
        
        // Check if first part is a directional side (t, b, l, r, x, y)
        let valid_sides = ["t", "b", "l", "r", "x", "y"];
        let (side, color_or_width_parts) = if parts.len() > 1 && valid_sides.contains(&parts[0]) {
            (Some(parts[0]), &parts[1..])
        } else {
            (None, &parts[..])
        };
        
        // Now check if remaining parts form a color-shade pattern
        if color_or_width_parts.len() == 2 {
            // Could be color-shade like "blue-500"
            let color_shade = color_or_width_parts.join("-");
            if let Some(hex) = get_tailwind_color_with_vars(&color_shade, vars) {
                let mut props = CssProps::new();
                let prop_name = if let Some(s) = side {
                    format!("border-{}-color", s)
                } else {
                    "border-color".to_string()
                };
                props.insert(prop_name, json!(hex));
                return Some(props);
            }
        }
        
        // Check for simple color without shade (single word color like "black", "white")
        if color_or_width_parts.len() == 1 {
            let potential_color = format!("{}-500", color_or_width_parts[0]);
            if let Some(hex) = get_tailwind_color_with_vars(&potential_color, vars) {
                let mut props = CssProps::new();
                let prop_name = if let Some(s) = side {
                    format!("border-{}-color", s)
                } else {
                    "border-color".to_string()
                };
                props.insert(prop_name, json!(hex));
                return Some(props);
            }
        }
        
        // Otherwise, check for width (e.g., border-2, border-t-4)
        if color_or_width_parts.len() == 1 {
            if let Ok(width) = color_or_width_parts[0].parse::<i32>() {
                return Some(border_props(side, width, vars));
            }
        }
    }
    // rounded* (border-radius)
    if class == "rounded" { return Some(rounded_props(None, Some("md"))); }
    if let Some(sz) = class.strip_prefix("rounded-") {
        return Some(rounded_props(None, Some(sz)));
    }
    for &(pref, side) in &[("rounded-t", "t"), ("rounded-b", "b"), ("rounded-l", "l"), ("rounded-r", "r")] {
        if class == pref { return Some(rounded_props(Some(side), Some("md"))); }
        if let Some(sz) = class.strip_prefix(&(pref.to_string() + "-")) {
            return Some(rounded_props(Some(side), Some(sz)));
        }
    }
    // cursor-*
    if let Some(cur) = class.strip_prefix("cursor-") {
        let mut props = CssProps::new();
        props.insert("cursor".into(), json!(match cur {
            "pointer" => "pointer",
            "default" => "default",
            "text" => "text",
            "move" => "move",
            "wait" => "wait",
            "not-allowed" => "not-allowed",
            other => other,
        }));
        return Some(props);
    }
    // transition*
    if class == "transition" || class == "transition-all" {
        let mut props = CssProps::new();
        props.insert("transition-property".into(), json!("all"));
        props.insert("transition-duration".into(), json!("150ms"));
        props.insert("transition-timing-function".into(), json!("ease-in-out"));
        return Some(props);
    }
    if class == "transition-none" {
        let mut props = CssProps::new();
        props.insert("transition-property".into(), json!("none"));
        props.insert("transition-duration".into(), json!("0ms"));
        return Some(props);
    }
    if let Some(rest) = class.strip_prefix("transition-") {
        // e.g., transition-colors → limit property; keep default duration/ease
        let mut props = CssProps::new();
        let property = match rest {
            "colors" => "color, background-color, border-color, fill, stroke",
            "opacity" => "opacity",
            "transform" => "transform",
            "shadow" => "box-shadow",
            other => other,
        };
        props.insert("transition-property".into(), json!(property));
        props.insert("transition-duration".into(), json!("150ms"));
        props.insert("transition-timing-function".into(), json!("ease-in-out"));
        return Some(props);
    }
    // width utilities: w-*, w-full, w-screen, w-min, w-max (treat min/max as auto), w-px
    if let Some(val) = class.strip_prefix("w-") {
        return width_like_props("width", val);
    }
    if let Some(val) = class.strip_prefix("min-w-") {
        return width_like_props("min-width", val);
    }
    if let Some(val) = class.strip_prefix("max-w-") {
        return width_like_props("max-width", val);
    }
    // Height utilities
    if let Some(val) = class.strip_prefix("h-") {
        return width_like_props("height", val);
    }
    if let Some(val) = class.strip_prefix("min-h-") {
        return width_like_props("min-height", val);
    }
    if let Some(val) = class.strip_prefix("max-h-") {
        return width_like_props("max-height", val);
    }
    None
}

fn parse_tailwind_spacing<F>(value: &str, builder: &F) -> Option<CssProps>
where
    F: Fn(i32) -> CssProps,
{
    if let Ok(n) = value.parse::<i32>() {
        let px = n * 4;
        return Some(builder(px));
    }
    None
}

fn padding_props(keys: &[&str], px_value: i32) -> CssProps {
    let mut props = CssProps::new();
    let val = format!("{}px", px_value);
    for key in keys {
        props.insert((*key).into(), json!(&val));
    }
    props
}

fn margin_props(keys: &[&str], px_value: i32) -> CssProps {
    let mut props = CssProps::new();
    let val = format!("{}px", px_value);
    for key in keys {
        props.insert((*key).into(), json!(&val));
    }
    props
}

fn border_props(side: Option<&str>, width: i32, _vars: &IndexMap<String, String>) -> CssProps {
    let mut props = CssProps::new();
    let width_str = format!("{}px", width);
    match side {
        None => {
            props.insert("border-width".into(), json!(&width_str));
        }
        Some("t") => {
            props.insert("border-top-width".into(), json!(&width_str));
        }
        Some("b") => {
            props.insert("border-bottom-width".into(), json!(&width_str));
        }
        Some("l") => {
            props.insert("border-left-width".into(), json!(&width_str));
        }
        Some("r") => {
            props.insert("border-right-width".into(), json!(&width_str));
        }
        Some("x") => {
            props.insert("border-left-width".into(), json!(&width_str));
            props.insert("border-right-width".into(), json!(&width_str));
        }
        Some("y") => {
            props.insert("border-top-width".into(), json!(&width_str));
            props.insert("border-bottom-width".into(), json!(&width_str));
        }
        _ => {
            props.insert("border-width".into(), json!(&width_str));
        }
    };
    props.insert("border-color".into(), json!("var(border)"));
    props.insert("border-style".into(), json!("solid"));
    props
}

fn rounded_props(side: Option<&str>, size: Option<&str>) -> CssProps {
    let mut props = CssProps::new();
    let px = match size.unwrap_or("md") {
        "none" => 0,
        "sm" => 2,
        "md" => 4,
        "lg" => 8,
        "xl" => 12,
        "2xl" => 16,
        "3xl" => 24,
        "full" => 9999,
        s => s.parse::<i32>().unwrap_or(4),
    };
    let v = json!(format!("{}px", px));
    match side {
        None => { props.insert("border-radius".into(), v); }
        Some("t") => {
            props.insert("border-top-left-radius".into(), v.clone());
            props.insert("border-top-right-radius".into(), v);
        }
        Some("b") => {
            props.insert("border-bottom-left-radius".into(), v.clone());
            props.insert("border-bottom-right-radius".into(), v);
        }
        Some("l") => { props.insert("border-top-left-radius".into(), v.clone()); props.insert("border-bottom-left-radius".into(), v); }
        Some("r") => { props.insert("border-top-right-radius".into(), v.clone()); props.insert("border-bottom-right-radius".into(), v); }
        _ => { props.insert("border-radius".into(), v); }
    }
    props
}

fn width_like_props(prop: &str, token: &str) -> Option<CssProps> {
    let mut props = CssProps::new();
    let value = match token {
        "full" => Some("100%".to_string()),
        "screen" => Some(if prop == "width" { "100vw" } else { "100vh" }.to_string()),
        "min" => Some("min-content".to_string()),
        "max" => Some("max-content".to_string()),
        "fit" => Some("fit-content".to_string()),
        "auto" => Some("auto".to_string()),
        "px" => Some("1px".to_string()),
        other => {
            // numeric scale n => n*4px, fraction e.g., 1/2 => 50%
            if let Some((a, b)) = other.split_once('/') {
                if let (Ok(na), Ok(nb)) = (a.parse::<f64>(), b.parse::<f64>()) {
                    let pct = (na / nb) * 100.0;
                    Some(format!("{}%", trim_trailing_zeros(pct)))
                } else { None }
            } else if let Ok(n) = other.parse::<i32>() {
                Some(format!("{}px", n * 4))
            } else {
                None
            }
        }
    }?;
    props.insert(prop.into(), json!(value));
    Some(props)
}

fn trim_trailing_zeros(num: f64) -> String {
    let mut s = format!("{:.6}", num);
    while s.contains('.') && s.ends_with('0') { s.pop(); }
    if s.ends_with('.') { s.pop(); }
    s
}

// ---------------- Tailwind subset ----------------

// static RE_NUM: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?P<prefix>(hover:)?(xs:|sm:|md:|lg:|xl:)*)?(?P<base>.+)$").unwrap());

fn css_escape_class(class: &str) -> String { class.replace(':', "\\:") }

fn class_to_selector(class: &str) -> String {
    let (_bp, hover, base) = parse_prefixed_class(class);
    if hover {
        format!(".{}:hover", css_escape_class(&base))
    } else {
        format!(".{}", css_escape_class(&base))
    }
}

// ------------- helpers for CSS output of media selectors -------------

/// Flatten CSS with potential selectors that include media prelude.
/// This simple post-processor merges entries that use the special selector format
/// "@media (min-width: X) {<sel>" where we will close the block at the end.
/// We group by media and inside concatenate selectors.
pub fn post_process_css(
    raw_rules: &[(String, CssProps)],
    vars: &IndexMap<String, String>,
) -> String {
    // Group into normal rules and media rules
    let mut normal = vec![];
    let mut media_map: IndexMap<String, Vec<(String, CssProps)>> = IndexMap::new();
    for (sel, props) in raw_rules.iter() {
        if let Some((media, inner)) = sel.split_once('{') {
            if media.trim_start().starts_with("@media ") && inner.ends_with('}') {
                let inner_sel = inner.trim_end_matches('}').to_string();
                media_map
                    .entry(media.trim().to_string())
                    .or_default()
                    .push((inner_sel, props.clone()));
                continue;
            }
        }
        normal.push((sel.clone(), props.clone()));
    }
    let mut out = String::new();
    for (sel, props) in normal {
        out.push_str(&sel);
        out.push('{');
        out.push_str(&css_props_string(&props, vars));
        out.push_str("}\n");
    }
    for (media, entries) in media_map {
        out.push_str(&media);
        out.push('{');
        for (sel, props) in entries {
            out.push_str(&sel);
            out.push('{');
            out.push_str(&css_props_string(&props, vars));
            out.push_str("}");
        }
        out.push_str("}\n");
    }
    out
}

// -------- Prefix parsing (hover:, breakpoint:) --------

fn parse_prefixed_class(class: &str) -> (Option<String>, bool, String) {
    // Split by ':' to find prefixes like md:hover:block
    let parts: Vec<&str> = class.split(':').collect();
    if parts.len() == 1 {
        return (None, false, class.to_string());
    }
    let mut bp: Option<String> = None;
    let mut hover = false;
    for &p in &parts[..parts.len() - 1] {
        match p {
            "hover" => hover = true,
            "xs" | "sm" | "md" | "lg" | "xl" => bp = Some(p.to_string()),
            _ => {}
        }
    }
    let base = parts.last().unwrap().to_string();
    (bp, hover, base)
}

fn wrap_with_media(selector: &str, bp_key: Option<&str>, bps: &IndexMap<String, String>) -> String {
    if let Some(k) = bp_key {
        if let Some(val) = bps.get(k) {
            return format!("@media (min-width: {}) {{{}}}", val, selector);
        }
    }
    selector.to_string()
}

/// Get a Tailwind color hex value from a string like "slate-200" or "blue-500"
fn get_tailwind_color(color_shade: &str) -> Option<String> {
    let parts: Vec<&str> = color_shade.split('-').collect();
    if parts.len() != 2 {
        return None;
    }
    let color_name = parts[0];
    let shade = parts[1];
    
    // First try standard Tailwind colors
    if let Some(hex) = TAILWIND_COLORS
        .get(color_name)
        .and_then(|shades| shades.get(shade))
    {
        return Some(hex.to_string());
    }
    
    None
}

fn get_tailwind_color_with_vars(color_shade: &str, vars: &IndexMap<String, String>) -> Option<String> {
    // First try standard Tailwind colors
    if let Some(hex) = get_tailwind_color(color_shade) {
        return Some(hex);
    }
    
    // If not found, check if color_shade matches a variable
    // Theme variables are flattened with "." separators, e.g., "colors.primary"
    // So we need to check:
    // 1. Direct match: "primary" → look for "primary" in vars
    // 2. Color namespace: "primary" → look for "colors.primary" in vars (plural)
    // 3. Color namespace: "primary" → look for "color.primary" in vars (singular)
    // 4. With shade: "primary-500" → look for "colors.primary" or "colors.primary-500" in vars
    
    if let Some(val) = vars.get(color_shade) {
        return Some(val.clone());
    }
    
    // Try with "colors." namespace prefix (plural - HookRenderer uses this)
    if let Some(val) = vars.get(&format!("colors.{}", color_shade)) {
        return Some(val.clone());
    }
    
    // Try with "color." namespace prefix (singular - fallback)
    if let Some(val) = vars.get(&format!("color.{}", color_shade)) {
        return Some(val.clone());
    }
    
    // Handle cases where the color name doesn't have a shade but we need to look for a variable
    // e.g., "primary" (from bg-primary) → look for "color.primary"
    let parts: Vec<&str> = color_shade.split('-').collect();
    if parts.len() >= 1 {
        let color_name = parts[0];
        
        // Try direct variable
        if let Some(val) = vars.get(color_name) {
            return Some(val.clone());
        }
        
        // Try with "color." namespace
        if let Some(val) = vars.get(&format!("color.{}", color_name)) {
            return Some(val.clone());
        }
    }
    
    None
}

/// Parse arbitrary values like bg-[var(--primary)], text-[#ff0000], border-[hsl(200,50%,50%)]
fn parse_arbitrary_value(class: &str) -> Option<CssProps> {
    // Match pattern: prefix-[value]
    if let Some(bracket_start) = class.find('[') {
        if !class.ends_with(']') {
            return None;
        }
        let prefix = &class[..bracket_start];
        let value = &class[bracket_start + 1..class.len() - 1];
        
        let mut props = CssProps::new();
        match prefix {
            "bg" => {
                props.insert("background-color".into(), json!(value));
                return Some(props);
            }
            "text" => {
                props.insert("color".into(), json!(value));
                return Some(props);
            }
            "border" => {
                props.insert("border-color".into(), json!(value));
                return Some(props);
            }
            "divide" => {
                props.insert("border-color".into(), json!(value));
                return Some(props);
            }
            _ => return None,
        }
    }
    None
}

// re-export minimal API for CLI
pub mod api {
    pub use super::{SelectorStyles, State};
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Add a theme with button styles
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
    fn embedded_defaults_and_version() {
        // Test that we can create a state and add a theme with variables
        let mut st = State::default_state();
        st.add_theme("default", IndexMap::new());
        st.set_theme("default").ok();
        
        let mut vars = IndexMap::new();
        vars.insert("primary".to_string(), "#007bff".to_string());
        st.set_variables(vars);
        
        assert!(st.themes.contains_key("default"));
        let def = st.themes.get("default").unwrap();
        assert!(def.variables.contains_key("primary"));

        // Version should compile and be non-empty (env! evaluated at compile-time)
        // Note: get_version() is only available for wasm32 target
        #[cfg(target_arch = "wasm32")]
        {
            let v = get_version();
            assert!(!v.is_empty());
        }
    }

    #[test]
    fn border_color_with_direction() {
        let mut st = State::new_default();
        
        // Test border-b-blue-500 (border-bottom with blue color shade 500)
        st.register_tailwind_classes(["border-b-blue-500".to_string()]);
        let css = st.css_for_web();
        assert!(css.contains(".border-b-blue-500{"));
        assert!(css.contains("border-bottom-color:#3b82f6") || css.contains("border-b-color:#3b82f6"));
        
        // Test border-t-red-500
        st.register_tailwind_classes(["border-t-red-500".to_string()]);
        let css = st.css_for_web();
        assert!(css.contains(".border-t-red-500{"));
        
        // Test border-blue-500 (all borders)
        st.register_tailwind_classes(["border-blue-500".to_string()]);
        let css = st.css_for_web();
        assert!(css.contains(".border-blue-500{"));
        assert!(css.contains("border-color:#3b82f6"));
    }

    #[test]
    fn multiple_selectors_support() {
        let mut st = State::new_default();
        let mut selectors = SelectorStyles::new();
        let mut props = CssProps::new();
        props.insert("color".to_string(), serde_json::json!("#ff0000"));
        selectors.insert("h1, h2, h3".to_string(), props);
        
        st.add_theme("test", selectors);
        st.set_theme("test").ok();
        
        // Test h1
        let android = st.android_styles_for("h1", &[]);
        assert_eq!(android.get("color").and_then(|v| v.as_str()), Some("#ff0000"), "h1 should have red color");
        
        // Test h2
        let android = st.android_styles_for("h2", &[]);
        assert_eq!(android.get("color").and_then(|v| v.as_str()), Some("#ff0000"), "h2 should have red color");
        
        // Test h3
        let android = st.android_styles_for("h3", &[]);
        assert_eq!(android.get("color").and_then(|v| v.as_str()), Some("#ff0000"), "h3 should have red color");
    }

    #[test]
    fn multiple_selectors_classes() {
        let mut st = State::new_default();
        let mut selectors = SelectorStyles::new();
        let mut props = CssProps::new();
        props.insert("padding".to_string(), serde_json::json!("10px"));
        selectors.insert(".btn, .link".to_string(), props);
        
        st.add_theme("test", selectors);
        st.set_theme("test").ok();
        
        // Test .btn
        let android = st.android_styles_for("div", &["btn".to_string()]);
        assert_eq!(android.get("padding").and_then(|v| v.as_f64()), Some(10.0), ".btn should have 10px padding");
        
        // Test .link
        let android = st.android_styles_for("div", &["link".to_string()]);
        assert_eq!(android.get("padding").and_then(|v| v.as_f64()), Some(10.0), ".link should have 10px padding");
    }

    #[test]
    fn border_width_with_direction() {
        let mut st = State::new_default();
        
        // Test border-b-2 (border-bottom width 2px)
        st.register_tailwind_classes(["border-b-2".to_string()]);
        let css = st.css_for_web();
        assert!(css.contains(".border-b-2{"));
        assert!(css.contains("border-bottom-width:2px"));
        
        // Test border-2 (all borders width 2px)
        st.register_tailwind_classes(["border-2".to_string()]);
        let css = st.css_for_web();
        assert!(css.contains(".border-2{"));
        assert!(css.contains("border-width:2px"));
    }

    #[test]
    fn display_flex_hover_breakpoint() {
        let mut st = State::new_default();
        
        // Set up theme with breakpoints
        st.add_theme("default", IndexMap::new());
        st.set_theme("default").ok();
        
        let mut breakpoints = IndexMap::new();
        breakpoints.insert("md".to_string(), "768px".to_string());
        st.set_breakpoints(breakpoints);
        
        st.register_tailwind_classes([
            "block".into(),
            "inline-flex".into(),
            "hidden".into(),
            "md:flex".into(),
            "md:hover:block".into(),
        ]);
        let css = st.css_for_web();
        assert!(css.contains(".block{"));
        assert!(css.contains("display:block"));
        assert!(css.contains(".inline-flex{"));
        assert!(css.contains("display:inline-flex"));
        assert!(css.contains(".hidden{"));
        assert!(css.contains("display:none"));
        // breakpoint rule
        assert!(css.contains("@media (min-width: 768px)"));
        assert!(css.contains(".flex{display:flex"));
        // hover inside media (substring check)
        assert!(css.contains(":hover{display:block"));

        // Android resolves base class styles ignoring prefixes
        let android = st.android_styles_for("div", &["md:flex".into()]);
        assert_eq!(android.get("display").and_then(|v| v.as_str()), Some("flex"));
    }

    #[test]
    fn parse_var_references_basic() {
        // Test basic var() parsing
        let refs = parse_var_references("var(color)");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].2, "color");
        assert_eq!(refs[0].0, 0); // start
        assert_eq!(refs[0].1, 10); // end (exclusive, so "var(color)" is 0..10)

        // Test var() with -- prefix
        let refs = parse_var_references("var(--primary)");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].2, "primary");

        // Test multiple var() references
        let refs = parse_var_references("var(--color) and var(size)");
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].2, "color");
        assert_eq!(refs[1].2, "size");

        // Test with whitespace
        let refs = parse_var_references("var( --spacing )");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].2, "spacing");

        // Test with dots and dashes
        let refs = parse_var_references("var(color.primary-500)");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].2, "color.primary-500");

        // Test no matches
        let refs = parse_var_references("no variables here");
        assert_eq!(refs.len(), 0);

        // Test incomplete var(
        let refs = parse_var_references("var(");
        assert_eq!(refs.len(), 0);

        // Test var without closing
        let refs = parse_var_references("var(color");
        assert_eq!(refs.len(), 0);
    }

    #[test]
    fn resolve_vars_basic() {
        let mut vars = IndexMap::new();
        vars.insert("primary".to_string(), "#ff0000".to_string());
        vars.insert("spacing".to_string(), "8px".to_string());
        vars.insert("color.blue".to_string(), "#0000ff".to_string());

        // Test basic resolution
        assert_eq!(resolve_vars("var(--primary)", &vars), "#ff0000");
        assert_eq!(resolve_vars("var(primary)", &vars), "#ff0000");
        assert_eq!(resolve_vars("var( --primary )", &vars), "#ff0000");

        // Test multiple vars
        assert_eq!(
            resolve_vars("var(--primary) var(--spacing)", &vars),
            "#ff0000 8px"
        );

        // Test dotted variable names
        assert_eq!(resolve_vars("var(--color.blue)", &vars), "#0000ff");

        // Test undefined variable (should not replace)
        assert_eq!(resolve_vars("var(--undefined)", &vars), "var(--undefined)");

        // Test $ prefix syntax
        assert_eq!(resolve_vars("$primary", &vars), "#ff0000");

        // Test no variables
        assert_eq!(resolve_vars("plain text", &vars), "plain text");
    }

    #[test]
    fn resolve_vars_edge_cases() {
        let mut vars = IndexMap::new();
        vars.insert("a".to_string(), "1".to_string());
        vars.insert("b".to_string(), "2".to_string());

        // Test adjacent vars
        assert_eq!(resolve_vars("var(a)var(b)", &vars), "12");

        // Test var in middle of text
        assert_eq!(resolve_vars("prefix var(a) suffix", &vars), "prefix 1 suffix");

        // Test empty input
        assert_eq!(resolve_vars("", &vars), "");

        // Test var with numbers
        vars.insert("var123".to_string(), "value".to_string());
        assert_eq!(resolve_vars("var(var123)", &vars), "value");

        // Test var with underscores
        vars.insert("my_var".to_string(), "test".to_string());
        assert_eq!(resolve_vars("var(my_var)", &vars), "test");
    }

    #[test]
    fn test_android_scrolling_mapping() {
        let mut state = State::default();
        state.display_density = 2.0;
        state.scaled_density = 2.0;
        state.current_theme = "default".to_string();
        
        let mut themes = IndexMap::new();
        let mut default_theme = crate::ThemeEntry::default();
        default_theme.name = Some("Default".to_string());
        
        let mut overflow_styles = IndexMap::new();
        overflow_styles.insert("overflowX".to_string(), serde_json::json!("auto"));
        overflow_styles.insert("overflowY".to_string(), serde_json::json!("scroll"));
        
        default_theme.selectors.insert(".scroller".to_string(), overflow_styles);
        themes.insert("default".to_string(), default_theme);
        state.themes = themes;
        
        let styles = state.android_styles_for("div", &vec![".scroller".to_string()]);
        
        assert_eq!(styles.get("androidScrollHorizontal"), Some(&serde_json::json!(true)));
        assert_eq!(styles.get("androidScrollVertical"), Some(&serde_json::json!(true)));
    }

    #[test]
    fn android_flex_row_default() {
        let st = State::new_default();
        // div with flex class should be horizontal (row) on Android
        let styles = st.android_styles_for("div", &["flex".to_string()]);
        assert_eq!(styles.get("androidOrientation").and_then(|v| v.as_str()), Some("horizontal"));
        assert_eq!(styles.get("flexDirection").and_then(|v| v.as_str()), Some("row"));
        
        // div without flex class should be vertical (column) on Android
        let styles = st.android_styles_for("div", &[]);
        assert_eq!(styles.get("androidOrientation").and_then(|v| v.as_str()), Some("vertical"));
        assert_eq!(styles.get("flexDirection").and_then(|v| v.as_str()), Some("column"));
    }

    #[test]
    fn android_gap_orientation_order() {
        let st = State::new_default();
        let styles = st.android_styles_for("div", &["flex".to_string(), "gap-4".to_string()]);
        
        // Check that androidOrientation comes BEFORE gap in the map
        let keys: Vec<&String> = styles.keys().collect();
        let orientation_idx = keys.iter().position(|&k| k == "androidOrientation").unwrap();
        let gap_idx = keys.iter().position(|&k| k == "gap").unwrap();
        
        assert!(orientation_idx < gap_idx, "androidOrientation should come before gap for correct layout processing");
    }

    #[test]
    fn margin_auto_support() {
        let mut st = State::new_default();
        st.register_tailwind_classes(["ml-auto".to_string(), "mr-auto".to_string(), "mx-auto".to_string()]);
        
        // Web CSS check
        let css = st.css_for_web();
        assert!(css.contains("margin-left:auto"));
        assert!(css.contains("margin-right:auto"));
        
        // Android check
        let styles = st.android_styles_for("div", &["ml-auto".to_string()]);
        assert_eq!(styles.get("marginLeft").and_then(|v| v.as_str()), Some("auto"));
        
        let styles = st.android_styles_for("div", &["mx-auto".to_string()]);
        assert_eq!(styles.get("marginLeft").and_then(|v| v.as_str()), Some("auto"));
        assert_eq!(styles.get("marginRight").and_then(|v| v.as_str()), Some("auto"));
    }

    #[test]
    fn alignment_mapping() {
        let st = State::new_default();
        
        // Test Row (default)
        let row_styles = st.android_styles_for("div", &["flex".to_string(), "justify-center".to_string(), "items-center".to_string()]);
        assert_eq!(row_styles.get("androidOrientation").and_then(|v| v.as_str()), Some("horizontal"));
        // Row: justify-center (horizontal) + items-center (vertical) -> center
        assert_eq!(row_styles.get("androidGravity").and_then(|v| v.as_str()), Some("center"));
        
        // Test Column
        let col_styles = st.android_styles_for("div", &["flex".to_string(), "flex-col".to_string(), "justify-center".to_string(), "items-center".to_string()]);
        assert_eq!(col_styles.get("androidOrientation").and_then(|v| v.as_str()), Some("vertical"));
        // Column: justify-center (vertical) + items-center (horizontal) -> center
        assert_eq!(col_styles.get("androidGravity").and_then(|v| v.as_str()), Some("center"));

        // Test Row Start/End
        let row_start_styles = st.android_styles_for("div", &["flex".to_string(), "justify-start".to_string(), "items-end".to_string()]);
        assert_eq!(row_start_styles.get("androidGravity").and_then(|v| v.as_str()), Some("bottom|start"));
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
        
        // Test button with bg-bg class and p-4
        let classes = vec!["bg-bg".to_string(), "p-4".to_string()];
        let styles = state.android_styles_for("button", &classes);
        
        println!("[test_button_bg_override] styles: {:?}", styles);
        
        // Should have white background from bg-bg, not blue from button selector
        assert_eq!(styles.get("backgroundColor").and_then(|v: &serde_json::Value| v.as_str()), Some("#ffffff"));
        
        // Should have 16px padding from p-4, not 8px from button selector
        // p-4 = 1rem = 16px (default)
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

    #[test]
    fn test_css_kebab_case_conversion() {
        let mut themes = IndexMap::new();
        let mut selectors = IndexMap::new();
        
        let mut props = IndexMap::new();
        // Use camelCase property which should be converted to kebab-case for web
        props.insert("backgroundColor".to_string(), json!("#ffffff"));
        props.insert("borderTopWidth".to_string(), json!(1));
        selectors.insert("body".to_string(), props);
        
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
        
        // Mark body as used so it's emitted
        state.used_tags.insert("body".to_string());
        
        let css = state.css_for_web();
        println!("[test_css_kebab_case_conversion] css: {}", css);
        
        assert!(css.contains("background-color:#ffffff;"));
        assert!(css.contains("border-top-width:1;"));
        assert!(!css.contains("backgroundColor:"));
        assert!(!css.contains("borderTopWidth:"));
    }
}

#[cfg(all(target_os = "android", feature = "android"))]
#[cfg(feature = "android")]
mod android_jni;

mod bridge_common;
mod utils;
mod ffi;

pub use ffi::*;

#[cfg(target_vendor = "apple")]
mod ios_ffi;
