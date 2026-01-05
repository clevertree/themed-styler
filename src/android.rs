use indexmap::IndexMap;
use serde_json::json;
use crate::theme::State;
use crate::utils::{merge_android_props, parse_prefixed_class, class_to_selector};
use crate::tailwind::dynamic_css_properties_for_class;
use crate::units::{dp_to_px, parse_and_convert_to_px};

impl State {
    pub fn android_base_styles(&self, selector: &str, classes: &[String]) -> IndexMap<String, serde_json::Value> {
        let (eff, vars) = self.effective_theme_all();
        let mut out: IndexMap<String, serde_json::Value> = IndexMap::new();

        out.insert("androidOrientation".to_string(), serde_json::json!("vertical"));

        let mut defaults = crate::CssProps::new();
        match selector.to_lowercase().as_str() {
            "div" => { defaults.insert("width".into(), json!("match_parent")); }
            "p" => {
                defaults.insert("width".into(), json!("match_parent"));
                defaults.insert("margin-vertical".into(), json!("16px"));
            }
            "h1" => {
                defaults.insert("width".into(), json!("match_parent"));
                defaults.insert("font-size".into(), json!("32px"));
                defaults.insert("font-weight".into(), json!("bold"));
                defaults.insert("margin-vertical".into(), json!("21.44px"));
            }
            "h2" => {
                defaults.insert("width".into(), json!("match_parent"));
                defaults.insert("font-size".into(), json!("24px"));
                defaults.insert("font-weight".into(), json!("bold"));
                defaults.insert("margin-vertical".into(), json!("19.92px"));
            }
            "h3" => {
                defaults.insert("width".into(), json!("match_parent"));
                defaults.insert("font-size".into(), json!("18.72px"));
                defaults.insert("font-weight".into(), json!("bold"));
                defaults.insert("margin-vertical".into(), json!("18.72px"));
            }
            "h4" => {
                defaults.insert("width".into(), json!("match_parent"));
                defaults.insert("font-size".into(), json!("16px"));
                defaults.insert("font-weight".into(), json!("bold"));
                defaults.insert("margin-vertical".into(), json!("21.28px"));
            }
            "h5" => {
                defaults.insert("width".into(), json!("match_parent"));
                defaults.insert("font-size".into(), json!("13.28px"));
                defaults.insert("font-weight".into(), json!("bold"));
                defaults.insert("margin-vertical".into(), json!("22.17px"));
            }
            "h6" => {
                defaults.insert("width".into(), json!("match_parent"));
                defaults.insert("font-size".into(), json!("10.72px"));
                defaults.insert("font-weight".into(), json!("bold"));
                defaults.insert("margin-vertical".into(), json!("24.96px"));
            }
            "input" => {
                defaults.insert("padding-vertical".into(), json!("8px"));
                defaults.insert("padding-horizontal".into(), json!("12px"));
                defaults.insert("border-radius".into(), json!("4px"));
                defaults.insert("border-width".into(), json!("1px"));
                defaults.insert("border-color".into(), json!("#cccccc"));
                defaults.insert("background-color".into(), json!("#ffffff"));
                defaults.insert("min-height".into(), json!("40px"));
                defaults.insert("android-gravity".into(), json!("center_vertical"));
            }
            "button" => {
                defaults.insert("padding-vertical".into(), json!("8px"));
                defaults.insert("padding-horizontal".into(), json!("16px"));
                defaults.insert("border-radius".into(), json!("4px"));
                defaults.insert("background-color".into(), json!("#2196F3"));
                defaults.insert("color".into(), json!("#ffffff"));
                defaults.insert("android-gravity".into(), json!("center"));
            }
            _ => {}
        }
        merge_android_props(&mut out, &defaults, &vars);

        if let Some(props) = eff.get(selector) {
            merge_android_props(&mut out, props, &vars);
        }

        for class in classes {
            let normalized_class = if class.starts_with('.') {
                class[1..].to_string()
            } else {
                class.clone()
            };
            
            let (_bp, _hover, base) = parse_prefixed_class(&normalized_class);
            let sel = class_to_selector(&base);
            if let Some(props) = eff.get(&sel) {
                merge_android_props(&mut out, props, &vars);
                continue;
            }
            if let Some(dynamic_props) = dynamic_css_properties_for_class(&base, &vars) {
                merge_android_props(&mut out, &dynamic_props, &vars);
                continue;
            }
            if let Some(props) = eff.get(&base) {
                merge_android_props(&mut out, props, &vars);
            }
        }
        
        if let Some(display) = out.get("display") {
            if display.as_str() == Some("flex") && !out.contains_key("flexDirection") {
                out.insert("flexDirection".to_string(), serde_json::json!("row"));
                out.insert("androidOrientation".to_string(), serde_json::json!("horizontal"));
            }
        }

        if !out.contains_key("flexDirection") {
            match selector.to_lowercase().as_str() {
                "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" => {
                    out.insert("flexDirection".to_string(), serde_json::json!("column"));
                    out.insert("androidOrientation".to_string(), serde_json::json!("vertical"));
                }
                _ => {}
            }
        }
        
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

    pub fn android_styles_for(&self, selector: &str, classes: &[String]) -> IndexMap<String, serde_json::Value> {
        let mut styles = self.android_base_styles(selector, classes);
        
        let density = self.display_density;
        let scaled_density = self.scaled_density;

        if let Some(flex_dir) = styles.get("flexDirection") {
            let orientation = if flex_dir.as_str() == Some("row") { "horizontal" } else { "vertical" };
            styles.shift_insert(0, "androidOrientation".to_string(), serde_json::json!(orientation));
        }
        
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
        
        if let Some(font_size) = styles.get("fontSize").cloned() {
            if let Some(serde_json::Value::Number(n)) = parse_and_convert_to_px(&font_size, density).as_ref() {
                let sp_value = n.as_f64().unwrap_or(14.0) as f32 / density * scaled_density;
                styles.insert("fontSize".to_string(), serde_json::json!(sp_value));
            }
        }
        
        if let Some(flex_wrap) = styles.get("flexWrap") {
            if flex_wrap.as_str() == Some("wrap") {
                styles.insert("androidFlexWrap".to_string(), serde_json::json!(true));
            }
        }

        if let Some(opacity) = styles.get("opacity").cloned() {
            styles.insert("androidAlpha".to_string(), opacity);
        }

        let is_horizontal = styles.get("androidOrientation").and_then(|v| v.as_str()) == Some("horizontal");
        let mut gravity_parts = Vec::new();

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

        if let Some(serde_json::Value::String(shadow)) = styles.get("boxShadow").cloned() {
            if !shadow.is_empty() {
                let elevation = if shadow.contains("20px") { 24 }
                               else if shadow.contains("15px") { 16 }
                               else if shadow.contains("10px") { 8 }
                               else { 4 };
                styles.insert("elevation".to_string(), serde_json::json!(dp_to_px(elevation as f32, density)));
            }
        }

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

        if styles.contains_key("flex") || styles.contains_key("flexGrow") {
            if !styles.contains_key("width") {
                styles.insert("width".to_string(), serde_json::json!("wrap_content"));
            }
            if !styles.contains_key("height") {
                styles.insert("height".to_string(), serde_json::json!("wrap_content"));
            }
        }
        
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
}
