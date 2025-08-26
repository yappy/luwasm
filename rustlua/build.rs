use std::env;
use std::path::PathBuf;

fn bindgen(header: &str, allowfile: &str, rs: &str) {
    let sysroot = format!(
        "{}/upstream/emscripten/cache/sysroot",
        env::var("EMSDK").unwrap()
    );
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindgen::Builder::default()
        .clang_arg(format!("--sysroot={sysroot}"))
        //.clang_arg(format!("-I{sysroot}/include"))
        // If `clang -target wasm32-unknown-emscripten`,
        // the default visibility of functions (in elf) is hidden.
        // https://github.com/rust-lang/rust-bindgen/issues/751#issuecomment-496891269
        .clang_arg("-fvisibility=default")
        .header(format!("{sysroot}/include/{header}"))
        .allowlist_file(format!("{sysroot}/include/{allowfile}"))
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("bindgen failed")
        .write_to_file(out_dir.join(rs))
        .expect("write failed");
}

fn main() {
    bindgen(
        "emscripten/emscripten.h",
        "emscripten/.*",
        "bind_emscripten.rs",
    );
    bindgen("emscripten/html5.h", "emscripten/.*", "bind_html5.rs");
    bindgen("SDL/SDL.h", "SDL/.*", "bind_sdl.rs");
}
