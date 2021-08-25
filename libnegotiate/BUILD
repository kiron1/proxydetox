load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "libnegotiate",
    srcs = [
        "src/gssapi.rs",
        "src/lib.rs",
        "src/sspi.rs",
    ],
    crate_features = ["gssapi"],
    tags = ["manual"],
    visibility = ["//visibility:public"],
    deps = [
        "//duktape-sys:duktape-sys",
    ] + select({
        "@bazel_tools//src/conditions:windows": ["//libsspi:libsspi"],
        "//conditions:default": ["//libnegotiate/cargo:libgssapi"],
    }),
)
