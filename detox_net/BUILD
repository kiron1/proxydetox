load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "detox_net",
    srcs = [
        "src/host_and_port.rs",
        "src/io.rs",
        "src/keepalive.rs",
        "src/lib.rs",
        "src/metered.rs",
        "src/path_or_uri.rs",
        "src/stream.rs",
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

rust_test(
    name = "detox_net_test",
    size = "small",
    crate = ":detox_net",
)
