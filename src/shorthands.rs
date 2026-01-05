use indexmap::IndexMap;
use crate::units::parse_and_convert_to_px;

pub fn expand_shorthands(styles: &mut IndexMap<String, serde_json::Value>) {
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
}

pub fn convert_dimensions_to_px(styles: &mut IndexMap<String, serde_json::Value>, density: f32) {
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
}
