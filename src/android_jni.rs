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
pub extern "system" fn Java_com_relay_client_ThemedStylerModule_nativeGetRnStyles(
  env: JNIEnv,
  class: JClass,
  selector: JString,
  classes_json: JString,
  themes_json: JString,
) -> jstring {
  Java_com_relay_pure_ThemedStylerModule_nativeGetRnStyles(env, class, selector, classes_json, themes_json)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_relay_pure_ThemedStylerModule_nativeGetRnStyles(
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
  let state = bridge_common::build_state(UsageSnapshot::default(), themes_input);
  let styles = state.rn_styles_for(&selector_str, &classes_vec);
  match serde_json::to_string(&styles) {
    Ok(json) => new_jstring(&mut env, &json),
    Err(_) => new_jstring(&mut env, "{}"),
  }
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
  let state = bridge_common::build_state(UsageSnapshot::default(), themes_input);
  let mut styles = state.rn_styles_for(&selector_str, &classes_vec);

  // Android-specific layout enhancements
  if selector_str == "div" || selector_str == "view" {
    if styles.get("flexDirection").map_or(false, |v| v.as_str() == Some("row")) {
      styles.insert("width".to_string(), serde_json::json!("match_parent"));
    } else if !styles.contains_key("width") {
      styles.insert("width".to_string(), serde_json::json!("match_parent"));
    }
    if selector_str == "div" && !styles.contains_key("height") {
      styles.insert("height".to_string(), serde_json::json!("wrap_content"));
    }
  }
  if selector_str == "span" || selector_str == "text" {
    if !styles.contains_key("width") {
      styles.insert("width".to_string(), serde_json::json!("wrap_content"));
    }
    if !styles.contains_key("height") {
      styles.insert("height".to_string(), serde_json::json!("wrap_content"));
    }
  }

  match serde_json::to_string(&styles) {
    Ok(json) => new_jstring(&mut env, &json),
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
