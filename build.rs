use cpp_build;

fn main() {
    // Configure Qt build
    cpp_build::Config::new()
        .include("/usr/include/qt6")  // Adjust for your Qt installation
        .include("/usr/include/qt6/QtCore")
        .include("/usr/include/qt6/QtWidgets")
        .include("/usr/include/qt6/QtGui")
        .flag("-std=c++17")
        .flag("-fPIC")
        .build("src/main.rs");
    
    println!("cargo:rustc-link-search=C:/Qt/6.6.0/mingw_64/lib");
    println!("cargo:rustc-link-lib=Qt6Core");
    println!("cargo:rustc-link-lib=Qt6Widgets");
    println!("cargo:rustc-link-lib=Qt6Gui");

    println!("cargo:rustc-link-search=./build/lib");
    println!("cargo:rustc-link-lib=notequarry_ui");
    
    println!("cargo:rerun-if-changed=src/ui/mainwindow.h");
    println!("cargo:rerun-if-changed=src/ui/mainwindow.cpp");
}