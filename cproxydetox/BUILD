load("@rules_rust//rust:defs.bzl", "rust_static_library")
load("@build_bazel_rules_swift//swift:swift.bzl", "swift_c_module")
load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")

rust_static_library(
    name = "cproxydetoximpl",
    srcs = [
        "src/lib.rs",
    ],
    aliases = aliases(),
    crate_features = select({
        "//:enable_negotiate": ["negotiate"],
        "//conditions:default": [],
    }),
    proc_macro_deps = all_crate_deps(
        proc_macro = True,
    ),
    deps = ["//proxydetoxlib"] + all_crate_deps(
        normal = True,
    ),
)

cc_library(
    name = "cproxydetox",
    hdrs = ["src/proxydetox.h"],
    includes = ["src"],
    visibility = ["//visibility:public"],
    deps = [":cproxydetoximpl"],
)

swift_c_module(
    name = "swiftlyproxydetox",
    module_map = ":proxydetox.modulemap",
    module_name = "Proxydetox",
    target_compatible_with = ["@platforms//os:macos"],
    visibility = ["//visibility:public"],
    deps = [
        ":cproxydetox",
    ],
)
