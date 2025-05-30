use std::env;
use std::path::PathBuf;
use bindgen::{CodegenConfig, RustEdition, RustTarget};

fn generate_data_bindings() {
    let mut config = CodegenConfig::empty();
    config.insert(CodegenConfig::TYPES);
    config.insert(CodegenConfig::VARS);

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .blocklist_var("pfn_.*")
        .default_enum_style(bindgen::EnumVariation::Consts)
        .opaque_type("mpv_handle")
        .clang_macro_fallback()
        .with_codegen_config(config);

    builder = builder
        .rust_target(RustTarget::nightly())
        .rust_edition(RustEdition::Edition2024)
        .wrap_unsafe_ops(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    let bindings = builder
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("data_bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn generate_pfns_bindings() {
    let mut config = CodegenConfig::empty();
    config.insert(CodegenConfig::VARS);

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .allowlist_var("pfn_.*")
        .merge_extern_blocks(true)
        .clang_macro_fallback()
        .with_codegen_config(config);

    #[cfg(feature = "dyn-sym")]
    {
        builder = builder.clang_arg("-DMPV_CPLUGIN_DYNAMIC_SYM");
    }

    builder = builder
        .rust_target(RustTarget::nightly())
        .rust_edition(RustEdition::Edition2024)
        .wrap_unsafe_ops(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    let bindings = builder
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("pfn_bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn generate_func_bindings() {
    let mut config = CodegenConfig::empty();
    config.insert(CodegenConfig::FUNCTIONS);

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .allowlist_function("mpv_.*")
        .merge_extern_blocks(true)
        .clang_macro_fallback()
        .with_codegen_config(config);

    #[cfg(feature = "dyn-sym")]
    {
        builder = builder.clang_arg("-DMPV_CPLUGIN_DYNAMIC_SYM");
    }

    builder = builder
        .rust_target(RustTarget::nightly())
        .rust_edition(RustEdition::Edition2024)
        .wrap_unsafe_ops(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    let bindings = builder
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("func_bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn generate_bindings() {
    generate_data_bindings();
    generate_pfns_bindings();
    generate_func_bindings();
}

fn main() {
    generate_bindings()
}
