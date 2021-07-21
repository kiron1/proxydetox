extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=duktape");

    let bindings = bindgen::Builder::default()
        .header("src/duktape.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .allowlist_function("duk_.*")
        .allowlist_type("duk_.*")
        .allowlist_var("DUK_.*")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    cc::Build::new()
        .file("src/duktape.c")
        .include("src")
        .compile("libduktape.a");
    println!(
        "cargo:rustc-link-search=native={}",
        env::var("OUT_DIR").unwrap()
    );
}
