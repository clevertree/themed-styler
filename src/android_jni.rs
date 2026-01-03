use crate::{version, State, bridge_common::{self, UsageSnapshot}};
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;

fn jstring_to_string(env: &mut JNIEnv, input: JString) -> Option<String> {
  if input.is_null() {
    return None;
  }
  match env.get_string(&input) {
    Ok(v) => v.to_str().ok().map(|s| s.to_string()),
    Err(_) => None,
  }
}

fn new_jstring(env: &mut JNIEnv, value: &str) -> jstring {
  match env.new_string(value) {
    Ok(jstr) => jstr.into_raw(),
    Err(_) => {
      match env.new_string("") {
        Ok(jstr) => jstr.into_raw(),
        Err(_) => std::ptr::null_mut(),
      }
    }
  }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_relay_client_ThemedStylerModule_nativeRenderCss(
  env: JNIEnv,
  class: JClass,
  usage_json: JString,
  themes_json: JString,
) -> jstring {
  Java_com_relay_pure_ThemedStylerModule_nativeRenderCss(env, class, usage_json, themes_json)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_relay_pure_ThemedStylerModule_nativeRenderCss(
  mut env: JNIEnv,
  _class: JClass,
  usage_json: JString,
  themes_json: JString,
) -> jstring {
  let usage = jstring_to_string(&mut env, usage_json).unwrap_or_else(|| "{}".to_string());
  let themes = jstring_to_string(&mut env, themes_json).unwrap_or_else(|| "{}".to_string());
  let snapshot = bridge_common::parse_usage_json(&usage);
  let themes_input = bridge_common::parse_themes_json(&themes);
  let state = bridge_common::build_state(snapshot, themes_input);
  let css = state.css_for_web();
  new_jstring(&mut env, &css)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_relay_client_ThemedStylerModule_nativeGetAndroidStyles(
  env: JNIEnv,
  class: JClass,
  selector: JString,
  classes_json: JString,
  themes_json: JString,
) -> jstring {
  Java_com_relay_pure_ThemedStylerModule_nativeGetAndroidStyles(env, class, selector, classes_json, themes_json)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_relay_pure_ThemedStylerModule_nativeGetAndroidStyles(
  mut env: JNIEnv,
  _class: JClass,
  selector: JString,
  classes_json: JString,
  themes_json: JString,
) -> jstring {
  let selector_str = jstring_to_string(&mut env, selector).unwrap_or_else(|| String::new());
  let classes = jstring_to_string(&mut env, classes_json).unwrap_or_else(|| "[]".to_string());
  let themes = jstring_to_string(&mut env, themes_json).unwrap_or_else(|| "{}".to_string());
  
  let classes_vec: Vec<String> = serde_json::from_str(&classes).unwrap_or_default();
  let themes_input = bridge_common::parse_themes_json(&themes);
  
  // DEBUG: Check if themes were parsed
  let themes_count = themes_input.themes.len();
  let current_theme_str = themes_input.current_theme.clone().unwrap_or_else(|| "none".to_string());
  
  let mut state = bridge_common::build_state(UsageSnapshot::default(), themes_input);
  
  // Extract display density from themes if provided
  if let Ok(themes_obj) = serde_json::from_str::<serde_json::Value>(&themes) {
    if let Some(density) = themes_obj.get("displayDensity").and_then(|v| v.as_f64()) {
      state.display_density = density as f32;
    }
    if let Some(scaled) = themes_obj.get("scaledDensity").and_then(|v| v.as_f64()) {
      state.scaled_density = scaled as f32;
    }
  }
  
  // Use Android-specific style transformation
  let styles = state.android_styles_for(&selector_str, &classes_vec);
  
  // Include debug info in result if nav-btn
  let debug_info = if selector_str.contains("button") && classes_vec.iter().any(|c| c.contains("nav-btn")) {
    format!(";DEBUG:parsed_themes={},current_theme={},state_themes={},style_props={}", 
      themes_count, current_theme_str, state.themes.len(), styles.len())
  } else {
    String::new()
  };

  match serde_json::to_string(&styles) {
    Ok(json) => {
      let result = if debug_info.is_empty() { json } else { format!("{}\"_debug\":\"{}\"}}", &json[..json.len()-1], debug_info) };
      new_jstring(&mut env, &result)
    },
    Err(_) => new_jstring(&mut env, "{}"),
  }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_relay_client_ThemedStylerModule_nativeGetDefaultState(
  env: JNIEnv,
  class: JClass,
) -> jstring {
  Java_com_relay_pure_ThemedStylerModule_nativeGetDefaultState(env, class)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_relay_pure_ThemedStylerModule_nativeGetDefaultState(
  mut env: JNIEnv,
  _class: JClass,
) -> jstring {
  let state = State::new_default();
  match serde_json::to_string(&state) {
    Ok(json) => new_jstring(&mut env, &json),
    Err(_) => new_jstring(&mut env, "{}"),
  }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_relay_client_ThemedStylerModule_nativeGetVersion(
  env: JNIEnv,
  class: JClass,
) -> jstring {
  Java_com_relay_pure_ThemedStylerModule_nativeGetVersion(env, class)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_relay_pure_ThemedStylerModule_nativeGetVersion(
  mut env: JNIEnv,
  _class: JClass,
) -> jstring {
  new_jstring(&mut env, version())
}
