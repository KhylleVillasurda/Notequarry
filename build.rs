fn main() {
    slint_build::compile_with_config(
        "ui_files/main_window.slint",
        slint_build::CompilerConfiguration::new().with_style("fluent".to_string()),
    )
    .unwrap();
}

// old build code: slint_build::compile("ui_files/main_window.slint").unwrap();
