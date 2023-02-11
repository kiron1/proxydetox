load("@rules_rust//rust:defs.bzl", "rust_library")
load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")

rust_library(
    name = "duktape",
    srcs = [
        "src/context.rs",
        "src/lib.rs",
        "src/stack.rs",
        "src/value.rs",
    ],
    aliases = aliases(),
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
    visibility = ["//visibility:public"],
    deps = ["//duktape-sys"] + all_crate_deps(
        normal = True,
    ),
)
