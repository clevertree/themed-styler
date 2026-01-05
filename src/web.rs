use indexmap::IndexMap;
use crate::theme::State;
use crate::utils::{parse_prefixed_class, class_to_selector, resolve_vars};
use crate::tailwind::dynamic_css_properties_for_class;

impl State {
    pub fn css_for_element(&self, selector: &str, classes: &[String]) -> String {
        let (eff, vars) = self.effective_theme_all();
        let mut props: IndexMap<String, serde_json::Value> = IndexMap::new();

        if let Some(base_props) = eff.get(selector) {
            for (k, v) in base_props {
                props.insert(k.clone(), v.clone());
            }
        }

        for class in classes {
            let normalized_class = if class.starts_with('.') {
                class[1..].to_string()
            } else {
                class.clone()
            };
            
            let (_bp, _hover, base) = parse_prefixed_class(&normalized_class);
            let sel = class_to_selector(&base);
            
            if let Some(class_props) = eff.get(&sel) {
                for (k, v) in class_props {
                    props.insert(k.clone(), v.clone());
                }
                continue;
            }
            
            if let Some(dynamic_props) = dynamic_css_properties_for_class(&base, &vars) {
                for (k, v) in dynamic_props {
                    props.insert(k.clone(), v.clone());
                }
                continue;
            }
            
            if let Some(class_props) = eff.get(&base) {
                for (k, v) in class_props {
                    props.insert(k.clone(), v.clone());
                }
            }
        }

        let mut css = String::new();
        for (k, v) in props {
            let val_str = match v {
                serde_json::Value::String(s) => resolve_vars(&s, &vars),
                serde_json::Value::Number(n) => n.to_string(),
                _ => v.to_string(),
            };
            css.push_str(&format!("{}: {}; ", k, val_str));
        }
        css.trim().to_string()
    }
}
