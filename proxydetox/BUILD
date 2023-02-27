load("@rules_pkg//:pkg.bzl", "pkg_zip")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_test")
load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")

rust_binary(
    name = "proxydetox",
    srcs = [
        "src/main.rs",
        "src/options.rs",
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
        "//proxydetoxlib",
    ] + all_crate_deps(
        normal = True,
    ),
)

rust_test(
    name = "proxydetox_test",
    srcs = [
        "src/options.rs",
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
        "//proxydetoxlib",
    ] + all_crate_deps(
        normal = True,
    ),
)

pkg_zip(
    name = "proxydetox.zip",
    srcs = [":proxydetox"],
    tags = ["manual"],
)
