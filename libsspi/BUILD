load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "libsspi",
    srcs = [
        "src/lib.rs",
        "src/win.rs",
    ],
    visibility = ["//visibility:public"],
    deps = [
    ] + select({
        "@bazel_tools//src/conditions:windows": ["//libsspi/cargo:windows"],
        "//conditions:default": [],
    }),
)
