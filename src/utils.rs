use indexmap::IndexMap;
use serde_json::json;
use crate::CssProps;

pub fn parse_var_references(input: &str) -> Vec<(usize, usize, String)> {
    let mut results = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 4 <= bytes.len() && &bytes[i..i+4] == b"var(" {
            let start = i;
            i += 4;
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\n' || bytes[i] == b'\r') {
                i += 1;
            }
            let has_prefix = i + 2 <= bytes.len() && &bytes[i..i+2] == b"--";
            if has_prefix { i += 2; }
            let name_start = i;
            while i < bytes.len() {
                let c = bytes[i];
                if (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || 
                   (c >= b'0' && c <= b'9') || c == b'_' || c == b'.' || c == b'-' {
                    i += 1;
                } else { break; }
            }
            let name_end = i;
            if name_start < name_end {
                while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\n' || bytes[i] == b'\r') {
                    i += 1;
                }
                if i < bytes.len() && bytes[i] == b')' {
                    let end = i + 1;
                    let var_name = std::str::from_utf8(&bytes[name_start..name_end]).unwrap_or("").to_string();
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

pub fn resolve_vars(input: &str, vars: &IndexMap<String, String>) -> String {
    let var_refs = parse_var_references(input);
    if var_refs.is_empty() {
        if input.starts_with('$') {
            if let Some(val) = vars.get(&input[1..]) { return val.clone(); }
        }
        return input.to_string();
    }
    let mut out = input.to_string();
    for (start, end, var_name) in var_refs.iter().rev() {
        if let Some(val) = vars.get(var_name) {
            out.replace_range(*start..*end, val);
        }
    }
    if out.starts_with('$') {
        if let Some(val) = vars.get(&out[1..]) { return val.clone(); }
    }
    out
}

pub fn camel_case(name: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for ch in name.chars() {
        if ch == '-' { upper = true; continue; }
        if upper { out.extend(ch.to_uppercase()); upper = false; }
        else { out.push(ch); }
    }
    out
}

pub fn parse_prefixed_class(class: &str) -> (Option<String>, bool, String) {
    let parts: Vec<&str> = class.split(':').collect();
    if parts.len() == 1 { return (None, false, class.to_string()); }
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

pub fn css_escape_class(class: &str) -> String { class.replace(':', "\\:") }

pub fn class_to_selector(class: &str) -> String {
    let (_bp, hover, base) = parse_prefixed_class(class);
    if hover { format!(".{}:hover", css_escape_class(&base)) }
    else { format!(".{}", css_escape_class(&base)) }
}

pub fn css_value_to_android(value: &serde_json::Value, vars: &IndexMap<String, String>, current_color: Option<&str>) -> serde_json::Value {
    match value {
        serde_json::Value::String(s) => {
            let s2 = resolve_vars(s, vars);
            // We don't have access to color module here easily without circular deps if we are not careful,
            // but utils.rs is mostly used by tailwind.rs and tests.
            // For now, let's just do the basic resolution.
            if let Some(n) = s2.strip_suffix("px") {
                if let Ok(parsed) = n.trim().parse::<f64>() { return json!(parsed); }
            }
            json!(s2)
        }
        _ => value.clone(),
    }
}

pub fn merge_android_props(
    into: &mut IndexMap<String, serde_json::Value>,
    css_props: &CssProps,
    vars: &IndexMap<String, String>,
) {
    let current_color = css_props.get("color")
        .and_then(|v| v.as_str())
        .map(|s| resolve_vars(s, vars))
        .or_else(|| into.get("color").and_then(|v| v.as_str()).map(|s| s.to_string()));

    for (k, v) in css_props.iter() {
        let val = css_value_to_android(v, vars, current_color.as_deref());
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
                let orientation = if val.as_str() == Some("column") || val.as_str() == Some("column-reverse") { "vertical" } else { "horizontal" };
                into.insert("androidOrientation".to_string(), serde_json::json!(orientation));
                into.insert("flexDirection".to_string(), val);
            }
            _ => { into.insert(camel_case(k), val); }
        }
    }
}

pub fn merge_props(into: &mut CssProps, from: &CssProps) {
    for (k, v) in from.iter() {
        into.insert(k.clone(), v.clone());
    }
}

pub fn split_tag_class_key(key: &str) -> Option<(String, String)> {
    let mut it = key.splitn(2, '|');
    let t = it.next()?.to_string();
    let c = it.next()?.to_string();
    if t.is_empty() || c.is_empty() {
        return None;
    }
    Some((t, c))
}

pub fn strip_hover_suffix(selector: &str) -> (&str, bool) {
    if let Some(stripped) = selector.strip_suffix(":hover") {
        (stripped, true)
    } else {
        (selector, false)
    }
}

pub fn should_emit_selector(
    sel: &str,
    used_tags: &IndexSet<String>,
    used_classes: &IndexSet<String>,
    used_tag_classes: &IndexSet<String>,
) -> bool {
    // Optionally handle :hover suffix
    let (base, _hover) = strip_hover_suffix(sel);

    // tag-only
    if is_simple_tag(base) {
        return used_tags.contains(base)
            || used_tag_classes
                .iter()
                .any(|k| k.split('|').next() == Some(base));
    }

    // .class-only
    if let Some(class_name) = base.strip_prefix('.') {
        // Normalize potential escaped class names as-is
        return used_classes.contains(class_name)
            || used_tag_classes
                .iter()
                .any(|k| k.ends_with(&format!("|{}", class_name)));
    }

    // tag.class
    if let Some((tag, class_name)) = split_tag_class_selector(base) {
        let key = format!("{}|{}", tag, class_name);
        return used_tag_classes.contains(&key)
            || (used_tags.contains(&tag) && used_classes.contains(&class_name));
    }

    // Other complex selectors are currently ignored
    false
}

pub fn is_simple_tag(s: &str) -> bool {
    // Match simple HTML tag-ish identifiers
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub fn split_tag_class_selector(s: &str) -> Option<(String, String)> {
    // "tag.class" -> (tag, class)
    let mut parts = s.splitn(2, '.');
    let tag = parts.next()?.to_string();
    let class_name = parts.next()?.to_string();
    if tag.is_empty() || class_name.is_empty() {
        return None;
    }
    Some((tag, class_name))
}
