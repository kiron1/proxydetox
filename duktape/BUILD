load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "duktape",
    srcs = [
        "src/context.rs",
        "src/lib.rs",
        "src/stack.rs",
        "src/value.rs",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//cargo:libc",
        "//cargo:thiserror",
        "//duktape-sys",
    ],
)
