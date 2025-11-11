// build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    // UPDATE THIS LINE with your actual Qt Creator build folder!
    let qt_build_folder = "build\\Desktop_Qt_6_10_0_MinGW_64_bit-Debug\\bin"; // Change this!
    
    let qt_build_dir = PathBuf::from(&manifest_dir).join(qt_build_folder);
    
    // Tell Cargo where to find the library
    // Try both debug and release folders
    println!("cargo:rustc-link-search=native={}/debug", qt_build_dir.display());
    println!("cargo:rustc-link-search=native={}/release", qt_build_dir.display());
    println!("cargo:rustc-link-search=native={}", qt_build_dir.display());
    
    // Link the library
    println!("cargo:rustc-link-lib=dylib=notequarry_ui");
    
    // Qt libraries
    println!("cargo:rustc-link-search=C:/Qt/6.10.0/mingw_64/lib");
    println!("cargo:rustc-link-search=C:/Qt/6.10.0/mingw_64/bin");
    
    println!("cargo:rerun-if-changed=src/ui/mainwindow.h");
    println!("cargo:rerun-if-changed=src/ui/mainwindow.cpp");
    println!("cargo:rerun-if-changed=src/ui/qt_bridge.h");
    println!("cargo:rerun-if-changed=src/ui/qt_bridge.cpp");
}