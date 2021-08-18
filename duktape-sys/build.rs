extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR");
    let bindings_rs = "bindings.rs";

    println!("cargo:rustc-link-lib=duktape");

    let bindings = bindgen::Builder::default()
        .header("src/duktape.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .allowlist_function("duk_.*")
        .allowlist_type("duk_.*")
        .allowlist_var("DUK_.*")
        .generate()
        .expect("generate bindings");

    let out_path = PathBuf::from(&out_dir);
    let out_path = out_path.join(bindings_rs);
    bindings
        .write_to_file(&out_path)
        .expect("write bindings to file");
    println!(
        "cargo:rustc-env=DUKTAPE_BINDINGS_RS={}",
        &out_path.display()
    );

    cc::Build::new()
        .file("src/duktape.c")
        .include("src")
        .compile("libduktape.a");
    println!(
        "cargo:rustc-link-search=native={}",
        env::var("OUT_DIR").unwrap()
    );
}
