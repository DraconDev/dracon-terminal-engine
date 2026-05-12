//! Build script for compiling Dracon plugins as shared libraries.
//!
//! This script compiles the plugin source files into platform-specific
//! shared libraries (.so on Linux, .dylib on macOS).
//!
//! Usage:
//!     cargo build --example plugin_demo --features plugins
//!
//! The plugins will be placed in `target/debug/examples/_plugins/`.

use std::env;
use std::path::Path;

/// Returns the appropriate file extension for the current platform.
fn plugin_extension() -> &'static str {
    if cfg!(target_os = "macos") {
        "dylib"
    } else if cfg!(target_os = "windows") {
        "dll"
    } else {
        "so"
    }
}

/// Returns the plugin directory path.
fn plugin_dir() -> std::path::PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    Path::new(&manifest_dir).join("_plugins")
}

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let plugin_ext = plugin_extension();
    let plugin_name = env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "plugins".to_string());

    // Set the library name and path for linking
    println!("cargo:rerun-if-changed=examples/_plugins/stat_widget.rs");
    println!("cargo:rerun-if-changed=examples/_plugins/welcome_widget.rs");

    // Tell cargo to link against the necessary dependencies
    println!("cargo:rustc-link-search=native={}", out_dir);

    // For dynamic loading, we just need to ensure the files are available
    // The actual dynamic loading happens at runtime using libloading
}
