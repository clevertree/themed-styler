use crate::bridge_common;
use std::ffi::{CStr, CString, c_char};
use std::ptr;

#[unsafe(no_mangle)]
pub extern "C" fn themed_styler_version() -> *const c_char {
    static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
    VERSION.as_ptr() as *const c_char
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn themed_styler_render_css(
    usage_json: *const c_char,
    themes_json: *const c_char,
) -> *mut c_char {
    if usage_json.is_null() || themes_json.is_null() {
        return ptr::null_mut();
    }

    let usage_str = match unsafe { CStr::from_ptr(usage_json).to_str() } {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let themes_str = match unsafe { CStr::from_ptr(themes_json).to_str() } {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let snapshot = bridge_common::parse_usage_json(usage_str);
    let themes_input = bridge_common::parse_themes_json(themes_str);
    let state = bridge_common::build_state(snapshot, themes_input);
    let css = state.css_for_web();
    
    match CString::new(css) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn themed_styler_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)) };
    }
}
