use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use serde_json::{json, Value};
use std::collections::HashMap;

use hbb_common::log;
use hbb_common::{ResultType, bail};
use serde_derive::{Deserialize, Serialize}; // Added for InitInfo

// Helper function to convert Rust String to C-compatible string
fn to_c_string(s: String) -> *mut c_char {
    CString::new(s)
        .expect("Failed to convert string to CString")
        .into_raw()
}

// Helper function to convert C-compatible string to Rust String
fn from_c_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(ptr) }
                .to_string_lossy()
                .into_owned(),
        )
    }
}

// Helper function to free strings allocated by the plugin
#[no_mangle]
pub extern "C" fn mcpdesk_obs_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        _ = CString::from_raw(ptr);
    }
}

// Convert &[u8] to *const c_char
fn str_to_cstr_ret(s: &str) -> *const c_char {
    CString::new(s).map_or(std::ptr::null(), |s| s.into_raw() as _)
}

// free C string
fn free_c_ptr(ptr: *mut c_char) {
    unsafe {
        if !ptr.is_null() {
            _ = CString::from_raw(ptr);
        }
    }
}

// =============================================================================
// Plugin ABI (from rustdesk/src/plugin/plugins.rs)
// =============================================================================

#[repr(C)]
#[derive(Copy, Clone)]
struct Callbacks {
    msg: extern "C" fn(
        peer: *const c_char,
        target: *const c_char,
        id: *const c_char,
        content: *const c_void,
        len: usize,
    ) -> PluginReturn,
    get_conf: extern "C" fn(peer: *const c_char, id: *const c_char, key: *const c_char) -> *const c_char,
    get_id: extern "C" fn() -> *const c_char,
    log: extern "C" fn(level: *const c_char, msg: *const c_char),
    // super::native::NativeReturnValue needs to be defined or imported
    // For now, use c_int as a placeholder for NativeReturnValue
    native: extern "C" fn(
        method: *const c_char,
        json: *const c_char,
        raw: *const c_void,
        raw_len: usize,
    ) -> c_int,
}

#[derive(Serialize, Deserialize)] // Added Deserialize for InitInfo
#[repr(C)]
struct InitInfo {
    is_server: bool,
    // Add id field here for PLUGIN_ID
    id: String, // Assuming plugin ID is passed in InitInfo
}

/// The plugin initialize data.
/// version: The version of the plugin, can't be nullptr.
/// local_peer_id: The local peer id, can't be nullptr.
/// cbs: The callbacks.
#[repr(C)]
struct InitData {
    version: *const c_char,
    info: *const c_char,
    cbs: Callbacks,
}

impl Drop for InitData {
    fn drop(&mut self) {
        // Only free if we actually own the memory, which is not the case for borrowed ptrs
        // free_c_ptr(self.version as _);
        // free_c_ptr(self.info as _);
    }
}

// Define PluginReturn (from rustdesk/src/plugin/errno.rs)
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PluginReturn {
    pub code: c_int,
    pub msg: *const c_char,
}

impl PluginReturn {
    pub const SUCCESS: Self = Self {
        code: 0,
        msg: std::ptr::null(),
    };

    pub fn is_success(&self) -> bool {
        self.code == 0
    }

    pub fn get_code_msg(&mut self, default_msg: &str) -> (i32, String) {
        let msg = if self.msg.is_null() {
            default_msg.to_string()
        } else {
            let s = unsafe { CStr::from_ptr(self.msg) }.to_string_lossy();
            let s = s.to_string();
            // Important: The plugin is responsible for freeing msg if it allocates it.
            // But this function does not free it. Host should free it.
            s
        };
        (self.code, msg)
    }

    pub fn from_err(err: &str) -> Self {
        Self {
            code: -1, // Generic error code
            msg: str_to_cstr_ret(err),
        }
    }
}

// =============================================================================
// Core Plugin ABI implementations
// =============================================================================

static mut GLOBAL_INIT_DATA: Option<InitData> = None;
static mut PLUGIN_ID: Option<String> = None;

#[no_mangle]
pub extern "C" fn init(data_ptr: *const InitData) -> PluginReturn {
    log::info!("Plugin 'mcpdesk_obs_plugin' init called");
    if data_ptr.is_null() {
        return PluginReturn::from_err("InitData is null");
    }
    unsafe {
        let data = &*data_ptr;
        GLOBAL_INIT_DATA = Some(*data); // Store a copy if needed, or just use the reference
        let info_str = from_c_string(data.info).unwrap_or_default();
        let init_info: InitInfo = serde_json::from_str(&info_str).unwrap_or_default();
        PLUGIN_ID = Some(init_info.id); // Assuming info contains plugin ID
    }
    PluginReturn::SUCCESS
}

#[no_mangle]
pub extern "C" fn reset(data_ptr: *const InitData) -> PluginReturn {
    log::info!("Plugin 'mcpdesk_obs_plugin' reset called");
    if data_ptr.is_null() {
        return PluginReturn::from_err("InitData is null");
    }
    unsafe {
        let data = &*data_ptr;
        GLOBAL_INIT_DATA = Some(*data);
    }
    PluginReturn::SUCCESS
}

#[no_mangle]
pub extern "C" fn clear() -> PluginReturn {
    log::info!("Plugin 'mcpdesk_obs_plugin' clear called");
    unsafe {
        GLOBAL_INIT_DATA = None;
        PLUGIN_ID = None;
    }
    PluginReturn::SUCCESS
}

#[no_mangle]
pub extern "C" fn desc() -> *const c_char {
    log::info!("Plugin 'mcpdesk_obs_plugin' desc called");
    let desc_str = json!({
        "id": "obs", // Matches plugin_obs name
        "name": "OBS Control Plugin",
        "version": "0.1.0",
        "description": "Control OBS Studio via MCP commands.",
        "platforms": "windows|linux|macos",
        "listen_events": ["obs_control_request"], // Custom event for MCP commands
        "config": {
            "test_config": "test_value"
        }
    }).to_string();
    str_to_cstr_ret(&desc_str)
}

#[no_mangle]
pub extern "C" fn call(
    method_ptr: *const c_char,
    peer_ptr: *const c_char,
    args_ptr: *const c_void,
    args_len: usize,
) -> PluginReturn {
    let method = match from_c_string(method_ptr) {
        Some(m) => m,
        None => return PluginReturn::from_err("Method is null"),
    };
    let peer = from_c_string(peer_ptr).unwrap_or_default();
    let args_slice = unsafe { std::slice::from_raw_parts(args_ptr as *const u8, args_len) };
    let args_str = String::from_utf8_lossy(args_slice);

    log::info!(
        "Plugin 'mcpdesk_obs_plugin' call called: method={}, peer={}, args={}",
        method, peer, args_str
    );

    // Dispatch based on method
    match method.as_str() {
        // OBS Control Commands
        "obs_start_streaming" => mcpdesk_obs_start_streaming(),
        "obs_stop_streaming" => mcpdesk_obs_stop_streaming(),
        "obs_set_scene" => {
            let args_json: Value = serde_json::from_str(&args_str).unwrap_or_default();
            let scene_name = args_json["scene_name"].as_str().unwrap_or_default();
            mcpdesk_obs_set_scene(str_to_cstr_ret(scene_name))
        }
        "obs_set_source_visibility" => {
            let args_json: Value = serde_json::from_str(&args_str).unwrap_or_default();
            let scene_name_ptr = args_json["scene_name"].as_str().map_or(std::ptr::null(), |s| str_to_cstr_ret(s));
            let source_name = args_json["source_name"].as_str().unwrap_or_default();
            let visible = args_json["visible"].as_bool().unwrap_or(false);
            mcpdesk_obs_set_source_visibility(scene_name_ptr, str_to_cstr_ret(source_name), visible)
        }
        "obs_set_streaming_settings" => {
            let args_json: Value = serde_json::from_str(&args_str).unwrap_or_default();
            mcpdesk_obs_set_streaming_settings(str_to_cstr_ret(&args_json.to_string()))
        }
        _ => PluginReturn::from_err(&format!("Unknown method: {}", method)),
    }
}

#[no_mangle]
pub extern "C" fn call_with_out_data(
    method_ptr: *const c_char,
    peer_ptr: *const c_char,
    args_ptr: *const c_void,
    args_len: usize,
    out_ptr: *mut *mut c_void,
    out_len_ptr: *mut usize,
) -> PluginReturn {
    let method = match from_c_string(method_ptr) {
        Some(m) => m,
        None => return PluginReturn::from_err("Method is null"),
    };
    let peer = from_c_string(peer_ptr).unwrap_or_default();
    let args_slice = unsafe { std::slice::from_raw_parts(args_ptr as *const u8, args_len) };
    let args_str = String::from_utf8_lossy(args_slice);

    log::info!(
        "Plugin 'mcpdesk_obs_plugin' call_with_out_data called: method={}, peer={}, args={}",
        method, peer, args_str
    );

    let mut result_json_ptr: *mut c_char = std::ptr::null_mut();
    let mut result_code = 0;

    match method.as_str() {
        "obs_get_scenes" => {
            let code = mcpdesk_obs_get_scenes(&mut result_json_ptr);
            result_code = code;
        }
        "obs_get_sources" => {
            let args_json: Value = serde_json::from_str(&args_str).unwrap_or_default();
            let scene_name_ptr = args_json["scene_name"].as_str().map_or(std::ptr::null(), |s| str_to_cstr_ret(s));
            let code = mcpdesk_obs_get_sources(scene_name_ptr, &mut result_json_ptr);
            result_code = code;
        }
        "obs_get_streaming_status" => {
            let code = mcpdesk_obs_get_streaming_status(&mut result_json_ptr);
            result_code = code;
        }
        _ => return PluginReturn::from_err(&format!("Unknown method with output: {}", method)),
    }

    if result_code == 0 {
        if result_json_ptr.is_null() {
            return PluginReturn::from_err("Plugin returned null output");
        }
        unsafe {
            let cstr = CStr::from_ptr(result_json_ptr);
            *out_ptr = result_json_ptr as *mut c_void;
            *out_len_ptr = cstr.to_bytes().len();
        }
        PluginReturn::SUCCESS
    } else {
        PluginReturn::from_err("Plugin call failed")
    }
}

// =============================================================================
// Existing mcpdesk_obs_* functions (moved from the original lib.rs content)
// =============================================================================

/// Starts streaming in OBS.
/// Returns 0 on success, non-zero on error.
pub extern "C" fn mcpdesk_obs_start_streaming() -> c_int {
    // Placeholder: Implement actual OBS interaction here
    log::info!("mcpdesk_obs_start_streaming called");
    // Example of success/failure
    0 // Success
}

/// Stops streaming in OBS.
/// Returns 0 on success, non-zero on error.
pub extern "C" fn mcpdesk_obs_stop_streaming() -> c_int {
    // Placeholder: Implement actual OBS interaction here
    log::info!("mcpdesk_obs_stop_streaming called");
    0 // Success
}

/// Sets the active scene in OBS.
/// `scene_name_ptr`: C-string for the scene name.
/// Returns 0 on success, non-zero on error.
pub extern "C" fn mcpdesk_obs_set_scene(scene_name_ptr: *const c_char) -> c_int {
    let scene_name = match from_c_string(scene_name_ptr) {
        Some(name) => name,
        None => {
            log::error!("Error: Scene name is NULL.");
            return -1; // Invalid argument
        }
    };
    // Placeholder: Implement actual OBS interaction here
    log::info!("mcpdesk_obs_set_scene called with scene: {}", scene_name);
    0 // Success
}

/// Gets a list of available scenes in OBS.
/// `output_json_ptr`: A pointer to a C-string pointer. The function allocates memory for a JSON string (array of scene names) and sets `*output_json_ptr` to point to it.
/// Returns 0 on success, non-zero on error.
pub extern "C" fn mcpdesk_obs_get_scenes(output_json_ptr: *mut *mut c_char) -> c_int {
    // Placeholder: Implement actual OBS interaction here
    log::info!("mcpdesk_obs_get_scenes called");
    let scenes = vec!["Scene 1", "Scene 2", "My Game Scene"];
    let json_output = json!(scenes).to_string();

    unsafe {
        *output_json_ptr = to_c_string(json_output);
    }
    0 // Success
}

/// Sets the visibility of a source in the current scene.
/// `scene_name_ptr`: C-string for the scene name (can be NULL for current scene).
/// `source_name_ptr`: C-string for the source name.
/// `visible`: `bool` (C `_Bool` or `int` interpreted as bool).
/// Returns 0 on success, non-zero on error.
pub extern "C" fn mcpdesk_obs_set_source_visibility(
    scene_name_ptr: *const c_char,
    source_name_ptr: *const c_char,
    visible: bool,
) -> c_int {
    let scene_name = from_c_string(scene_name_ptr); // Option<String>
    let source_name = match from_c_string(source_name_ptr) {
        Some(name) => name,
        None => {
            log::error!("Error: Source name is NULL.");
            return -1; // Invalid argument
        }
    };

    // Placeholder: Implement actual OBS interaction here
    match scene_name {
        Some(name) => log::info!(
            "mcpdesk_obs_set_source_visibility called for scene: {}, source: {}, visible: {}",
            name, source_name, visible
        ),
        None => log::info!(
            "mcpdesk_obs_set_source_visibility called for current scene, source: {}, visible: {}",
            source_name, visible
        ),
    }
    0 // Success
}

/// Gets a list of sources in the current scene.
/// `scene_name_ptr`: C-string for the scene name (can be NULL for current scene).
/// `output_json_ptr`: A pointer to a C-string pointer for a JSON string (array of source names).
/// Returns 0 on success, non-zero on error.
pub extern "C" fn mcpdesk_obs_get_sources(
    scene_name_ptr: *const c_char,
    output_json_ptr: *mut *mut c_char,
) -> c_int {
    let scene_name = from_c_string(scene_name_ptr); // Option<String>

    // Placeholder: Implement actual OBS interaction here
    let sources = match scene_name {
        Some(name) => {
            log::info!("mcpdesk_obs_get_sources called for scene: {}", name);
            vec![format!("Source A in {}", name), format!("Source B in {}", name)]
        }
        None => {
            log::info!("mcpdesk_obs_get_sources called for current scene");
            vec!["Main Cam", "Screen Capture", "Microphone"]
        }
    };
    let json_output = json!(sources).to_string();

    unsafe {
        *output_json_ptr = to_c_string(json_output);
    }
    0 // Success
}

/// Sets streaming quality/output settings.
/// `settings_json_ptr`: C-string for a JSON object representing settings.
/// Returns 0 on success, non-zero on error.
pub extern "C" fn mcpdesk_obs_set_streaming_settings(settings_json_ptr: *const c_char) -> c_int {
    let settings_json = match from_c_string(settings_json_ptr) {
        Some(json_str) => json_str,
        None => {
            log::error!("Error: Settings JSON is NULL.");
            return -1; // Invalid argument
        }
    };

    let settings: Value = match serde_json::from_str(&settings_json) {
        Ok(val) => val,
        Err(e) => {
            log::error!("Error parsing settings JSON: {}", e);
            return -2; // JSON parsing error
        }
    };

    // Placeholder: Implement actual OBS interaction here
    log::info!(
        "mcpdesk_obs_set_streaming_settings called with settings: {:?}",
        settings
    );
    0 // Success
}

/// Gets current streaming status (active/inactive, bitrate, FPS).
/// `output_json_ptr`: A pointer to a C-string pointer for a JSON string (status details).
/// Returns 0 on success, non-zero on error.
pub extern "C" fn mcpdesk_obs_get_streaming_status(output_json_ptr: *mut *mut c_char) -> c_int {
    // Placeholder: Implement actual OBS interaction here
    log::info!("mcpdesk_obs_get_streaming_status called");
    let status = json!({
        "streaming_active": true,
        "bitrate": 5000,
        "fps": 60.0,
        "output_skipped_frames": 10,
        "output_total_frames": 10000,
    })
    .to_string();

    unsafe {
        *output_json_ptr = to_c_string(status);
    }
    0 // Success
}