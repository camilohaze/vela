/*!
Build script for desktop runtime

This script generates FFI bindings for the desktop runtime.
Note: C++ compilation is disabled until Skia and platform dependencies are available.
*/

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to rerun this build script if any C++ files change
    println!("cargo:rerun-if-changed=cpp/");

    // TODO: Add C++ compilation when Skia is available
    // For now, just generate stub FFI bindings

    generate_bindings();
}

fn add_skia_libraries(target_os: &str) {
    match target_os {
        "windows" => {
            println!("cargo:rustc-link-lib=dylib=skia");
            println!("cargo:rustc-link-lib=dylib=skia.dll");
        }
        "macos" => {
            println!("cargo:rustc-link-lib=dylib=skia");
            println!("cargo:rustc-link-lib=framework=ApplicationServices");
        }
        "linux" => {
            println!("cargo:rustc-link-lib=dylib=skia");
            println!("cargo:rustc-link-lib=dylib=fontconfig");
            println!("cargo:rustc-link-lib=dylib=freetype");
        }
        _ => {}
    }
}

fn generate_bindings() {
    // Stub bindings are now implemented directly in bridge.rs
    // No file generation needed
}