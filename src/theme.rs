use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize, Deserializer};
use crate::{CssProps, SelectorStyles};

fn default_display_density() -> f32 { 1.0 }
fn default_scaled_density() -> f32 { 1.0 }

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
    pub themes: IndexMap<String, ThemeEntry>,
    pub default_theme: String,
    pub current_theme: String,
    #[serde(default = "default_display_density")]
    pub display_density: f32,
    #[serde(default = "default_scaled_density")]
    pub scaled_density: f32,
    
    #[serde(default)]
    pub used_classes: IndexSet<String>,
    #[serde(default)]
    pub used_tags: IndexSet<String>,
    #[serde(default)]
    pub used_tag_classes: IndexSet<String>,
}

impl State {
    pub fn new_default() -> Self {
        Self {
            themes: IndexMap::new(),
            default_theme: "default".to_string(),
            current_theme: "default".to_string(),
            display_density: 1.0,
            scaled_density: 1.0,
            used_classes: IndexSet::new(),
            used_tags: IndexSet::new(),
            used_tag_classes: IndexSet::new(),
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
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

    pub fn set_default_theme(&mut self, name: impl Into<String>) {
        self.default_theme = name.into();
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

    pub fn register_selectors<I: IntoIterator<Item = String>>(&mut self, _selectors: I) {
        // Deprecated: selectors are now part of the theme entry
    }

    pub fn effective_theme_all(&self) -> (SelectorStyles, IndexMap<String, String>) {
        let mut eff_selectors = SelectorStyles::new();
        let mut eff_vars = IndexMap::new();
        
        let theme_name = if self.themes.contains_key(&self.current_theme) {
            &self.current_theme
        } else if self.themes.contains_key(&self.default_theme) {
            &self.default_theme
        } else if !self.themes.is_empty() {
            self.themes.keys().next().unwrap()
        } else {
            return (eff_selectors, eff_vars);
        };

        let mut chain = Vec::new();
        let mut curr = Some(theme_name.as_str());
        while let Some(name) = curr {
            if chain.contains(&name) { break; }
            chain.push(name);
            curr = self.themes.get(name).and_then(|t| t.inherits.as_deref());
        }
        
        for name in chain.into_iter().rev() {
            if let Some(theme) = self.themes.get(name) {
                for (sel, props) in &theme.selectors {
                    let entry = eff_selectors.entry(sel.clone()).or_default();
                    for (k, v) in props {
                        entry.insert(k.clone(), v.clone());
                    }
                }
                for (k, v) in &theme.variables {
                    eff_vars.insert(k.clone(), v.clone());
                }
            }
        }
        
        (eff_selectors, eff_vars)
    }

    pub fn effective_breakpoints(&self) -> IndexMap<String, String> {
        let mut eff_breakpoints = IndexMap::new();
        
        let theme_name = if self.themes.contains_key(&self.current_theme) {
            &self.current_theme
        } else if self.themes.contains_key(&self.default_theme) {
            &self.default_theme
        } else if !self.themes.is_empty() {
            self.themes.keys().next().unwrap()
        } else {
            return eff_breakpoints;
        };

        let mut chain = Vec::new();
        let mut curr = Some(theme_name.as_str());
        while let Some(name) = curr {
            if chain.contains(&name) { break; }
            chain.push(name);
            curr = self.themes.get(name).and_then(|t| t.inherits.as_deref());
        }
        
        for name in chain.into_iter().rev() {
            if let Some(theme) = self.themes.get(name) {
                for (k, v) in &theme.breakpoints {
                    eff_breakpoints.insert(k.clone(), v.clone());
                }
            }
        }
        
        eff_breakpoints
    }
}

pub fn bundled_state() -> State {
    State::new_default()
}
