load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "detox_hyper",
    srcs = [
        "src/conn.rs",
        "src/http.rs",
        "src/lib.rs",
    ],
    aliases = aliases(),
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
    visibility = ["//visibility:public"],
    deps = [
        "//detox_auth",
        "//detox_futures",
        "//detox_net",
        "//paclib",
    ] + [
    ] + all_crate_deps(
        normal = True,
    ),
)

rust_test(
    name = "detox_hyper_test",
    size = "small",
    crate = ":detox_hyper",
)
