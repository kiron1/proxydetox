load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

rust_library(
    name = "detox_auth",
    srcs = [
        "src/lib.rs",
        "src/negotiate.rs",
        "src/netrc.rs",
    ],
    aliases = aliases(),
    crate_features = select({
        "//:enable_negotiate": ["negotiate"],
        "//conditions:default": [],
    }),
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
    visibility = ["//visibility:public"],
    deps = [
        "//detox_net",
        "//paclib",
    ] + select({
        "//:enable_negotiate": ["//spnego"],
        "//conditions:default": [],
    }) + all_crate_deps(
        normal = True,
    ),
)

rust_test(
    name = "detox_auth_test",
    size = "small",
    crate = ":detox_auth",
)
