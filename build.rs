fn main() {
    let config = slint_build::CompilerConfiguration::new().with_library_paths(
        std::collections::HashMap::from([(
            "material".to_string(),
            std::path::Path::new("ui/material/material.slint").to_path_buf(),
        )]),
    );
    slint_build::compile_with_config("ui/appwindow.slint", config).unwrap();
}