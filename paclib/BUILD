load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")
load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")

rust_library(
    name = "paclib",
    srcs = [
        "src/dns.rs",
        "src/engine.rs",
        "src/evaluator.rs",
        "src/lib.rs",
        "src/proxy.rs",
    ],
    aliases = aliases(),
    compile_data = [
        "src/pac_utils.js",
    ],
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
    visibility = ["//visibility:public"],
    deps = [
        "//detox_net",
    ] + all_crate_deps(
        normal = True,
    ),
)

rust_test(
    name = "paclib_test",
    crate = ":paclib",
)
