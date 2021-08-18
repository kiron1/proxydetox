load("@rules_rust//rust:defs.bzl", "rust_binary")

rust_binary(
    name = "paceval",
    srcs = [
        "src/main.rs",
    ],
    visibility = ["//visibility:public"],
    deps = [
        "//cargo:argh",
        "//cargo:http",
        "//paclib",
    ],
)
