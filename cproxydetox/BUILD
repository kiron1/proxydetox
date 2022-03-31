load("@rules_rust//rust:defs.bzl", "rust_static_library")
load("@build_bazel_rules_swift//swift:swift.bzl", "swift_c_module")

rust_static_library(
    name = "cproxydetoximpl",
    srcs = [
        "src/lib.rs",
    ],
    crate_features = select({
        "//:enable_negotiate": ["negotiate"],
        "//conditions:default": [],
    }),
    deps = [
        "//cproxydetox/cargo:dirs",
        "//cproxydetox/cargo:hyper",
        "//cproxydetox/cargo:libc",
        "//cproxydetox/cargo:tokio",
        "//proxydetox:proxydetoxlib",
    ],
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
