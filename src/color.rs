use indexmap::IndexMap;
use serde_json::json;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()? as f32 / 255.0;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()? as f32 / 255.0;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()? as f32 / 255.0;
                Some(Color { r, g, b, a: 1.0 })
            }
            4 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()? as f32 / 255.0;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()? as f32 / 255.0;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()? as f32 / 255.0;
                let a = u8::from_str_radix(&hex[3..4].repeat(2), 16).ok()? as f32 / 255.0;
                Some(Color { r, g, b, a })
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
                Some(Color { r, g, b, a: 1.0 })
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()? as f32 / 255.0;
                Some(Color { r, g, b, a })
            }
            _ => None,
        }
    }

    pub fn to_hex(&self) -> String {
        if self.a >= 1.0 {
            format!("#{:02x}{:02x}{:02x}", (self.r * 255.0) as u8, (self.g * 255.0) as u8, (self.b * 255.0) as u8)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", (self.r * 255.0) as u8, (self.g * 255.0) as u8, (self.b * 255.0) as u8, (self.a * 255.0) as u8)
        }
    }

    pub fn mix(c1: Color, c2: Color, weight: f32) -> Color {
        let w = weight.clamp(0.0, 1.0);
        let w1 = 1.0 - w;
        Color {
            r: c1.r * w1 + c2.r * w,
            g: c1.g * w1 + c2.g * w,
            b: c1.b * w1 + c2.b * w,
            a: c1.a * w1 + c2.a * w,
        }
    }
}

pub fn resolve_color(
    value: &str,
    current_color: Option<&str>,
    vars: &IndexMap<String, String>,
) -> String {
    let value = value.trim();
    
    if value == "currentColor" {
        return current_color.unwrap_or("#000000").to_string();
    }

    if value == "transparent" {
        return "#00000000".to_string();
    }

    if value == "black" {
        return "#000000".to_string();
    }

    if value == "white" {
        return "#ffffff".to_string();
    }

    if value == "gray" || value == "grey" {
        return "#808080".to_string();
    }

    if value == "red" {
        return "#ff0000".to_string();
    }

    if value == "green" {
        return "#00ff00".to_string();
    }

    if value == "blue" {
        return "#0000ff".to_string();
    }

    if value.starts_with("color-mix(") && value.ends_with(')') {
        return resolve_color_mix(value, current_color, vars);
    }

    if value.starts_with('#') {
        return value.to_string();
    }

    // Fallback to variable resolution if not already done
    value.to_string()
}

fn resolve_color_mix(
    value: &str,
    current_color: Option<&str>,
    vars: &IndexMap<String, String>,
) -> String {
    // color-mix(in srgb, color1 [percentage], color2 [percentage])
    let content = &value[10..value.len() - 1];
    let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
    
    if parts.len() < 3 {
        return "#000000".to_string();
    }

    log::debug!("[resolve_color_mix] content={} current_color={:?}", content, current_color);

    // Skip "in srgb"
    let c1_part = parts[1];
    let c2_part = parts[2];

    let (c1_str, p1) = parse_color_and_percentage(c1_part);
    let (c2_str, p2) = parse_color_and_percentage(c2_part);

    let c1_hex = resolve_color(c1_str, current_color, vars);
    let c2_hex = resolve_color(c2_str, current_color, vars);

    log::debug!("[resolve_color_mix] c1_str={} -> c1_hex={}, c2_str={} -> c2_hex={}", c1_str, c1_hex, c2_str, c2_hex);

    let c1 = Color::from_hex(&c1_hex).unwrap_or(Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 });
    let c2 = Color::from_hex(&c2_hex).unwrap_or(Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 });

    let weight = if let Some(p) = p2 {
        p / 100.0
    } else if let Some(p) = p1 {
        1.0 - (p / 100.0)
    } else {
        0.5
    };

    let result = Color::mix(c1, c2, weight).to_hex();
    log::debug!("[resolve_color_mix] weight={} result={}", weight, result);
    result
}

fn parse_color_and_percentage(part: &str) -> (&str, Option<f32>) {
    let parts: Vec<&str> = part.split_whitespace().collect();
    if parts.len() == 2 {
        if let Some(p) = parts[1].strip_suffix('%') {
            if let Ok(val) = p.parse::<f32>() {
                return (parts[0], Some(val));
            }
        }
    }
    (part, None)
}
