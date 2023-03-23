load("@rules_rust//rust:defs.bzl", "rust_library")
load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")

rust_library(
    name = "detox_net",
    srcs = [
        "src/host_and_port.rs",
        "src/keepalive.rs",
        "src/lib.rs",
        "src/metered.rs",
        "src/path_or_uri.rs",
    ],
    aliases = aliases(),
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
    visibility = ["//visibility:public"],
    deps = all_crate_deps(
        normal = True,
    ),
)
