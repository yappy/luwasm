use std::env;
use std::path::PathBuf;

fn main() {
    let sysroot = format!(
        "{}/upstream/emscripten/cache/sysroot",
        env::var("EMSDK").unwrap()
    );

    //*
    let bindings = bindgen::Builder::default()
        //.clang_arg(format!("--sysroot={sysroot}"))
        .clang_arg(format!("-I{sysroot}/include"))
        // If `clang -target wasm32-unknown-emscripten`,
        // the default visibility of functions (in elf) is hidden.
        // https://github.com/rust-lang/rust-bindgen/issues/751#issuecomment-496891269
        .clang_arg("-fvisibility=default")
        .header(format!("{sysroot}/include/emscripten/emscripten.h"))
        .header(format!("{sysroot}/include/emscripten/console.h"))
        .header(format!("{sysroot}/include/emscripten/html5.h"))
        .allowlist_file(format!("{sysroot}/include/emscripten/.*"))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("bindgen failed");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_dir.join("bind_em.rs"))
        .expect("write failed");
    // */
}
