use indexmap::IndexMap;
use once_cell::sync::Lazy;
use serde_json::json;
use crate::utils::{parse_prefixed_class, resolve_vars};

pub type CssProps = IndexMap<String, serde_json::Value>;

// Tailwind color palette - embedded from tailwind-colors.html
pub static TAILWIND_COLORS: Lazy<IndexMap<&'static str, IndexMap<&'static str, &'static str>>> = Lazy::new(|| {
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

pub fn get_tailwind_color(color_shade: &str) -> Option<String> {
    let parts: Vec<&str> = color_shade.split('-').collect();
    if parts.len() != 2 {
        return None;
    }
    let color_name = parts[0];
    let shade = parts[1];
    
    if let Some(hex) = TAILWIND_COLORS
        .get(color_name)
        .and_then(|shades| shades.get(shade))
    {
        return Some(hex.to_string());
    }
    
    None
}

pub fn get_tailwind_color_with_vars(color_shade: &str, vars: &IndexMap<String, String>) -> Option<String> {
    if let Some(hex) = get_tailwind_color(color_shade) {
        return Some(hex);
    }
    
    if let Some(val) = vars.get(color_shade) {
        return Some(val.clone());
    }
    
    if let Some(val) = vars.get(&format!("colors.{}", color_shade)) {
        return Some(val.clone());
    }
    
    if let Some(val) = vars.get(&format!("color.{}", color_shade)) {
        return Some(val.clone());
    }
    
    let parts: Vec<&str> = color_shade.split('-').collect();
    if parts.len() >= 1 {
        let color_name = parts[0];
        if let Some(val) = vars.get(color_name) {
            return Some(val.clone());
        }
        if let Some(val) = vars.get(&format!("color.{}", color_name)) {
            return Some(val.clone());
        }
    }
    
    None
}

pub fn dynamic_css_properties_for_class(class: &str, vars: &IndexMap<String, String>) -> Option<CssProps> {
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
        let parts: Vec<&str> = rest.split('-').collect();
        let valid_sides = ["t", "b", "l", "r", "x", "y"];
        let (side, color_or_width_parts) = if parts.len() > 1 && valid_sides.contains(&parts[0]) {
            (Some(parts[0]), &parts[1..])
        } else {
            (None, &parts[..])
        };
        if color_or_width_parts.len() == 2 {
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
    // width utilities
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
        None => { props.insert("border-width".into(), json!(&width_str)); }
        Some("t") => { props.insert("border-top-width".into(), json!(&width_str)); }
        Some("b") => { props.insert("border-bottom-width".into(), json!(&width_str)); }
        Some("l") => { props.insert("border-left-width".into(), json!(&width_str)); }
        Some("r") => { props.insert("border-right-width".into(), json!(&width_str)); }
        Some("x") => {
            props.insert("border-left-width".into(), json!(&width_str));
            props.insert("border-right-width".into(), json!(&width_str));
        }
        Some("y") => {
            props.insert("border-top-width".into(), json!(&width_str));
            props.insert("border-bottom-width".into(), json!(&width_str));
        }
        _ => { props.insert("border-width".into(), json!(&width_str)); }
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

fn parse_arbitrary_value(class: &str) -> Option<CssProps> {
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
