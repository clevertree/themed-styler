use crate::{version, State, bridge_common};
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use log::{LevelFilter, debug, error};
use android_logger::Config;
use std::sync::RwLock;
use once_cell::sync::Lazy;

static STATE: Lazy<RwLock<Option<State>>> = Lazy::new(|| RwLock::new(None));

fn init_logger() {
    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Debug)
            .with_tag("ThemedStylerRust"),
    );
}

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
pub extern "system" fn Java_com_relay_client_ThemedStylerModule_nativeInitialize(
  mut env: JNIEnv,
  _class: JClass,
  themes_json: JString,
  display_density: f32,
  scaled_density: f32,
) {
  init_logger();
  let themes = jstring_to_string(&mut env, themes_json).unwrap_or_else(|| "{}".to_string());
  let themes_input = bridge_common::parse_themes_json(&themes);
  let mut state = bridge_common::build_state(themes_input);
  state.display_density = display_density;
  state.scaled_density = scaled_density;
  
  debug!("[nativeInitialize] density={} scaled={} current_theme={}", 
    state.display_density, state.scaled_density, state.current_theme);
    
  let mut global_state = STATE.write().unwrap();
  *global_state = Some(state);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_relay_client_ThemedStylerModule_nativeGetAndroidStyles(
  mut env: JNIEnv,
  _class: JClass,
  selector: JString,
  class_name: JString,
) -> jstring {
  let selector_str = jstring_to_string(&mut env, selector).unwrap_or_else(|| "div".to_string());
  let class_name_str = jstring_to_string(&mut env, class_name).unwrap_or_else(|| String::new());
  
  // Split classes and add dot prefix (offloading work to Rust)
  let classes_vec: Vec<String> = class_name_str
    .split_whitespace()
    .filter(|s| !s.is_empty())
    .map(|s| format!(".{}", s))
    .collect();

  let state_lock = STATE.read().unwrap();
  let state = match &*state_lock {
    Some(s) => s,
    None => {
      error!("[nativeGetAndroidStyles] STATE not initialized! Call nativeInitialize first.");
      return new_jstring(&mut env, "{}");
    }
  };
  
  let styles = state.android_styles_for(&selector_str, &classes_vec);
  
  match serde_json::to_string(&styles) {
    Ok(json) => new_jstring(&mut env, &json),
    Err(_) => new_jstring(&mut env, "{}"),
  }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_relay_client_ThemedStylerModule_nativeProcessStyles(
  mut env: JNIEnv,
  _class: JClass,
  styles_json: JString,
) -> jstring {
  let styles_str = jstring_to_string(&mut env, styles_json).unwrap_or_else(|| "{}".to_string());
  let styles: indexmap::IndexMap<String, serde_json::Value> = serde_json::from_str(&styles_str).unwrap_or_default();
  
  let state_lock = STATE.read().unwrap();
  let state = match &*state_lock {
    Some(s) => s,
    None => return new_jstring(&mut env, "{}"),
  };

  let processed = state.process_styles(styles);
  let json = serde_json::to_string(&processed).unwrap_or_else(|_| "{}".to_string());
  new_jstring(&mut env, &json)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_relay_client_ThemedStylerModule_nativeGetVersion(
  mut env: JNIEnv,
  _class: JClass,
) -> jstring {
  new_jstring(&mut env, version())
}
