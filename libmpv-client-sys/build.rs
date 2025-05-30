use std::env;
use std::path::PathBuf;
use bindgen::{RustEdition, RustTarget};

fn main() {
    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        // .dynamic_library_name("mpv")
        // .dynamic_link_require_all(true)
        .default_enum_style(bindgen::EnumVariation::Consts)
        .opaque_type("mpv_handle")
        .clang_macro_fallback()
        .merge_extern_blocks(true);

    #[cfg(target_os = "windows")]
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
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
